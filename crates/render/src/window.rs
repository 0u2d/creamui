//! Window creation and the reactive render loop.
//!
//! [`run`] owns the winit event loop. On startup, and again every time a
//! [`creamui_reactive::Signal`] read while building the UI changes, it
//! rebuilds the widget tree, recomputes layout, repaints via
//! [`crate::painter::SkiaPainter`], and requests a redraw; the actual
//! `RedrawRequested` handler only uploads the already-painted buffer to the
//! GPU and presents it.

use crate::gpu::GpuState;
use crate::painter::SkiaPainter;
use creamui_core::{BoxedWidget, CursorIcon, Key, KeyInput, Point, Renderer, Scene, Size};
use creamui_reactive::{create_effect, Effect, Signal};
use creamui_theme::Color;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key as WinitKey, NamedKey};
use winit::window::{CursorIcon as WinitCursorIcon, Window, WindowAttributes, WindowId, WindowLevel};

/// How long the text-input caret stays in each visibility phase while
/// blinking (on, then off, then on again).
const CARET_BLINK_INTERVAL: Duration = Duration::from_millis(530);

fn translate_cursor_icon(icon: CursorIcon) -> WinitCursorIcon {
    match icon {
        CursorIcon::Default => WinitCursorIcon::Default,
        CursorIcon::Text => WinitCursorIcon::Text,
        CursorIcon::Pointer => WinitCursorIcon::Pointer,
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
        WinitKey::Named(NamedKey::Home) => Some(Key::Home),
        WinitKey::Named(NamedKey::End) => Some(Key::End),
        _ => None,
    }
}

/// Options for the window CreamUI opens, set once at startup.
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
        }
    }
}

/// Enables verbose logging when `CREAMUI_DEBUG=1` is set in the
/// environment, without overriding an explicit `RUST_LOG`.
fn init_logging() {
    if std::env::var("CREAMUI_DEBUG").as_deref() == Ok("1") && std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "creamui_render=debug,creamui_core=debug");
    }
    let _ = env_logger::try_init();
}

struct FrameState {
    painter: SkiaPainter,
    renderer: Renderer,
    scene: Option<Scene>,
}

type SharedWindow = Rc<RefCell<Option<Arc<Window>>>>;

/// A handle to the live window, for desktop-shell operations (resize, move,
/// always-on-top) issued from outside the render loop — e.g. a click
/// handler. Cheap to clone; every clone shares the same underlying window.
///
/// Handed to the `on_window_ready` callback passed to [`run`] once the
/// window has actually been created (winit windows don't exist until the
/// event loop resumes, so this can't be available any earlier). All methods
/// are no-ops if called after the window has closed.
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
            window.set_window_level(if enabled { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal });
        }
    }
}

struct AppHandler {
    options: WindowOptions,
    viewport: Signal<Size>,
    scale_factor: Signal<f64>,
    frame: Rc<RefCell<FrameState>>,
    shared_window: SharedWindow,
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    pointer_pos: Point,
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
    /// The system cursor icon last set on the window, so `CursorMoved`
    /// only calls into the backend when it actually changes.
    current_cursor: CursorIcon,
    /// Index into the current `Scene`'s draggables while the left mouse
    /// button is held down over one, `None` otherwise.
    dragging: Option<usize>,
    /// Rebuilds the UI, recomputes layout, and repaints — the same routine
    /// `_effect` runs on signal changes, exposed here so caret blinking and
    /// focus changes (which don't touch any `Signal`) can also trigger it.
    repaint: Rc<dyn Fn()>,
    _effect: Effect,
    /// Called once, the first time the window is created (see `resumed`).
    on_window_ready: Rc<dyn Fn(WindowHandle)>,
    window_ready_notified: bool,
    t_run: Instant,
    first_present_logged: bool,
}

