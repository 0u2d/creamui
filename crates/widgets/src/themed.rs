//! Themed widgets: opinionated, styled wrappers around the headless widgets
//! in [`crate::raw`]. Each one reads its appearance from a
//! [`creamui_theme::Theme`] passed in at construction time (the "one theme,
//! in memory" model for the MVP — see the crate root docs for the future
//! `ThemeProvider`).
//!
//! These are meant to be copied and adapted: a themed `Button` is nothing
//! more than a [`crate::raw::RawButton`] with theme-derived style baked in,
//! so writing a derived component (e.g. a `DangerButton`) is just writing a
//! new constructor function in the same shape.

use crate::raw::{RawButton, RawText, RawView};
use creamui_core::layout::{AlignItems, JustifyContent, LengthPercentage, Rect as LayoutRect, Style};
use creamui_core::{BoxedWidget, Painter, Rect, TextAlign, Widget};
use creamui_theme::Theme;
use std::rc::Rc;

fn centered_box_style(padding: f32) -> Style {
    Style {
        padding: LayoutRect {
            left: LengthPercentage::Length(padding),
            right: LengthPercentage::Length(padding),
            top: LengthPercentage::Length(padding * 0.6),
            bottom: LengthPercentage::Length(padding * 0.6),
        },
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
        ..Default::default()
    }
}

/// A themed, clickable button with a centered text label.
pub struct Button {
    inner: RawButton,
}

impl Button {
    pub fn new(theme: &Theme, label: impl Into<String>, on_click: impl Fn() + 'static) -> Self {
        let text = RawText::new(label, theme.text_primary, 16.0);
        let inner = RawButton::new(centered_box_style(theme.spacing_medium * 2.0), on_click)
            .background(theme.accent)
            .corner_radius(theme.radius_medium)
            .child(Box::new(text));
        Button { inner }
    }
}

impl Widget for Button {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }

    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        self.inner.on_click()
    }
}

/// Themed body text using the theme's primary text color.
pub struct Text {
    inner: RawText,
}

impl Text {
    pub fn new(theme: &Theme, text: impl Into<String>) -> Self {
        Text {
            inner: RawText::new(text, theme.text_primary, 14.0),
        }
    }

    /// Same as [`Text::new`] but using the theme's secondary (muted) text color.
    pub fn secondary(theme: &Theme, text: impl Into<String>) -> Self {
        Text {
            inner: RawText::new(text, theme.text_secondary, 14.0),
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.inner.font_size = size;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.inner.align = align;
        self
    }
}

impl Widget for Text {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }
}

/// A themed surface container ("card") with background and rounded corners.
pub struct View {
    inner: RawView,
}

impl View {
    pub fn new(theme: &Theme, style: Style) -> Self {
        View {
            inner: RawView::new(style)
                .background(theme.surface_elevated)
                .corner_radius(theme.radius_medium),
        }
    }

    pub fn child(mut self, widget: BoxedWidget) -> Self {
        self.inner = self.inner.child(widget);
        self
    }

    pub fn with_children(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.inner = self.inner.with_children(widgets);
        self
    }
}

impl Widget for View {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        Widget::children(&mut self.inner)
    }
}
