//! Window creation and the reactive render loop.
//!
//! [`run`] opens a single window; [`AppBuilder`] opens several, all sharing
//! one process and one winit event loop — e.g. a desktop-shell dock where
//! each icon/panel is its own window but spawning a process per icon would
//! multiply fixed per-process overhead (runtime, allocator, embedded font,
//! and — for the GPU backend — the graphics driver) for no benefit. Every
//! window keeps its own reactive state, so a signal change in one never
//! touches another's frame.
//!
//! On startup, and again on the next compositor frame after a
//! [`creamui_reactive::Signal`] read while building a window's UI changes,
//! that window's widget tree is rebuilt, its layout recomputed, it's repainted via
//! [`crate::painter::SkiaPainter`], and a redraw is requested; the actual
//! `RedrawRequested` handler only uploads the already-painted buffer to that
//! window's presenter (GPU or CPU — see [`crate::backend::RenderBackend`])
//! and presents it.

use crate::backend::RenderBackend;
use crate::cpu::CpuState;
use crate::gpu::GpuState;
use crate::painter::SkiaPainter;
use creamui_core::{
    BoxedWidget, CursorIcon, Key, KeyInput, Modifiers, Point, Renderer, Scene, Size,
};
use creamui_reactive::{create_effect, Effect, Signal};
use creamui_theme::Color;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey};
use winit::window::{
    CursorIcon as WinitCursorIcon, Window, WindowAttributes, WindowId, WindowLevel,
};

/// How long the text-input caret stays in each visibility phase while
/// blinking (on, then off, then on again).
const CARET_BLINK_INTERVAL: Duration = Duration::from_millis(530);

/// Writes `value` to `signal` only if it differs, avoiding a needless
/// re-render when a window manager fires a resize event with no real change.
fn set_if_changed<T: Clone + PartialEq + 'static>(signal: &Signal<T>, value: T) {
    if signal.peek() != value {
        signal.set(value);
    }
}

fn translate_cursor_icon(icon: CursorIcon) -> WinitCursorIcon {
    match icon {
        CursorIcon::Default => WinitCursorIcon::Default,
        CursorIcon::Text => WinitCursorIcon::Text,
        CursorIcon::Pointer => WinitCursorIcon::Pointer,
        CursorIcon::NotAllowed => WinitCursorIcon::NotAllowed,
    }
}

/// Translates a winit logical key into CreamUI's backend-agnostic [`Key`].
/// Returns `None` for keys with no CreamUI meaning (modifiers, function
/// keys, etc.) — those are silently ignored rather than delivered.
fn translate_key(key: &WinitKey) -> Option<Key> {
    match key {
        WinitKey::Character(s) => s.chars().next().map(Key::Char),
        WinitKey::Named(NamedKey::Space) => Some(Key::Char(' ')),
        WinitKey::Named(NamedKey::Backspace) => Some(Key::Backspace),
        WinitKey::Named(NamedKey::Delete) => Some(Key::Delete),
        WinitKey::Named(NamedKey::Enter) => Some(Key::Enter),
        WinitKey::Named(NamedKey::Tab) => Some(Key::Tab),
        WinitKey::Named(NamedKey::Escape) => Some(Key::Escape),
        WinitKey::Named(NamedKey::ArrowLeft) => Some(Key::Left),
        WinitKey::Named(NamedKey::ArrowRight) => Some(Key::Right),
        WinitKey::Named(NamedKey::ArrowUp) => Some(Key::Up),
        WinitKey::Named(NamedKey::ArrowDown) => Some(Key::Down),
        WinitKey::Named(NamedKey::Home) => Some(Key::Home),
        WinitKey::Named(NamedKey::End) => Some(Key::End),
        _ => None,
    }
}

/// Options for a window CreamUI opens, set once at startup.
///
/// This intentionally covers only what's needed to host anything from a
/// full application window to a borderless desktop-shell widget
/// (`decorations: false`, `transparent: true`).
#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    /// Which backend composites the CPU-rasterized frame to the window:
    /// GPU (`wgpu`, the default) or CPU-only (`softbuffer`). Can be
    /// force-overridden at launch with `CUI_OVERRIDE_RENDER_BACKEND=gpu|cpu`
    /// regardless of what's set here — see [`RenderBackend::resolve`].
    pub backend: RenderBackend,
}

impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions {
            title: "CreamUI".to_string(),
            width: 800,
            height: 600,
            resizable: true,
            decorations: true,
            transparent: false,
            backend: RenderBackend::default(),
        }
    }
}

