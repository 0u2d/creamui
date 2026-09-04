//! [`creamui_core::Painter`] implementation backed by `tiny-skia` CPU
//! rasterization. The resulting pixmap is uploaded to a GPU texture and
//! blitted to the window surface by [`crate::gpu`] — compositing happens on
//! the GPU even though shape/glyph rasterization happens on the CPU.

use crate::font::Font;
use creamui_core::{Painter, Rect, TextAlign};
use creamui_theme::Color;
use fontdue::layout::HorizontalAlign;
use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct SkiaPainter {
    pub pixmap: Pixmap,
    font: Option<Font>,
}

impl SkiaPainter {
    pub fn new(width: u32, height: u32) -> Self {
        SkiaPainter {
            pixmap: Pixmap::new(width.max(1), height.max(1)).expect("non-zero pixmap size"),
            font: Font::load_system(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixmap = Pixmap::new(width.max(1), height.max(1)).expect("non-zero pixmap size");
    }

    pub fn clear(&mut self, color: Color) {
        let [r, g, b, a] = color.to_f32();
        self.pixmap
            .fill(tiny_skia::Color::from_rgba(r, g, b, a).expect("valid color"));
    }

    fn rounded_rect_path(rect: Rect, radius: f32) -> Option<tiny_skia::Path> {
        let radius = radius.min(rect.width / 2.0).min(rect.height / 2.0).max(0.0);
        let mut pb = PathBuilder::new();
        let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
        if radius <= 0.01 {
            pb.push_rect(tiny_skia::Rect::from_xywh(x, y, w, h)?);
        } else {
            const K: f32 = 0.5522847498;
            let r = radius;
            pb.move_to(x + r, y);
            pb.line_to(x + w - r, y);
            pb.cubic_to(x + w - r + r * K, y, x + w, y + r - r * K, x + w, y + r);
            pb.line_to(x + w, y + h - r);
            pb.cubic_to(x + w, y + h - r + r * K, x + w - r + r * K, y + h, x + w - r, y + h);
            pb.line_to(x + r, y + h);
            pb.cubic_to(x + r - r * K, y + h, x, y + h - r + r * K, x, y + h - r);
            pb.line_to(x, y + r);
            pb.cubic_to(x, y + r - r * K, x + r - r * K, y, x + r, y);
            pb.close();
        }
        pb.finish()
    }
}

impl Painter for SkiaPainter {
    fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        let Some(path) = Self::rounded_rect_path(rect, corner_radius) else {
            return;
        };
        let [r, g, b, a] = color.to_f32();
        let mut paint = Paint::default();
        paint.set_color_rgba8(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        );
        paint.anti_alias = true;
        self.pixmap
            .fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32, corner_radius: f32) {
        let Some(path) = Self::rounded_rect_path(rect, corner_radius) else {
            return;
        };
        let [r, g, b, a] = color.to_f32();
        let mut paint = Paint::default();
        paint.set_color_rgba8(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        );
        paint.anti_alias = true;
        let stroke = Stroke {
            width,
            ..Default::default()
        };
        self.pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    fn fill_text(&mut self, rect: Rect, text: &str, color: Color, font_size: f32, align: TextAlign) {
        let Some(font) = self.font.as_mut() else {
            return;
        };
        let horizontal_align = match align {
            TextAlign::Start => HorizontalAlign::Left,
            TextAlign::Center => HorizontalAlign::Center,
            TextAlign::End => HorizontalAlign::Right,
        };
        let glyphs = font.layout_text(text, font_size, rect.x, rect.y, rect.width, rect.height, horizontal_align);

        let pixmap_width = self.pixmap.width() as i32;
        let pixmap_height = self.pixmap.height() as i32;
        let pixels = self.pixmap.pixels_mut();
        let text_alpha = color.a as f32 / 255.0;

        for glyph in glyphs {
            for gy in 0..glyph.height {
                let py = glyph.y + gy as i32;
                if py < 0 || py >= pixmap_height {
                    continue;
                }
                for gx in 0..glyph.width {
                    let px = glyph.x + gx as i32;
                    if px < 0 || px >= pixmap_width {
                        continue;
                    }
                    let coverage = glyph.coverage[gy * glyph.width + gx] as f32 / 255.0;
                    let src_alpha = coverage * text_alpha;
                    if src_alpha <= 0.0 {
                        continue;
                    }

                    let idx = (py * pixmap_width + px) as usize;
                    let dst = pixels[idx];
                    let inv = 1.0 - src_alpha;

                    // Premultiplied src-over: dst channels are already
                    // premultiplied, and `color.r/g/b * src_alpha` is the
                    // premultiplied source contribution.
                    let out_r = (color.r as f32 * src_alpha) + (dst.red() as f32 * inv);
                    let out_g = (color.g as f32 * src_alpha) + (dst.green() as f32 * inv);
                    let out_b = (color.b as f32 * src_alpha) + (dst.blue() as f32 * inv);
                    let out_a = (255.0 * src_alpha) + (dst.alpha() as f32 * inv);

                    pixels[idx] = tiny_skia::PremultipliedColorU8::from_rgba(
                        out_r.min(255.0) as u8,
                        out_g.min(255.0) as u8,
                        out_b.min(255.0) as u8,
                        out_a.min(255.0) as u8,
                    )
                    .unwrap_or(dst);
                }
            }
        }
    }
}
