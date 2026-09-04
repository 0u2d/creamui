//! Glyph rasterization via `fontdue`, loading a system sans-serif font.
//!
//! CreamUI does not bundle a font for the MVP: it probes common Linux font
//! paths at startup and uses the first one found. Bundling a default font
//! (and proper `fontconfig` integration) is tracked on the roadmap.

use fontdue::layout::{CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle, VerticalAlign};
use fontdue::Font as FontdueFont;

const CANDIDATE_PATHS: &[&str] = &[
    "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
    "/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf",
    "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];

/// A loaded font ready for rasterization, plus a reusable text layout buffer.
pub struct Font {
    inner: FontdueFont,
    layout: Layout,
}

/// One rasterized glyph, positioned in the coordinate space passed to
/// [`Font::layout_text`].
pub struct PositionedGlyph {
    pub x: i32,
    pub y: i32,
    pub width: usize,
    pub height: usize,
    /// Per-pixel coverage (`0..=255`), row-major, `width * height` long.
    pub coverage: Vec<u8>,
}

impl Font {
    /// Loads the first available system font from a fixed list of common
    /// Linux install paths. Returns `None` if none exist, in which case
    /// callers should skip text rendering rather than panic.
    pub fn load_system() -> Option<Self> {
        for path in CANDIDATE_PATHS {
            if let Ok(bytes) = std::fs::read(path) {
                match FontdueFont::from_bytes(bytes, fontdue::FontSettings::default()) {
                    Ok(inner) => {
                        log::debug!("creamui-render: loaded font from {path}");
                        return Some(Font {
                            inner,
                            layout: Layout::new(CoordinateSystem::PositiveYDown),
                        });
                    }
                    Err(err) => log::debug!("creamui-render: failed to parse font at {path}: {err}"),
                }
            }
        }
        log::warn!("creamui-render: no system font found in known paths; text will not render");
        None
    }

    /// Lays out `text` inside a box of `max_width` starting at `(x, y)`,
    /// centering it both horizontally and vertically, and rasterizes each
    /// glyph into a coverage mask.
    pub fn layout_text(
        &mut self,
        text: &str,
        font_size: f32,
        x: f32,
        y: f32,
        max_width: f32,
        max_height: f32,
        align: HorizontalAlign,
    ) -> Vec<PositionedGlyph> {
        self.layout.reset(&LayoutSettings {
            x,
            y,
            max_width: Some(max_width),
            max_height: Some(max_height),
            horizontal_align: align,
            vertical_align: VerticalAlign::Middle,
            ..LayoutSettings::default()
        });
        self.layout
            .append(&[&self.inner], &TextStyle::new(text, font_size, 0));

        self.layout
            .glyphs()
            .iter()
            .filter(|g| g.width > 0 && g.height > 0)
            .map(|g| {
                let (_, bitmap) = self.inner.rasterize_config(g.key);
                PositionedGlyph {
                    x: g.x as i32,
                    y: g.y as i32,
                    width: g.width,
                    height: g.height,
                    coverage: bitmap,
                }
            })
            .collect()
    }
}