/// Enables verbose logging when `CUI_DEBUG=1` is set in the environment,
/// without overriding an explicit `RUST_LOG`.
fn init_logging() {
    if std::env::var("CUI_DEBUG").as_deref() == Ok("1") && std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "creamui_render=debug,creamui_core=debug");
    }
    let _ = env_logger::try_init();
}

struct FrameState {
    painter: SkiaPainter,
    renderer: Renderer,
    scene: Option<Scene>,
}

/// Whichever backend is actually composing frames for a window, picked once
/// in `resumed` per [`WindowOptions::backend`] (as resolved by
/// [`RenderBackend::resolve`]).
enum Presenter {
    Gpu(GpuState),
    Cpu(CpuState),
}

impl Presenter {
    fn present(&mut self, rgba: &[u8], width: u32, height: u32) {
        match self {
            Presenter::Gpu(gpu) => gpu.present(rgba, width, height),
            Presenter::Cpu(cpu) => cpu.present(rgba, width, height),
        }
    }
}

type SharedWindow = Rc<RefCell<Option<Arc<Window>>>>;

/// A handle to a live window, for desktop-shell operations (resize, move,
/// always-on-top) issued from outside the render loop — e.g. a click
/// handler. Cheap to clone; every clone shares the same underlying window.
///
/// Handed to a window's `on_window_ready` callback once that window has
/// actually been created (winit windows don't exist until the event loop
/// resumes, so this can't be available any earlier). All methods are no-ops
/// if called after the window has closed.
#[derive(Clone)]
pub struct WindowHandle(SharedWindow);

impl WindowHandle {
    /// Requests a new logical-pixel window size. The actual resize (and any
    /// resulting `Resized` event) happens asynchronously, same as a user
    /// dragging the window border.
    pub fn resize(&self, width: u32, height: u32) {
        if let Some(window) = self.0.borrow().as_ref() {
            let _ = window.request_inner_size(winit::dpi::LogicalSize::new(width, height));
        }
    }

    /// Moves the window's top-left corner to a logical-pixel screen position.
    pub fn set_position(&self, x: i32, y: i32) {
        if let Some(window) = self.0.borrow().as_ref() {
            window.set_outer_position(winit::dpi::LogicalPosition::new(x, y));
        }
    }

    /// Pins (or unpins) the window above all others — the standard
    /// desktop-shell/widget-overlay behavior.
    pub fn set_always_on_top(&self, enabled: bool) {
        if let Some(window) = self.0.borrow().as_ref() {
            window.set_window_level(if enabled {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            });
        }
    }
}

/// One window's worth of setup, queued via [`AppBuilder::window`] and opened
/// once [`AppBuilder::run`] starts the shared event loop.
struct WindowSpec {
    options: WindowOptions,
    on_window_ready: Box<dyn Fn(WindowHandle)>,
    repaint: Rc<dyn Fn()>,
    repaint_scene: Rc<dyn Fn()>,
    repaint_light: Rc<dyn Fn()>,
    render: Rc<dyn Fn()>,
    dirty: Rc<Cell<bool>>,
    scene_dirty: Rc<Cell<bool>>,
    _effect: Effect,
    viewport: Signal<Size>,
    scale_factor: Signal<f64>,
    frame: Rc<RefCell<FrameState>>,
    shared_window: SharedWindow,
    focused: Rc<Cell<Option<usize>>>,
    caret_visible: Rc<Cell<bool>>,
}

/// Builds and runs one or more CreamUI windows sharing a single process and
/// event loop.
///
/// Each window keeps entirely separate reactive/paint state — a signal
/// change in one window's UI only ever rebuilds and repaints that window.
/// The main cost this amortizes across windows is the *fixed* per-process
/// overhead a naive one-process-per-window design would otherwise multiply:
/// the Rust runtime, the embedded font and its glyph atlas, and — for any
/// window using [`RenderBackend::Gpu`] — a single shared `wgpu::Instance`
/// (GPU driver init is normally the single biggest contributor to a
/// CreamUI process's memory footprint; see [`RenderBackend::Cpu`] to avoid
/// it altogether).
///
/// ```no_run
/// # use creamui_render::{AppBuilder, WindowOptions};
/// # use creamui_theme::Color;
/// # use creamui_core::{BoxedWidget, Size};
/// # fn build(_: Size) -> BoxedWidget { unimplemented!() }
/// AppBuilder::new()
///     .window(WindowOptions::default(), Color::rgb(0, 0, 0), |_handle| {}, build)
///     .window(WindowOptions::default(), Color::rgb(0, 0, 0), |_handle| {}, build)
///     .run();
/// ```
#[derive(Default)]
pub struct AppBuilder {
    specs: Vec<PendingWindow>,
}

