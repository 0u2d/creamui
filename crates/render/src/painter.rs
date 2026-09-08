//! [`creamui_core::Painter`] implementation backed by `tiny-skia` CPU
//! rasterization. The resulting pixmap is uploaded to a GPU texture and
//! blitted to the window surface by [`crate::gpu`] — compositing happens on
//! the GPU even though shape/glyph rasterization happens on the CPU.

use crate::font::Font;
use creamui_core::{Painter, Point, Rect, TextAlign};
use creamui_theme::Color;
use fontdue::layout::HorizontalAlign;
use tiny_skia::{Mask, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Widgets are laid out and painted in logical (DPI-independent) pixels;
/// `SkiaPainter` scales every coordinate by `scale` (the window's
/// `scale_factor`) before rasterizing, so the backing `pixmap` — and the
/// GPU texture it's uploaded into — are always sized in physical pixels for
/// crisp output on HiDPI displays.
pub struct SkiaPainter {
    pub pixmap: Pixmap,
    font: Font,
    bold_font: Font,
    pub pointer: Option<Point>,
    pub press_origin: Option<Point>,
    pub animated: bool,
    started: std::time::Instant,
    scale: f32,
    /// One [`Mask`] per active [`Painter::push_clip`], each already
    /// intersected with its parent so the top of the stack is always the
    /// full cumulative clip region.
    clip_stack: Vec<Mask>,
}

impl SkiaPainter {
    /// `width`/`height` are physical pixels.
    pub fn new(width: u32, height: u32) -> Self {
        SkiaPainter {
            pixmap: Pixmap::new(width.max(1), height.max(1)).expect("non-zero pixmap size"),
            font: Font::load(),
            bold_font: Font::bold(),
            pointer: None,
            press_origin: None,
            animated: false,
            started: std::time::Instant::now(),
            scale: 1.0,
            clip_stack: Vec::new(),
        }
    }

    /// Sets the logical-to-physical pixel scale factor applied to every
    /// subsequent paint call.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.01);
    }

    /// `width`/`height` are physical pixels.
    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        // Reallocating a full window-sized pixel buffer for every reactive
        // frame dominated pointer-drag time. Most frames are not resizes;
        // retain and clear the existing buffer in that overwhelmingly common
        // case.
        if self.pixmap.width() == width && self.pixmap.height() == height {
            return;
        }
        self.pixmap = Pixmap::new(width, height).expect("non-zero pixmap size");
        // A resize mid-clip-stack shouldn't happen (push/pop are balanced
        // within one frame, and resize only ever runs between frames), but
        // clear defensively rather than risk stale masks sized for the old pixmap.
        self.clip_stack.clear();
    }

    pub fn clear(&mut self, color: Color) {
        self.animated = false;
        let [r, g, b, a] = color.to_f32();
        self.pixmap
            .fill(tiny_skia::Color::from_rgba(r, g, b, a).expect("valid color"));
    }

    fn draw_text(
        &mut self,
        rect: Rect,
        text: &str,
        color: Color,
        selected: Option<(std::ops::Range<usize>, Color)>,
        font_size: f32,
        align: TextAlign,
        bold: bool,
    ) {
        let horizontal_align = match align {
            TextAlign::Start => HorizontalAlign::Left,
            TextAlign::Center => HorizontalAlign::Center,
            TextAlign::End => HorizontalAlign::Right,
        };
        let rect = scale_rect(rect, self.scale);
        let font = if bold {
            &mut self.bold_font
        } else {
            &mut self.font
        };
        let glyphs = font.layout_text(
            text,
            font_size * self.scale,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            horizontal_align,
        );
        let pixmap_width = self.pixmap.width() as i32;
        let pixmap_height = self.pixmap.height() as i32;
        let clip_mask = self.clip_stack.last();
        let pixels = self.pixmap.pixels_mut();
        for glyph in glyphs {
            let glyph_color = selected
                .as_ref()
                .filter(|(range, _)| range.contains(&glyph.byte_offset))
                .map_or(color, |(_, selected_color)| *selected_color);
            let text_alpha = glyph_color.a as f32 / 255.0;
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
                    let idx = (py * pixmap_width + px) as usize;
                    let clip = clip_mask
                        .map(|mask| mask.data()[idx] as f32 / 255.0)
                        .unwrap_or(1.0);
                    let src_alpha = coverage * text_alpha * clip;
                    if src_alpha <= 0.0 {
                        continue;
                    }
                    let dst = pixels[idx];
                    let inv = 1.0 - src_alpha;
                    let out_r = (glyph_color.r as f32 * src_alpha) + (dst.red() as f32 * inv);
                    let out_g = (glyph_color.g as f32 * src_alpha) + (dst.green() as f32 * inv);
                    let out_b = (glyph_color.b as f32 * src_alpha) + (dst.blue() as f32 * inv);
                    let out_a = (255.0 * src_alpha) + (dst.alpha() as f32 * inv);
                    pixels[idx] = tiny_skia::PremultipliedColorU8::from_rgba(
                        out_r.round() as u8,
                        out_g.round() as u8,
                        out_b.round() as u8,
                        out_a.round() as u8,
                    )
                    .unwrap_or(dst);
                }
            }
        }
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
            pb.cubic_to(
                x + w,
                y + h - r + r * K,
                x + w - r + r * K,
                y + h,
                x + w - r,
                y + h,
            );
            pb.line_to(x + r, y + h);
            pb.cubic_to(x + r - r * K, y + h, x, y + h - r + r * K, x, y + h - r);
            pb.line_to(x, y + r);
            pb.cubic_to(x, y + r - r * K, x + r - r * K, y, x + r, y);
            pb.close();
        }
        pb.finish()
    }
}

