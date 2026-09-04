//! Text measurement backing [`crate::raw::RawText`]'s `taffy` measure
//! function (see `creamui_core::Widget::measure`).
//!
//! This embeds the exact same bundled font file as `creamui-render`'s
//! renderer (see its `font` module) purely to measure glyph layout, without
//! rasterizing anything — the actual rendering still happens in the
//! backend. Since both crates embed identical bytes and run the same
//! `fontdue` version, a size computed here matches what the renderer will
//! actually lay out at paint time.

use fontdue::layout::TextStyle;
use fontdue::layout::{CoordinateSystem, HorizontalAlign, Layout, LayoutSettings};
use fontdue::Font;
use std::sync::OnceLock;

/// CreamUI's bundled default font (DejaVu Sans) — identical bytes to
/// `creamui-render`'s copy; see `assets/fonts/DejaVuSans-LICENSE.txt`.
const FONT_BYTES: &[u8] = include_bytes!("../../../assets/fonts/DejaVuSans.ttf");

fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| {
        Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default())
            .expect("bundled font bytes are a valid, fixed asset checked in at build time")
    })
}

/// A width large enough that single-line text never wraps against it, but
/// far from `f32::MAX` so intermediate arithmetic (`max_width - padding`)
/// can't overflow to infinity/NaN.
const UNBOUNDED_WIDTH: f32 = 1_000_000.0;

/// Returns `(width, height)` in logical pixels for `text` set at
/// `font_size`, laid out as a single line within `max_width` (pass
/// [`UNBOUNDED_WIDTH`], exposed via [`unbounded_width`], for the text's
/// natural, unwrapped width).
///
/// The width comes from `fontdue`'s own end-of-line padding calculation
/// (`max_width - line.padding`) rather than a hand-rolled estimate, so it
/// matches exactly what `fontdue` will do when the renderer lays out the
/// same text with the same `max_width` at paint time — no fudge factor
/// needed.
pub fn measure(text: &str, font_size: f32, max_width: f32) -> (f32, f32) {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        max_width: Some(max_width),
        horizontal_align: HorizontalAlign::Left,
        ..LayoutSettings::default()
    });
    layout.append(&[font()], &TextStyle::new(text, font_size, 0));

    let width = layout
        .lines()
        .and_then(|lines| lines.first())
        .map(|line| (max_width - line.padding).max(0.0))
        .unwrap_or(0.0);
    let height = font_size * 1.4;
    (width.max(1.0), height)
}

/// See [`UNBOUNDED_WIDTH`].
pub fn unbounded_width() -> f32 {
    UNBOUNDED_WIDTH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longer_text_measures_wider() {
        let (short_width, _) = measure("Hi", 16.0, unbounded_width());
        let (long_width, _) = measure("Hello, CreamUI!", 16.0, unbounded_width());
        assert!(long_width > short_width);
    }

    #[test]
    fn constraining_max_width_does_not_exceed_it() {
        let (natural_width, _) = measure("Hello, CreamUI!", 16.0, unbounded_width());
        let (constrained_width, _) = measure("Hello, CreamUI!", 16.0, natural_width);
        assert!(
            (constrained_width - natural_width).abs() < 0.01,
            "measuring with max_width set to the natural width should reproduce that width with no wrap: got {constrained_width}, expected {natural_width}"
        );
    }
}
