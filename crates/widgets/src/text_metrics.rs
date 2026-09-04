//! Approximate text sizing used to give [`crate::raw::RawText`] leaves a
//! real content size in the layout tree.
//!
//! `taffy` has no built-in notion of text, so leaves with no explicit size
//! default to zero and collapse in their parent's flex/grid layout. This
//! module loads the same system font family probed by
//! `creamui-render`'s renderer (see its `font` module) purely to measure
//! glyph advances, without rasterizing anything — the actual rendering
//! still happens in the backend.

use fontdue::layout::{CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle};
use fontdue::Font;
use std::sync::OnceLock;

const CANDIDATE_PATHS: &[&str] = &[
    "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
    "/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf",
    "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];

fn font() -> Option<&'static Font> {
    static FONT: OnceLock<Option<Font>> = OnceLock::new();
    FONT.get_or_init(|| {
        for path in CANDIDATE_PATHS {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(font) = Font::from_bytes(bytes, fontdue::FontSettings::default()) {
                    return Some(font);
                }
            }
        }
        None
    })
    .as_ref()
}

/// Returns an estimated `(width, height)` in logical pixels for `text` set
/// at `font_size`, single-line. Falls back to a fixed character-width
/// heuristic if no system font could be found.
pub fn measure(text: &str, font_size: f32) -> (f32, f32) {
    let Some(font) = font() else {
        return (text.chars().count() as f32 * font_size * 0.6, font_size * 1.4);
    };

    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        horizontal_align: HorizontalAlign::Left,
        ..LayoutSettings::default()
    });
    layout.append(&[font], &TextStyle::new(text, font_size, 0));

    let width = layout
        .glyphs()
        .iter()
        .map(|g| g.x + g.width as f32)
        .fold(0.0_f32, f32::max);
    let height = font_size * 1.4;
    // Small safety margin: this measures the tight rasterized glyph bounding
    // box, but the renderer's word-wrap decision is based on cumulative
    // advance widths, which can be a hair wider — without this, text can
    // wrap one word early at paint time even though it measured as fitting.
    (width.max(1.0) + font_size * 0.15, height)
}