impl AppHandler {
    /// Sets `viewport` (logical pixels) from the window's current physical
    /// inner size and `scale_factor`.
    fn sync_viewport_from_window(&self, window: &Window) {
        let physical = window.inner_size();
        let scale = self.scale_factor.peek();
        self.viewport.set(Size {
            width: (physical.width as f64 / scale) as f32,
            height: (physical.height as f64 / scale) as f32,
        });
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let t0 = std::time::Instant::now();
        let attrs = WindowAttributes::default()
            .with_title(self.options.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(self.options.width, self.options.height))
            .with_resizable(self.options.resizable)
            .with_decorations(self.options.decorations)
            .with_transparent(self.options.transparent)
            // Stay hidden until the first frame is painted and presented
            // below, so the OS never shows an empty/default-colored
            // surface before CreamUI's own content is on screen.
            .with_visible(false);

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );
        log::debug!("creamui-render: window created: {:?}", t0.elapsed());
        log::debug!(
            "creamui-render: window created ({}x{} logical, scale factor {})",
            self.options.width,
            self.options.height,
            window.scale_factor()
        );

        self.scale_factor.set(window.scale_factor());
        self.sync_viewport_from_window(&window);

        let mut gpu = GpuState::new(window.clone());
        log::debug!("creamui-render: gpu state ready: {:?}", t0.elapsed());

        // Present the already-painted first frame (built by the initial
        // `create_effect` run in `run`, before this window existed) while
        // still hidden, then reveal — so the window never shows a blank
        // frame before its real content.
        {
            let frame = self.frame.borrow();
            let pixmap = &frame.painter.pixmap;
            gpu.present(pixmap.data(), pixmap.width(), pixmap.height());
        }
        window.set_visible(true);
        log::debug!("creamui-render: window shown: {:?}", t0.elapsed());
        self.gpu = Some(gpu);

        *self.shared_window.borrow_mut() = Some(window.clone());
        self.window = Some(window.clone());

