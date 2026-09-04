//! End-to-end test: build a themed widget tree, run it through
//! `creamui_core::render_frame` with a recording `Painter`, and verify both
//! painting and click hit-testing work together.

use creamui_core::layout::{AlignItems, JustifyContent, Style};
use creamui_core::{render_frame, Painter, Point, Rect, Size, TextAlign};
use creamui_reactive::Signal;
use creamui_theme::{Color, Theme};
use creamui_widgets::raw::RawView;
use creamui_widgets::themed::Button;

#[derive(Default)]
struct RecordingPainter {
    filled_rects: Vec<(Rect, Color)>,
    texts: Vec<String>,
}

impl Painter for RecordingPainter {
    fn fill_rect(&mut self, rect: Rect, color: Color, _corner_radius: f32) {
        self.filled_rects.push((rect, color));
    }

    fn stroke_rect(&mut self, _rect: Rect, _color: Color, _width: f32, _corner_radius: f32) {}

    fn fill_text(&mut self, _rect: Rect, text: &str, _color: Color, _font_size: f32, _align: TextAlign) {
        self.texts.push(text.to_string());
    }
}

#[test]
fn button_paints_and_responds_to_clicks() {
    let theme = Theme::dark();
    let counter = Signal::new(0);
    let counter_for_click = counter.clone();

    let root_style = Style {
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
        size: creamui_core::layout::Size {
            width: creamui_core::layout::Dimension::Length(400.0),
            height: creamui_core::layout::Dimension::Length(300.0),
        },
        ..Default::default()
    };

    let root = RawView::new(root_style).child(Box::new(Button::new(&theme, "Click me", move || {
        counter_for_click.update(|c| *c += 1);
    })));

    let mut painter = RecordingPainter::default();
    let scene = render_frame(
        Box::new(root),
        Size { width: 400.0, height: 300.0 },
        &mut painter,
    );

    assert_eq!(painter.texts, vec!["Click me".to_string()]);
    assert!(
        painter.filled_rects.iter().any(|(_, color)| *color == theme.accent),
        "button should paint with the theme's accent color"
    );

    let button_rect = painter
        .filled_rects
        .iter()
        .find(|(_, color)| *color == theme.accent)
        .map(|(rect, _)| *rect)
        .expect("button rect");
    let center = Point {
        x: button_rect.x + button_rect.width / 2.0,
        y: button_rect.y + button_rect.height / 2.0,
    };

    let handler = scene.hit_test(center).expect("click inside button should hit");
    handler();
    assert_eq!(counter.get(), 1);

    assert!(
        scene.hit_test(Point { x: 0.0, y: 0.0 }).is_none(),
        "clicking far outside the button should not hit anything"
    );
}