fn scale_rect(rect: Rect, scale: f32) -> Rect {
    Rect {
        x: rect.x * scale,
        y: rect.y * scale,
        width: rect.width * scale,
        height: rect.height * scale,
    }
}

impl Painter for SkiaPainter {
    fn hovered(&self, rect: Rect) -> bool {
        self.pointer.is_some_and(|point| rect.contains(point))
    }
    fn pressed(&self, rect: Rect) -> bool {
        self.hovered(rect) && self.press_origin.is_some_and(|point| rect.contains(point))
    }
    fn animation_time(&mut self) -> f32 {
        self.animated = true;
        self.started.elapsed().as_secs_f32()
    }
    fn stroke_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        let mut path = PathBuilder::new();
        path.move_to(from.x * self.scale, from.y * self.scale);
        path.line_to(to.x * self.scale, to.y * self.scale);
        if let Some(path) = path.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, color.a);
            let stroke = Stroke {
                width: width * self.scale,
                line_cap: tiny_skia::LineCap::Round,
                ..Default::default()
            };
            self.pixmap.stroke_path(
                &path,
                &paint,
                &stroke,
                Transform::identity(),
                self.clip_stack.last(),
            );
        }
    }
    fn fill_text_weight(
        &mut self,
        rect: Rect,
        text: &str,
        color: Color,
        font_size: f32,
        align: TextAlign,
        bold: bool,
    ) {
        self.draw_text(rect, text, color, None, font_size, align, bold);
    }
    fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        let Some(path) =
            Self::rounded_rect_path(scale_rect(rect, self.scale), corner_radius * self.scale)
        else {
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
        self.pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            self.clip_stack.last(),
        );
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32, corner_radius: f32) {
        let Some(path) =
            Self::rounded_rect_path(scale_rect(rect, self.scale), corner_radius * self.scale)
        else {
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
            width: width * self.scale,
            ..Default::default()
        };
        self.pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            Transform::identity(),
            self.clip_stack.last(),
        );
    }

    fn push_clip(&mut self, rect: Rect) {
        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let scaled = scale_rect(rect, self.scale);

        let Some(path) = Self::rounded_rect_path(scaled, 0.0) else {
            // Degenerate (zero-size) clip rect: nothing inside it can be
            // visible, so push a fully-blocking (all-zero) mask.
            if let Some(mask) = Mask::new(width, height) {
                self.clip_stack.push(mask);
            }
            return;
        };

        let mask = match self.clip_stack.last() {
            Some(parent) => {
                let mut mask = parent.clone();
                mask.intersect_path(
                    &path,
                    tiny_skia::FillRule::Winding,
                    true,
                    Transform::identity(),
                );
                mask
            }
            None => {
                let mut mask = Mask::new(width, height).expect("non-zero pixmap size");
                mask.fill_path(
                    &path,
                    tiny_skia::FillRule::Winding,
                    true,
                    Transform::identity(),
                );
                mask
            }
        };
        self.clip_stack.push(mask);
    }

    fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    fn fill_text(
        &mut self,
        rect: Rect,
        text: &str,
        color: Color,
        font_size: f32,
        align: TextAlign,
    ) {
        self.draw_text(rect, text, color, None, font_size, align, false);
    }

    fn fill_text_selected(
        &mut self,
        rect: Rect,
        text: &str,
        color: Color,
        selected_color: Color,
        selected: std::ops::Range<usize>,
        font_size: f32,
        align: TextAlign,
    ) {
        self.draw_text(
            rect,
            text,
            color,
            Some((selected, selected_color)),
            font_size,
            align,
            false,
        );
    }
}