/// Everything [`AppBuilder::window`] needs to defer construction to
/// [`AppBuilder::run`], where all windows' backends are resolved together
/// (so a single shared `wgpu::Instance` can be started once, up front, if
/// any of them need it).
struct PendingWindow {
    options: WindowOptions,
    clear_color: Color,
    on_window_ready: Box<dyn Fn(WindowHandle)>,
    build_ui: Box<dyn Fn(Size) -> BoxedWidget>,
}

impl AppBuilder {
    pub fn new() -> Self {
        AppBuilder { specs: Vec::new() }
    }

    /// Queues a window to be opened when [`run`](AppBuilder::run) starts the
    /// shared event loop. See [`crate::run`] for what each argument does.
    pub fn window(
        mut self,
        options: WindowOptions,
        clear_color: Color,
        on_window_ready: impl Fn(WindowHandle) + 'static,
        build_ui: impl Fn(Size) -> BoxedWidget + 'static,
    ) -> Self {
        self.specs.push(PendingWindow {
            options,
            clear_color,
            on_window_ready: Box::new(on_window_ready),
            build_ui: Box::new(build_ui),
        });
        self
    }

    /// Opens every queued window and runs one shared event loop until all of
    /// them have closed.
    pub fn run(self) {
        run_windows(self.specs);
    }
}

/// Per-window state for a window that has actually been created (its winit
/// [`Window`] exists and its presenter is ready). Lives in [`AppHandler`],
/// keyed by [`WindowId`], from the moment `resumed` creates it until
/// `CloseRequested` removes it.
struct WindowState {
    viewport: Signal<Size>,
    scale_factor: Signal<f64>,
    frame: Rc<RefCell<FrameState>>,
    window: Arc<Window>,
    presenter: Presenter,
    pointer_pos: Point,
    modifiers: ModifiersState,
    /// Index into the current `Scene`'s focusables, if any widget has
    /// keyboard focus. Only stable while the widget tree's shape doesn't
    /// change — see `Scene`'s doc comment. Shared with `repaint` (below) so
    /// each repaint knows which widget, if any, to paint a focus overlay
    /// (e.g. a text input's caret) onto.
    focused: Rc<Cell<Option<usize>>>,
    /// The text-input caret's current blink phase, shared with `repaint`
    /// the same way as `focused`.
    caret_visible: Rc<Cell<bool>>,
    /// When the caret should next toggle visibility (see `about_to_wait`).
    next_blink: Instant,
    next_animation: Instant,
    /// The system cursor icon last set on the window, so `CursorMoved`
    /// only calls into the backend when it actually changes.
    current_cursor: CursorIcon,
    /// Callback for the widget currently under the pointer. Keeping the
    /// callback rather than a scene index makes it safe across re-renders.
    hovered: Option<(creamui_core::Rect, Rc<dyn Fn(bool)>)>,
    /// Index into the current `Scene`'s draggables while the left mouse
    /// button is held down over one, `None` otherwise.
    dragging: Option<usize>,
    /// Invalidates this window. Signal changes and caret/focus updates are
    /// coalesced until the next `RedrawRequested` frame.
    repaint: Rc<dyn Fn()>,
    repaint_scene: Rc<dyn Fn()>,
    /// Paint-only refresh (no rebuild, no layout) for hover/press/focus-only
    /// changes — cheap enough to call from every `CursorMoved`.
    repaint_light: Rc<dyn Fn()>,
    /// Executes the deferred build/layout/paint pass. Signal writes only
    /// schedule this; `RedrawRequested` performs it once per compositor
    /// frame.
    render: Rc<dyn Fn()>,
    dirty: Rc<Cell<bool>>,
    scene_dirty: Rc<Cell<bool>>,
    _effect: Effect,
    t_run: Instant,
    first_present_logged: bool,
}