        if !self.window_ready_notified {
            self.window_ready_notified = true;
            (self.on_window_ready)(WindowHandle(self.shared_window.clone()));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::debug!("creamui-render: close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width == 0 || new_size.height == 0 {
                    return;
                }
                let scale = self.scale_factor.peek();
                self.viewport.set(Size {
                    width: (new_size.width as f64 / scale) as f32,
                    height: (new_size.height as f64 / scale) as f32,
                });
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                log::debug!("creamui-render: scale factor changed to {scale_factor}");
                self.scale_factor.set(scale_factor);
                if let Some(window) = self.window.clone() {
                    self.sync_viewport_from_window(&window);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.scale_factor.peek();
                self.pointer_pos = Point {
                    x: (position.x / scale) as f32,
                    y: (position.y / scale) as f32,
                };

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
                    if let Some(window) = self.window.as_ref() {
                        window.set_cursor(translate_cursor_icon(hovered_cursor));
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
                let frame = self.frame.borrow();
                let Some(scene) = frame.scene.as_ref() else { return };

                let click_handler = scene.hit_test(self.pointer_pos).cloned();
                let new_focus = scene.focus_hit_test(self.pointer_pos);
                let focus_changed = new_focus != self.focused.get();
                self.focused.set(new_focus);
                let drag_start = scene.drag_hit_test(self.pointer_pos).and_then(|index| {
                    scene
                        .draggable_at(index)
                        .map(|(rect, handler)| (index, rect, handler.clone()))
                });
                drop(frame);

                if focus_changed {
                    // Reset the blink phase so the caret appears solid the
                    // instant a text input gains focus, rather than
                    // possibly landing mid-blink.
                    self.caret_visible.set(true);
                    self.next_blink = Instant::now() + CARET_BLINK_INTERVAL;
                    (self.repaint)();
                }

                if let Some((index, rect, handler)) = drag_start {
                    self.dragging = Some(index);
                    let local = Point {
                        x: self.pointer_pos.x - rect.x,
                        y: self.pointer_pos.y - rect.y,
                    };
                    handler(local, rect);
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
                let Some(scene) = frame.scene.as_ref() else { return };
                let handler = scene
                    .scroll_hit_test(self.pointer_pos)
                    .and_then(|index| scene.on_scroll_at(index).cloned());
                drop(frame);

                if let Some(handler) = handler {
                    handler(delta_y);
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
                let Some(key) = translate_key(&event.logical_key) else { return };
                let Some(index) = self.focused.get() else { return };

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
                    handler(KeyInput { key });
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(gpu) = self.gpu.as_mut() else { return };
                let frame = self.frame.borrow();
                let pixmap = &frame.painter.pixmap;
                gpu.present(pixmap.data(), pixmap.width(), pixmap.height());
                if !self.first_present_logged {
                    self.first_present_logged = true;
                    log::debug!("creamui-render: first present done: {:?}", self.t_run.elapsed());
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.focused.get().is_some() {
            let now = Instant::now();
            if now >= self.next_blink {
                self.caret_visible.set(!self.caret_visible.get());
                self.next_blink = now + CARET_BLINK_INTERVAL;
                (self.repaint)();
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_blink));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
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
pub fn run(
    options: WindowOptions,
    clear_color: Color,
    on_window_ready: impl Fn(WindowHandle) + 'static,
    build_ui: impl Fn(Size) -> BoxedWidget + 'static,
) {
    init_logging();
    let t_run = std::time::Instant::now();
    log::debug!("creamui-render: run() start");

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
    let build_ui = Rc::new(build_ui);
    let focused: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
    let caret_visible: Rc<Cell<bool>> = Rc::new(Cell::new(true));

    let dump_frame_path = std::env::var("CREAMUI_DUMP_FRAME").ok();

    // Rebuilds the UI, recomputes layout, and repaints. Runs both reactively
    // (wrapped in `create_effect` below, whenever a `Signal` it reads
    // changes) and manually (on focus changes and caret blink ticks, which
    // touch `focused`/`caret_visible` — plain `Cell`s, not `Signal`s, so
    // they need an explicit nudge instead of the reactive system's).
    let repaint: Rc<dyn Fn()> = Rc::new({
        let viewport = viewport.clone();
        let scale_factor = scale_factor.clone();
        let frame = frame.clone();
        let window = shared_window.clone();
        let build_ui = build_ui.clone();
        let focused = focused.clone();
        let caret_visible = caret_visible.clone();
        let dump_frame_path = dump_frame_path.clone();
        move || {
            // Widgets are laid out in logical pixels; the painter (and the
            // GPU texture it feeds) is sized in physical pixels so HiDPI
            // displays stay crisp — see `SkiaPainter`'s doc comment.
            let logical_size = viewport.get();
            let scale = scale_factor.get();
            let root = build_ui(logical_size);

            let mut frame = frame.borrow_mut();
            let FrameState { painter, renderer, .. } = &mut *frame;
            let physical_width = (logical_size.width as f64 * scale).round() as u32;
            let physical_height = (logical_size.height as f64 * scale).round() as u32;
            painter.set_scale(scale as f32);
            painter.resize(physical_width, physical_height);
            painter.clear(clear_color);
            let scene = renderer.render_focused(root, logical_size, painter, focused.get(), caret_visible.get());
            frame.scene = Some(scene);

            // Debug aid: dump each painted frame to a PNG on disk, e.g. for
            // headless verification where no on-screen compositor is available.
            if let Some(path) = &dump_frame_path {
                if let Err(err) = frame.painter.pixmap.save_png(path) {
                    log::warn!("creamui-render: failed to write CREAMUI_DUMP_FRAME to {path}: {err}");
                }
            }
            drop(frame);

            if let Some(window) = window.borrow().as_ref() {
                window.request_redraw();
            }
        }
    });

    let effect_repaint = repaint.clone();
    let effect = create_effect(move || effect_repaint());

    log::debug!("creamui-render: before EventLoop::new: {:?}", t_run.elapsed());
    let event_loop = EventLoop::new().expect("failed to create event loop");
    log::debug!("creamui-render: event loop created: {:?}", t_run.elapsed());
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut handler = AppHandler {
        options,
        viewport,
        scale_factor,
        frame,
        shared_window,
        window: None,
        gpu: None,
        pointer_pos: Point::default(),
        focused,
        caret_visible,
        next_blink: Instant::now() + CARET_BLINK_INTERVAL,
        current_cursor: CursorIcon::Default,
        dragging: None,
        repaint,
        _effect: effect,
        on_window_ready: Rc::new(on_window_ready),
        window_ready_notified: false,
        t_run,
        first_present_logged: false,
    };
    event_loop.run_app(&mut handler).expect("event loop exited with an error");
}
