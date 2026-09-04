//! Headless widgets: fully unstyled building blocks with no opinion on
//! color, radius, or spacing. Themed widgets (see [`crate::themed`]) wrap
//! these and fill in appearance from a [`creamui_theme::Theme`]; apps that
//! want a completely custom look can use these directly instead.

use creamui_core::layout::Style;
use creamui_core::{BoxedWidget, Painter, Rect, TextAlign, Widget};
use creamui_theme::Color;
use std::rc::Rc;

/// An unstyled rectangular container that lays out its children.
pub struct RawView {
    pub style: Style,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub children: Vec<BoxedWidget>,
}

impl RawView {
    pub fn new(style: Style) -> Self {
        RawView {
            style,
            background: None,
            corner_radius: 0.0,
            children: Vec::new(),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn child(mut self, widget: BoxedWidget) -> Self {
        self.children.push(widget);
        self
    }

    pub fn with_children(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.children = widgets;
        self
    }
}

impl Widget for RawView {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        std::mem::take(&mut self.children)
    }
}

/// Unstyled text with no color or size opinion beyond what's passed in.
pub struct RawText {
    pub text: String,
    pub color: Color,
    pub font_size: f32,
    pub align: TextAlign,
    pub style: Style,
}

impl RawText {
    pub fn new(text: impl Into<String>, color: Color, font_size: f32) -> Self {
        RawText {
            text: text.into(),
            color,
            font_size,
            align: TextAlign::Center,
            style: Style::default(),
        }
    }
}

impl Widget for RawText {
    fn style(&self) -> Style {
        let mut style = self.style.clone();
        if style.size.width == creamui_core::layout::Dimension::Auto
            && style.size.height == creamui_core::layout::Dimension::Auto
        {
            let (width, height) = crate::text_metrics::measure(&self.text, self.font_size);
            style.size = creamui_core::layout::Size {
                width: creamui_core::layout::Dimension::Length(width),
                height: creamui_core::layout::Dimension::Length(height),
            };
        }
        style
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        painter.fill_text(rect, &self.text, self.color, self.font_size, self.align);
    }
}

/// An unstyled clickable region. Paints only its `background`/`border` if
/// set; combine with [`RawText`] as a child for a labeled button.
pub struct RawButton {
    pub style: Style,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub children: Vec<BoxedWidget>,
    pub on_click: Rc<dyn Fn()>,
}

impl RawButton {
    pub fn new(style: Style, on_click: impl Fn() + 'static) -> Self {
        RawButton {
            style,
            background: None,
            corner_radius: 0.0,
            children: Vec::new(),
            on_click: Rc::new(on_click),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn child(mut self, widget: BoxedWidget) -> Self {
        self.children.push(widget);
        self
    }
}

impl Widget for RawButton {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        std::mem::take(&mut self.children)
    }

    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        Some(self.on_click.clone())
    }
}
