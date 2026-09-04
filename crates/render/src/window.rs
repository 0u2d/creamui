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
use creamui_core::{render_frame, BoxedWidget, Point, Scene, Size};
use creamui_reactive::{create_effect, Effect, Signal};
use creamui_theme::Color;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

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
    scene: Option<Scene>,
}

type SharedWindow = Rc<RefCell<Option<Arc<Window>>>>;

struct AppHandler {
    options: WindowOptions,
    viewport: Signal<Size>,
    frame: Rc<RefCell<FrameState>>,
    shared_window: SharedWindow,
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    pointer_pos: Point,
    _effect: Effect,
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title(self.options.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(self.options.width, self.options.height))
            .with_resizable(self.options.resizable)
            .with_decorations(self.options.decorations)
            .with_transparent(self.options.transparent);

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );
        log::debug!("creamui-render: window created ({}x{})", self.options.width, self.options.height);

        self.gpu = Some(GpuState::new(window.clone()));
        *self.shared_window.borrow_mut() = Some(window.clone());
        self.window = Some(window.clone());
        window.request_redraw();
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
                self.viewport.set(Size {
                    width: new_size.width as f32,
                    height: new_size.height as f32,
                });
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_pos = Point {
                    x: position.x as f32,
                    y: position.y as f32,
                };
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let handler = self
                    .frame
                    .borrow()
                    .scene
                    .as_ref()
                    .and_then(|scene| scene.hit_test(self.pointer_pos).cloned());
                if let Some(handler) = handler {
                    log::debug!("creamui-render: click hit at {:?}", self.pointer_pos);
                    handler();
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(gpu) = self.gpu.as_mut() else { return };
                let frame = self.frame.borrow();
                let pixmap = &frame.painter.pixmap;
                gpu.present(pixmap.data(), pixmap.width(), pixmap.height());
            }
            _ => {}
        }
    }
}

/// Opens a window and runs the reactive render loop until it is closed.
///
/// `build_ui` is called once up front and again whenever a signal it reads
/// changes; it must construct a fresh widget tree covering `viewport` each
/// time (widgets are cheap, immutable descriptions — see
/// `creamui_core::Widget`). `clear_color` is the color the window is wiped
/// to before `build_ui`'s tree is painted.
pub fn run(options: WindowOptions, clear_color: Color, build_ui: impl Fn(Size) -> BoxedWidget + 'static) {
    init_logging();

    let viewport = Signal::new(Size {
        width: options.width as f32,
        height: options.height as f32,
    });
    let frame = Rc::new(RefCell::new(FrameState {
        painter: SkiaPainter::new(options.width, options.height),
        scene: None,
    }));
    let shared_window: SharedWindow = Rc::new(RefCell::new(None));

    let dump_frame_path = std::env::var("CREAMUI_DUMP_FRAME").ok();

    let effect_viewport = viewport.clone();
    let effect_frame = frame.clone();
    let effect_window = shared_window.clone();
    let effect = create_effect(move || {
        let size = effect_viewport.get();
        let root = build_ui(size);

        let mut frame = effect_frame.borrow_mut();
        frame.painter.resize(size.width as u32, size.height as u32);
        frame.painter.clear(clear_color);
        let scene = render_frame(root, size, &mut frame.painter);
        frame.scene = Some(scene);

        // Debug aid: dump each painted frame to a PNG on disk, e.g. for
        // headless verification where no on-screen compositor is available.
        if let Some(path) = &dump_frame_path {
            if let Err(err) = frame.painter.pixmap.save_png(path) {
                log::warn!("creamui-render: failed to write CREAMUI_DUMP_FRAME to {path}: {err}");
            }
        }
        drop(frame);

        if let Some(window) = effect_window.borrow().as_ref() {
            window.request_redraw();
        }
    });

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut handler = AppHandler {
        options,
        viewport,
        frame,
        shared_window,
        window: None,
        gpu: None,
        pointer_pos: Point::default(),
        _effect: effect,
    };
    event_loop.run_app(&mut handler).expect("event loop exited with an error");
}
