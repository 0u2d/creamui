//! Re-export of the showcase so it can also be embedded by the web demo.

#[path = "main.rs"]
mod app;

pub use app::launch;