impl WindowState {
    /// Sets `viewport` (logical pixels) from the window's current physical
    /// inner size and `scale_factor`.
    fn sync_viewport_from_window(&self) {
        let physical = self.window.inner_size();
        let scale = self.scale_factor.peek();
        set_if_changed(
            &self.viewport,
            Size {
                width: (physical.width as f64 / scale) as f32,
                height: (physical.height as f64 / scale) as f32,
            },
        );
    }

    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(new_size) => {
                if new_size.width == 0 || new_size.height == 0 {
                    return;
                }
                let scale = self.scale_factor.peek();
                set_if_changed(
                    &self.viewport,
                    Size {
                        width: (new_size.width as f64 / scale) as f32,
                        height: (new_size.height as f64 / scale) as f32,
                    },
                );
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                log::debug!("creamui-render: scale factor changed to {scale_factor}");
                set_if_changed(&self.scale_factor, scale_factor);
                self.sync_viewport_from_window();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.scale_factor.peek();
                self.pointer_pos = Point {
                    x: (position.x / scale) as f32,
                    y: (position.y / scale) as f32,
                };
                self.frame.borrow_mut().painter.pointer = Some(self.pointer_pos);
                (self.repaint_light)();

                let hovered_cursor = {
                    let frame = self.frame.borrow();
                    frame
                        .scene
                        .as_ref()
                        .and_then(|scene| scene.cursor_hit_test(self.pointer_pos))
                        .unwrap_or(CursorIcon::Default)
                };
                if hovered_cursor != self.current_cursor {
                    self.current_cursor = hovered_cursor;
                    self.window
                        .set_cursor(translate_cursor_icon(hovered_cursor));
                }

                let next_hover = self
                    .frame
                    .borrow()
                    .scene
                    .as_ref()
                    .and_then(|scene| scene.hover_hit_test(self.pointer_pos));
                // Widget descriptions are recreated on each reactive frame,
                // so callback `Rc`s are not stable. The visible rect is: it
                // prevents a stationary pointer from producing leave/enter
                // churn after an unrelated redraw.
                let unchanged = matches!(
                    (&self.hovered, &next_hover),
                    (Some((current_rect, _)), Some((next_rect, _))) if current_rect == next_rect
                );
                if !unchanged {
                    if let Some((_, current)) = self.hovered.take() {
                        current(false);
                    }
                    if let Some((rect, next)) = next_hover {
                        next(true);
                        self.hovered = Some((rect, next));
                    }
                }

                if let Some(index) = self.dragging {
                    let frame = self.frame.borrow();
                    if let Some(scene) = frame.scene.as_ref() {
                        if let Some((rect, handler)) = scene.draggable_at(index) {
                            let local = Point {
                                x: self.pointer_pos.x - rect.x,
                                y: self.pointer_pos.y - rect.y,
                            };
                            let handler = handler.clone();
                            drop(frame);
                            handler(local, rect);
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.frame.borrow_mut().painter.press_origin = Some(self.pointer_pos);
                (self.repaint_light)();
                let frame = self.frame.borrow();
                let Some(scene) = frame.scene.as_ref() else {
                    return;
                };

                let click_handler = scene.hit_test(self.pointer_pos).cloned();
                let new_focus = scene.focus_hit_test(self.pointer_pos);
                let focus_changed = new_focus != self.focused.get();
                self.focused.set(new_focus);
                let drag_start = scene.drag_hit_test(self.pointer_pos).and_then(|index| {
                    scene
                        .draggable_at(index)
                        .map(|(rect, handler)| (index, rect, handler.clone()))
                });
                let drag_anchor = scene.drag_start_at(self.pointer_pos);
                drop(frame);

                if focus_changed {
                    // Reset the blink phase so the caret appears solid the
                    // instant a text input gains focus, rather than
                    // possibly landing mid-blink.
                    self.caret_visible.set(true);
                    self.next_blink = Instant::now() + CARET_BLINK_INTERVAL;
                    (self.repaint_light)();
                }

                if let Some((index, rect, handler)) = drag_start {
                    self.dragging = Some(index);
                    let local = Point {
                        x: self.pointer_pos.x - rect.x,
                        y: self.pointer_pos.y - rect.y,
                    };
                    handler(local, rect);
                }
                if let Some((rect, handler)) = drag_anchor {
                    handler(
                        Point {
                            x: self.pointer_pos.x - rect.x,
                            y: self.pointer_pos.y - rect.y,
                        },
                        rect,
                    );
                }
                if let Some(handler) = click_handler {
                    log::debug!("creamui-render: click hit at {:?}", self.pointer_pos);
                    handler();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = None;
                self.frame.borrow_mut().painter.press_origin = None;
                (self.repaint_light)();
            }
            WindowEvent::CursorLeft { .. } => {
                self.frame.borrow_mut().painter.pointer = None;
                if let Some((_, callback)) = self.hovered.take() {
                    callback(false);
                }
                (self.repaint_light)();
            }
            WindowEvent::Focused(false) => {
                self.frame.borrow_mut().painter.press_origin = None;
                self.dragging = None;
                (self.repaint_light)();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scale = self.scale_factor.peek();
                // Convention: positive `delta_y` reveals content further
                // down (increases a scroll view's offset), matching
                // "natural" wheel-down scrolling.
                let delta_y: f32 = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => -y * 40.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => -(pos.y / scale) as f32,
                };

                let frame = self.frame.borrow();
                let Some(scene) = frame.scene.as_ref() else {
                    return;
                };
                let handler = scene.scroll_hit_test(self.pointer_pos).map(|index| {
                    (
                        scene.on_scroll_at(index).cloned(),
                        scene.scroll_is_local_at(index),
                    )
                });
                drop(frame);

                if let Some((Some(handler), local)) = handler {
                    handler(delta_y);
                    if local {
                        (self.repaint_scene)();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                let Some(key) = translate_key(&event.logical_key) else {
                    return;
                };
                if key == Key::Tab {
                    let next = self.frame.borrow().scene.as_ref().and_then(|scene| {
                        scene.next_focus(self.focused.get(), self.modifiers.shift_key())
                    });
                    self.focused.set(next);
                    (self.repaint_light)();
                    return;
                }
                let Some(index) = self.focused.get() else {
                    return;
                };

                let handler = self
                    .frame
                    .borrow()
                    .scene
                    .as_ref()
                    .and_then(|scene| scene.on_key_at(index).cloned());
                if let Some(handler) = handler {
                    // Keep the caret solid through the keystroke rather than
                    // possibly toggling off right as the text changes.
                    self.caret_visible.set(true);
                    self.next_blink = Instant::now() + CARET_BLINK_INTERVAL;
                    handler(KeyInput {
                        key,
                        modifiers: Modifiers {
                            ctrl: self.modifiers.control_key(),
                            shift: self.modifiers.shift_key(),
                        },
                    });
                    // Rebuild now, not on the next debounced redraw: a
                    // queued second keystroke would otherwise still see
                    // `handler`'s stale pre-edit snapshot.
                    (self.render)();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::RedrawRequested => {
                if self.dirty.get() {
                    (self.render)();
                    // `render` already repaints the scene, so a pending
                    // `scene_dirty` from earlier in the same event is moot.
                    self.scene_dirty.set(false);
                } else if self.scene_dirty.replace(false) {
                    (self.repaint_scene)();
                }
                let frame = self.frame.borrow();
                let pixmap = &frame.painter.pixmap;
                self.presenter
                    .present(pixmap.data(), pixmap.width(), pixmap.height());
                if !self.first_present_logged {
                    self.first_present_logged = true;
                    log::debug!(
                        "creamui-render: first present done: {:?}",
                        self.t_run.elapsed()
                    );
                }
            }
            _ => {}
        }
    }
}

/// The shared [`ApplicationHandler`] driving every window opened by
/// [`AppBuilder`] (and, for a single window, [`run`]) from one event loop.
struct AppHandler {
    /// Drained the first time `resumed` runs: creates each window's winit
    /// `Window` and presenter, then moves it into `windows`. `resumed` can
    /// in principle be called again later (e.g. mobile lifecycle), at which
    /// point this is already empty and a no-op.
    pending: Vec<WindowSpec>,
    windows: HashMap<WindowId, WindowState>,
    /// Shared by every window using [`RenderBackend::Gpu`] — one
    /// `wgpu::Instance` regardless of how many GPU windows are open, since
    /// its ~100-200ms Windows loader/ICD cost and driver memory footprint
    /// are the whole reason multi-window-in-one-process is worth doing.
    /// `None` if no queued window resolved to the GPU backend.
    gpu_instance: Option<Rc<wgpu::Instance>>,
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        for spec in self.pending.drain(..) {
            let t0 = Instant::now();
            let attrs = WindowAttributes::default()
                .with_title(spec.options.title.clone())
                .with_inner_size(winit::dpi::LogicalSize::new(
                    spec.options.width,
                    spec.options.height,
                ))
                .with_resizable(spec.options.resizable)
                .with_decorations(spec.options.decorations)
                .with_transparent(spec.options.transparent);

            let window = Arc::new(
                event_loop
                    .create_window(attrs)
                    .expect("failed to create window"),
            );
            // Show the window the instant it exists rather than waiting for
            // GPU init (adapter/device/pipeline — several hundred ms on
            // Windows) to finish. That init cost doesn't go away, but the
            // window appearing immediately is what "the app feels slow to
            // launch" is actually about; the OS-default surface briefly
            // shown underneath gets replaced by the real first frame a
            // moment later.
            window.set_visible(true);
            log::debug!(
                "creamui-render: window created and shown: {:?}",
                t0.elapsed()
            );
            log::debug!(
                "creamui-render: window created ({}x{} logical, scale factor {})",
                spec.options.width,
                spec.options.height,
                window.scale_factor()
            );

            spec.scale_factor.set(window.scale_factor());
            {
                let physical = window.inner_size();
                let scale = spec.scale_factor.peek();
                spec.viewport.set(Size {
                    width: (physical.width as f64 / scale) as f32,
                    height: (physical.height as f64 / scale) as f32,
                });
            }

            let mut presenter = match spec.options.backend {
                RenderBackend::Gpu => {
                    let instance = self
                        .gpu_instance
                        .as_ref()
                        .expect("a window resolved to RenderBackend::Gpu but no shared wgpu::Instance was created");
                    Presenter::Gpu(GpuState::new(window.clone(), instance))
                }
                RenderBackend::Cpu => Presenter::Cpu(CpuState::new(window.clone())),
            };
            log::debug!(
                "creamui-render: {:?} presenter ready: {:?}",
                spec.options.backend,
                t0.elapsed()
            );

            // Present the already-painted first frame (built by the initial
            // `create_effect` run in `run_windows`, before this window
            // existed).
            {
                let frame = spec.frame.borrow();
                let pixmap = &frame.painter.pixmap;
                presenter.present(pixmap.data(), pixmap.width(), pixmap.height());
            }
            log::debug!("creamui-render: first frame presented: {:?}", t0.elapsed());

            *spec.shared_window.borrow_mut() = Some(window.clone());
            (spec.on_window_ready)(WindowHandle(spec.shared_window.clone()));

            let window_id = window.id();
            self.windows.insert(
                window_id,
                WindowState {
                    viewport: spec.viewport,
                    scale_factor: spec.scale_factor,
                    frame: spec.frame,
                    window,
                    presenter,
                    pointer_pos: Point::default(),
                    modifiers: ModifiersState::default(),
                    focused: spec.focused,
                    caret_visible: spec.caret_visible,
                    next_blink: Instant::now() + CARET_BLINK_INTERVAL,
                    next_animation: Instant::now(),
                    current_cursor: CursorIcon::Default,
                    hovered: None,
                    dragging: None,
                    repaint: spec.repaint,
                    repaint_scene: spec.repaint_scene,
                    repaint_light: spec.repaint_light,
                    render: spec.render,
                    dirty: spec.dirty,
                    scene_dirty: spec.scene_dirty,
                    _effect: spec._effect,
                    t_run: t0,
                    first_present_logged: false,
                },
            );
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            log::debug!("creamui-render: close requested for window {window_id:?}");
            self.windows.remove(&window_id);
            if self.windows.is_empty() {
                event_loop.exit();
            }
            return;
        }

        if let Some(state) = self.windows.get_mut(&window_id) {
            state.handle_window_event(event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let mut next_wake: Option<Instant> = None;
        for state in self.windows.values_mut() {
            if state.frame.borrow().painter.animated {
                if now >= state.next_animation {
                    state.next_animation = now + Duration::from_millis(32);
                    (state.repaint)();
                }
                next_wake =
                    Some(next_wake.map_or(state.next_animation, |t| t.min(state.next_animation)));
            }
            if state.focused.get().is_none() {
                continue;
            }
            if now >= state.next_blink {
                state.caret_visible.set(!state.caret_visible.get());
                state.next_blink = now + CARET_BLINK_INTERVAL;
                (state.repaint)();
            }
            next_wake = Some(next_wake.map_or(state.next_blink, |t| t.min(state.next_blink)));
        }
        event_loop.set_control_flow(match next_wake {
            Some(t) => ControlFlow::WaitUntil(t),
            None => ControlFlow::Wait,
        });
    }
}

/// Opens a window and runs the reactive render loop until it is closed.
///
/// `build_ui` is called once up front and again whenever a signal it reads
/// changes; it must construct a fresh widget tree covering `viewport` each
/// time (widgets are cheap, immutable descriptions — see
/// `creamui_core::Widget`). `clear_color` is the color the window is wiped
/// to before `build_ui`'s tree is painted. `on_window_ready` is called once,
/// as soon as the window exists, with a [`WindowHandle`] for issuing
/// window-level operations (resize, move, always-on-top) later — e.g. from
/// a click handler.
///
/// To open several windows sharing one process and event loop (e.g. a
/// desktop-shell dock), use [`AppBuilder`] instead.
pub fn run(
    options: WindowOptions,
    clear_color: Color,
    on_window_ready: impl Fn(WindowHandle) + 'static,
    build_ui: impl Fn(Size) -> BoxedWidget + 'static,
) {
    AppBuilder::new()
        .window(options, clear_color, on_window_ready, build_ui)
        .run();
}

fn run_windows(specs: Vec<PendingWindow>) {
    assert!(
        !specs.is_empty(),
        "creamui-render: AppBuilder::run() called with no windows queued"
    );

    init_logging();
    let t_run = Instant::now();
    log::debug!("creamui-render: run() start with {} window(s)", specs.len());

    // Each window's requested backend can be force-overridden at launch via
    // `CUI_OVERRIDE_RENDER_BACKEND` — resolve up front, once per window, so
    // every later decision (whether to pay GPU init cost at all, which
    // presenter `resumed` builds for that window) uses the same value.
    let mut specs = specs;
    for spec in &mut specs {
        spec.options.backend = RenderBackend::resolve(spec.options.backend);
    }
    let any_gpu = specs
        .iter()
        .any(|s| matches!(s.options.backend, RenderBackend::Gpu));

    // `wgpu::Instance::new` doesn't depend on any window and costs
    // ~100-200ms on Windows (Vulkan/DX12 loader + ICD enumeration) — kick it
    // off now so it overlaps with the initial UI builds below instead of
    // sitting on `resumed`'s critical path. One instance is shared by every
    // GPU-backend window; skipped entirely if none of them need it.
    let gpu_instance_handle = any_gpu.then(|| std::thread::spawn(GpuState::create_instance));

    let dump_frame_path = std::env::var("CUI_DUMP_FRAME").ok();
    let multiple_windows = specs.len() > 1;

    let pending: Vec<WindowSpec> = specs
        .into_iter()
        .enumerate()
        .map(|(index, spec)| {
            build_window_spec(index, spec, dump_frame_path.as_deref(), multiple_windows)
        })
        .collect();

    log::debug!(
        "creamui-render: before EventLoop::new: {:?}",
        t_run.elapsed()
    );
    let event_loop = EventLoop::new().expect("failed to create event loop");
    log::debug!("creamui-render: event loop created: {:?}", t_run.elapsed());
    event_loop.set_control_flow(ControlFlow::Wait);

    let gpu_instance = gpu_instance_handle.map(|handle| {
        let instance = handle
            .join()
            .expect("gpu instance creation thread panicked");
        log::debug!("creamui-render: gpu instance ready: {:?}", t_run.elapsed());
        Rc::new(instance)
    });

    let mut handler = AppHandler {
        pending,
        windows: HashMap::new(),
        gpu_instance,
    };
    event_loop
        .run_app(&mut handler)
        .expect("event loop exited with an error");
}

/// Builds one window's pre-creation state (signals, frame buffer, reactive
/// effect) — everything that doesn't depend on the winit `Window` actually
/// existing yet. `resumed` finishes the job once the event loop starts.
fn build_window_spec(
    index: usize,
    spec: PendingWindow,
    dump_frame_path: Option<&str>,
    multiple_windows: bool,
) -> WindowSpec {
    let PendingWindow {
        options,
        clear_color,
        on_window_ready,
        build_ui,
    } = spec;

    let viewport = Signal::new(Size {
        width: options.width as f32,
        height: options.height as f32,
    });
    let scale_factor = Signal::new(1.0f64);
    let frame = Rc::new(RefCell::new(FrameState {
        painter: SkiaPainter::new(options.width, options.height),
        renderer: Renderer::new(),
        scene: None,
    }));
    let shared_window: SharedWindow = Rc::new(RefCell::new(None));
    let build_ui: Rc<dyn Fn(Size) -> BoxedWidget> = Rc::from(build_ui);
    let focused: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
    let caret_visible: Rc<Cell<bool>> = Rc::new(Cell::new(true));

    // With multiple windows sharing one `CUI_DUMP_FRAME` path, suffix each
    // window's dump with its index rather than having every window's
    // repaint clobber the same file.
    let dump_frame_path: Option<String> = dump_frame_path.map(|path| {
        if multiple_windows {
            format!("{path}.{index}")
        } else {
            path.to_string()
        }
    });

    let dirty = Rc::new(Cell::new(false));
    // Tree built eagerly by `repaint`, consumed by the next `render`.
    let pending_root: Rc<RefCell<Option<BoxedWidget>>> = Rc::new(RefCell::new(None));

    // The expensive half of a frame. It is deliberately separate from
    // `repaint` below: pointer input may invalidate a UI dozens of times
    // before the compositor is ready for its next frame.
    let render: Rc<dyn Fn()> = Rc::new({
        let viewport = viewport.clone();
        let scale_factor = scale_factor.clone();
        let frame = frame.clone();
        let window = shared_window.clone();
        let build_ui = build_ui.clone();
        let focused = focused.clone();
        let caret_visible = caret_visible.clone();
        let dirty = dirty.clone();
        let pending_root = pending_root.clone();
        move || {
            dirty.set(false);
            // Widgets are laid out in logical pixels; the painter (and the
            // presenter it feeds) is sized in physical pixels so HiDPI
            // displays stay crisp — see `SkiaPainter`'s doc comment.
            let logical_size = viewport.peek();
            let scale = scale_factor.peek();
            // `repaint` usually already built this; fall back for
            // non-signal-driven redraws (animation ticks, caret blink).
            let root = pending_root
                .borrow_mut()
                .take()
                .unwrap_or_else(|| build_ui(logical_size));

            let mut frame = frame.borrow_mut();
            let FrameState {
                painter, renderer, ..
            } = &mut *frame;
            let physical_width = (logical_size.width as f64 * scale).round() as u32;
            let physical_height = (logical_size.height as f64 * scale).round() as u32;
            painter.set_scale(scale as f32);
            painter.resize(physical_width, physical_height);
            painter.clear(clear_color);
            let scene = renderer.render_focused(
                root,
                logical_size,
                painter,
                focused.get(),
                caret_visible.get(),
            );
            frame.scene = Some(scene);

            // Debug aid: dump each painted frame to a PNG on disk, e.g. for
            // headless verification where no on-screen compositor is available.
            if let Some(path) = &dump_frame_path {
                if let Err(err) = frame.painter.pixmap.save_png(path) {
                    log::warn!("creamui-render: failed to write CUI_DUMP_FRAME to {path}: {err}");
                }
            }
            drop(frame);

            if let Some(window) = window.borrow().as_ref() {
                window.request_redraw();
            }
        }
    });

    let repaint_scene: Rc<dyn Fn()> = Rc::new({
        let viewport = viewport.clone();
        let scale_factor = scale_factor.clone();
        let frame = frame.clone();
        let window = shared_window.clone();
        let focused = focused.clone();
        let caret_visible = caret_visible.clone();
        move || {
            let logical_size = viewport.peek();
            let scale = scale_factor.peek();
            let mut frame = frame.borrow_mut();
            let physical_width = (logical_size.width as f64 * scale).round() as u32;
            let physical_height = (logical_size.height as f64 * scale).round() as u32;
            frame.painter.set_scale(scale as f32);
            frame.painter.resize(physical_width, physical_height);
            frame.painter.clear(clear_color);
            let FrameState {
                painter, renderer, ..
            } = &mut *frame;
            if let Some(scene) =
                renderer.repaint_focused(painter, focused.get(), caret_visible.get())
            {
                frame.scene = Some(scene);
            }
            drop(frame);
            if let Some(window) = window.borrow().as_ref() {
                window.request_redraw();
            }
        }
    });

    // `create_effect` wraps this, so it must call `build_ui` itself, right
    // here, to stay subscribed to whatever `Signal`s the active branch
    // reads — a closure that only flips `dirty` for `render` to build later
    // reads no `Signal` and de-subscribes the effect from everything after
    // its first run. Layout/paint stay deferred through `dirty`.
    let repaint: Rc<dyn Fn()> = Rc::new({
        let viewport = viewport.clone();
        let build_ui = build_ui.clone();
        let pending_root = pending_root.clone();
        let render = render.clone();
        let window = shared_window.clone();
        let dirty = dirty.clone();
        move || {
            let logical_size = viewport.get();
            *pending_root.borrow_mut() = Some(build_ui(logical_size));
            // The first reactive run happens before winit has created the
            // window, so render immediately to provide its initial frame.
            // Afterwards merely mark dirty and let RedrawRequested coalesce
            // all input updates into one layout/paint pass.
            if let Some(window) = window.borrow().as_ref() {
                if !dirty.replace(true) {
                    window.request_redraw();
                }
            } else {
                render();
            }
        }
    });

    let scene_dirty = Rc::new(Cell::new(false));

    // Paint-only counterpart to `repaint`: no rebuild, no layout.
    let repaint_light: Rc<dyn Fn()> = Rc::new({
        let repaint_scene = repaint_scene.clone();
        let window = shared_window.clone();
        let dirty = dirty.clone();
        let scene_dirty = scene_dirty.clone();
        move || {
            if let Some(window) = window.borrow().as_ref() {
                if !dirty.get() && !scene_dirty.replace(true) {
                    window.request_redraw();
                }
            } else {
                repaint_scene();
            }
        }
    });

    let effect_repaint = repaint.clone();
    let effect = create_effect(move || effect_repaint());

    WindowSpec {
        options,
        on_window_ready,
        repaint,
        repaint_scene,
        repaint_light,
        render,
        dirty,
        scene_dirty,
        _effect: effect,
        viewport,
        scale_factor,
        frame,
        shared_window,
        focused,
        caret_visible,
    }
}
