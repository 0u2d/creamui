//! wgpu + winit windowing and rendering backend for CreamUI.
//!
//! Rasterization happens on the CPU via `tiny-skia` ([`painter::SkiaPainter`]);
//! the GPU ([`gpu`]) only uploads and composites the result. This keeps the
//! MVP's rendering code simple while still presenting through the GPU.

mod font;
mod gpu;
mod painter;
mod window;

pub use painter::SkiaPainter;
pub use window::{run, WindowHandle, WindowOptions};
