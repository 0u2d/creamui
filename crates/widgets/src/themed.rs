//! Themed widgets: opinionated, styled wrappers around the headless widgets
//! in [`crate::raw`]. Each one reads its appearance from a
//! [`creamui_theme::Theme`] passed in at construction time — either a fixed
//! value, or `creamui_theme::ThemeProvider::get()`'s result each render, to
//! support runtime theme switching.
//!
//! These are meant to be copied and adapted: a themed `Button` is nothing
//! more than a [`crate::raw::RawButton`] with theme-derived style baked in,
//! so writing a derived component (e.g. a `DangerButton`) is just writing a
//! new constructor function in the same shape.

use crate::raw::{
    RawButton, RawCheckbox, RawScrollView, RawSlider, RawText, RawTextInput, RawView,
};
use creamui_core::layout::{
    AlignItems, JustifyContent, LengthPercentage, Rect as LayoutRect, Style,
};
use creamui_core::{BoxedWidget, CursorIcon, KeyInput, Painter, Point, Rect, TextAlign, Widget};
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
        Self::with_style(
            theme,
            centered_box_style(theme.spacing_medium * 2.0),
            label,
            on_click,
        )
    }

    /// A themed button with caller-controlled layout. Its colors and radius
    /// still come from `theme`, so an application-wide theme change remains
    /// consistent while each button can choose its own size, margin, or flex
    /// placement.
    pub fn with_style(
        theme: &Theme,
        style: Style,
        label: impl Into<String>,
        on_click: impl Fn() + 'static,
    ) -> Self {
        let text = RawText::new(label, theme.text_primary, 16.0);
        let inner = RawButton::new(style, on_click)
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

    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.inner.cursor_icon()
    }
}

/// A themed checkbox: filled with the theme's accent color when checked,
/// outlined with its border color otherwise.
pub struct Checkbox {
    inner: RawCheckbox,
}

impl Checkbox {
    pub fn new(theme: &Theme, checked: bool, on_click: impl Fn() + 'static) -> Self {
        let mut inner =
            RawCheckbox::new(20.0, checked, theme.accent, theme.border_strong, on_click);
        inner = inner.corner_radius(theme.radius_small);
        Checkbox { inner }
    }
}

impl Widget for Checkbox {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        self.inner.on_click()
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.inner.cursor_icon()
    }
}

/// A themed single-line text input.
pub struct TextInput {
    inner: RawTextInput,
}

impl TextInput {
    /// The style used when none is given explicitly: a fixed 200x36 box,
    /// matching this widget's original hardcoded layout.
    pub fn default_style() -> Style {
        Style {
            size: creamui_core::layout::Size {
                width: creamui_core::layout::Dimension::Length(200.0),
                height: creamui_core::layout::Dimension::Length(36.0),
            },
            ..Default::default()
        }
    }

    pub fn new(
        theme: &Theme,
        value: impl Into<String>,
        on_change: impl Fn(String) + 'static,
    ) -> Self {
        Self::with_style(theme, Self::default_style(), value, on_change)
    }

    /// Same as [`TextInput::new`], but with full control over layout
    /// instead of the fixed 200x36 default.
    pub fn with_style(
        theme: &Theme,
        style: Style,
        value: impl Into<String>,
        on_change: impl Fn(String) + 'static,
    ) -> Self {
        let inner = RawTextInput::new(style, value, 14.0, theme.text_primary, on_change)
            .background(theme.surface_elevated)
            .border(theme.border, 1.0)
            .corner_radius(theme.radius_small);
        TextInput { inner }
    }

    /// Grayed-out text shown when the value is empty.
    pub fn placeholder(mut self, theme: &Theme, text: impl Into<String>) -> Self {
        self.inner = self.inner.placeholder(text, theme.text_disabled);
        self
    }
}

impl Widget for TextInput {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn focusable(&self) -> bool {
        self.inner.focusable()
    }

    fn on_key(&self) -> Option<Rc<dyn Fn(KeyInput)>> {
        self.inner.on_key()
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.inner.cursor_icon()
    }

    fn paint_focused_overlay(&self, painter: &mut dyn Painter, rect: Rect, caret_visible: bool) {
        self.inner
            .paint_focused_overlay(painter, rect, caret_visible);
    }
}

/// A themed horizontal slider.
pub struct Slider {
    inner: RawSlider,
}

impl Slider {
    /// The style used when none is given explicitly: a fixed 160x20 box,
    /// matching this widget's original hardcoded layout.
    pub fn default_style() -> Style {
        Style {
            size: creamui_core::layout::Size {
                width: creamui_core::layout::Dimension::Length(160.0),
                height: creamui_core::layout::Dimension::Length(20.0),
            },
            ..Default::default()
        }
    }

    pub fn new(theme: &Theme, value: f32, on_change: impl Fn(f32) + 'static) -> Self {
        Self::with_style(theme, Self::default_style(), value, on_change)
    }

    /// Same as [`Slider::new`], but with full control over layout instead
    /// of the fixed 160x20 default.
    pub fn with_style(
        theme: &Theme,
        style: Style,
        value: f32,
        on_change: impl Fn(f32) + 'static,
    ) -> Self {
        let inner = RawSlider::new(
            style,
            value,
            theme.border_strong,
            theme.accent,
            theme.accent,
            on_change,
        );
        Slider { inner }
    }
}

impl Widget for Slider {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        self.inner.on_drag()
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.inner.cursor_icon()
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

    /// Overrides the semantic primary/secondary color for cases such as a
    /// brand mark or a status value.
    pub fn color(mut self, color: creamui_theme::Color) -> Self {
        self.inner.color = color;
        self
    }

    /// Gives text a layout style for width, margin, flex/grid placement, etc.
    pub fn style(mut self, style: Style) -> Self {
        self.inner.style = style;
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

    fn measure(&self) -> Option<creamui_core::MeasureFn> {
        self.inner.measure()
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

/// A themed vertically-scrollable container.
pub struct ScrollView {
    inner: RawScrollView,
}

impl ScrollView {
    pub fn new(
        theme: &Theme,
        style: Style,
        scroll_y: f32,
        on_scroll: impl Fn(f32) + 'static,
    ) -> Self {
        let inner = RawScrollView::new(style, scroll_y, on_scroll)
            .background(theme.surface)
            .corner_radius(theme.radius_medium);
        ScrollView { inner }
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

impl Widget for ScrollView {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        Widget::children(&mut self.inner)
    }

    fn clips_children(&self) -> bool {
        self.inner.clips_children()
    }

    fn scroll_offset(&self) -> Point {
        self.inner.scroll_offset()
    }

    fn on_scroll(&self) -> Option<Rc<dyn Fn(f32)>> {
        self.inner.on_scroll()
    }
}
