//! Headless widgets: fully unstyled building blocks with no opinion on
//! color, radius, or spacing. Themed widgets (see [`crate::themed`]) wrap
//! these and fill in appearance from a [`creamui_theme::Theme`]; apps that
//! want a completely custom look can use these directly instead.

use creamui_core::layout::Style;
use creamui_core::{
    BoxedWidget, CursorIcon, Key, KeyInput, Painter, Point, Rect, TextAlign, Widget,
};
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

    /// Replaces the layout style. Useful when a base style is refined by a
    /// reusable component before it is returned.
    pub fn layout_style(mut self, style: Style) -> Self {
        self.style = style;
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

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn layout_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for RawText {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        painter.fill_text(rect, &self.text, self.color, self.font_size, self.align);
    }

    fn measure(&self) -> Option<creamui_core::MeasureFn> {
        let text = self.text.clone();
        let font_size = self.font_size;
        Some(Box::new(move |known_dimensions, available_space| {
            let max_width = match (known_dimensions.width, available_space.width) {
                (Some(w), _) => w,
                (None, creamui_core::layout::AvailableSpace::Definite(w)) => w,
                (None, _) => crate::text_metrics::unbounded_width(),
            };
            let (natural_width, natural_height) =
                crate::text_metrics::measure(&text, font_size, max_width);
            creamui_core::layout::Size {
                width: known_dimensions.width.unwrap_or(natural_width),
                height: known_dimensions.height.unwrap_or(natural_height),
            }
        }))
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

    pub fn layout_style(mut self, style: Style) -> Self {
        self.style = style;
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

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }
}

/// An unstyled single-line text input. The caller owns the current text
/// (typically a `String` `Signal`) and updates it from `on_change`, fired
/// on every keystroke — same "no internal state" pattern as every other
/// widget. Supports appending characters and backspace; cursor
/// positioning/selection is not implemented yet (see ROADMAP.md).
pub struct RawTextInput {
    pub style: Style,
    pub value: String,
    pub placeholder: String,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub background: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub font_size: f32,
    pub on_change: Rc<dyn Fn(String)>,
}

impl RawTextInput {
    pub fn new(
        style: Style,
        value: impl Into<String>,
        font_size: f32,
        text_color: Color,
        on_change: impl Fn(String) + 'static,
    ) -> Self {
        RawTextInput {
            style,
            value: value.into(),
            placeholder: String::new(),
            text_color,
            placeholder_color: text_color,
            background: None,
            border_color: None,
            border_width: 1.0,
            corner_radius: 0.0,
            font_size,
            on_change: Rc::new(on_change),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>, color: Color) -> Self {
        self.placeholder = text.into();
        self.placeholder_color = color;
        self
    }
}

impl Widget for RawTextInput {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
        if let Some(color) = self.border_color {
            painter.stroke_rect(rect, color, self.border_width, self.corner_radius);
        }
        let padding = 8.0;
        let text_rect = Rect {
            x: rect.x + padding,
            y: rect.y,
            width: (rect.width - padding * 2.0).max(0.0),
            height: rect.height,
        };
        if self.value.is_empty() {
            if !self.placeholder.is_empty() {
                painter.fill_text(
                    text_rect,
                    &self.placeholder,
                    self.placeholder_color,
                    self.font_size,
                    TextAlign::Start,
                );
            }
        } else {
            painter.fill_text(
                text_rect,
                &self.value,
                self.text_color,
                self.font_size,
                TextAlign::Start,
            );
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Text)
    }

    fn paint_focused_overlay(&self, painter: &mut dyn Painter, rect: Rect, caret_visible: bool) {
        if !caret_visible {
            return;
        }
        let padding = 8.0;
        let (text_width, _) = crate::text_metrics::measure(
            &self.value,
            self.font_size,
            crate::text_metrics::unbounded_width(),
        );
        let text_width = if self.value.is_empty() {
            0.0
        } else {
            text_width
        };
        let caret_x = (rect.x + padding + text_width)
            .min(rect.x + rect.width - 1.0)
            .max(rect.x);
        let caret_height = (self.font_size * 1.2).min(rect.height);
        let caret_rect = Rect {
            x: caret_x,
            y: rect.y + (rect.height - caret_height) / 2.0,
            width: 1.5,
            height: caret_height,
        };
        painter.fill_rect(caret_rect, self.text_color, 0.0);
    }

    fn on_key(&self) -> Option<Rc<dyn Fn(KeyInput)>> {
        let value = self.value.clone();
        let on_change = self.on_change.clone();
        Some(Rc::new(move |input: KeyInput| {
            let mut next = value.clone();
            match input.key {
                Key::Char(c) => next.push(c),
                Key::Backspace => {
                    next.pop();
                }
                _ => return,
            }
            on_change(next);
        }))
    }
}

/// An unstyled checkbox: a fixed-size clickable box, filled when `checked`
/// and outlined otherwise. The caller owns the checked state (typically a
/// `bool` `Signal`) and toggles it from `on_click` — this widget has no
/// state of its own, same as every other widget (see `creamui_core::Widget`).
pub struct RawCheckbox {
    pub box_size: f32,
    pub checked: bool,
    pub fill_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
    pub on_click: Rc<dyn Fn()>,
}

impl RawCheckbox {
    pub fn new(
        box_size: f32,
        checked: bool,
        fill_color: Color,
        border_color: Color,
        on_click: impl Fn() + 'static,
    ) -> Self {
        RawCheckbox {
            box_size,
            checked,
            fill_color,
            border_color,
            border_width: 1.5,
            corner_radius: 0.0,
            on_click: Rc::new(on_click),
        }
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

impl Widget for RawCheckbox {
    fn style(&self) -> Style {
        Style {
            size: creamui_core::layout::Size {
                width: creamui_core::layout::Dimension::Length(self.box_size),
                height: creamui_core::layout::Dimension::Length(self.box_size),
            },
            ..Default::default()
        }
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if self.checked {
            painter.fill_rect(rect, self.fill_color, self.corner_radius);
        } else {
            painter.stroke_rect(
                rect,
                self.border_color,
                self.border_width,
                self.corner_radius,
            );
        }
    }

    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        Some(self.on_click.clone())
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }
}

/// An unstyled horizontal slider: drag (or click) anywhere along its track
/// to set a `0.0..=1.0` value. The caller owns the value (typically an
/// `f32` `Signal`) and updates it from `on_change` — same pattern as every
/// other interactive widget here.
pub struct RawSlider {
    pub style: Style,
    pub value: f32,
    pub track_color: Color,
    pub fill_color: Color,
    pub handle_color: Color,
    pub track_height: f32,
    pub handle_size: f32,
    pub on_change: Rc<dyn Fn(f32)>,
}

impl RawSlider {
    pub fn new(
        style: Style,
        value: f32,
        track_color: Color,
        fill_color: Color,
        handle_color: Color,
        on_change: impl Fn(f32) + 'static,
    ) -> Self {
        RawSlider {
            style,
            value: value.clamp(0.0, 1.0),
            track_color,
            fill_color,
            handle_color,
            track_height: 4.0,
            handle_size: 16.0,
            on_change: Rc::new(on_change),
        }
    }
}

impl Widget for RawSlider {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        let track_y = rect.y + (rect.height - self.track_height) / 2.0;
        let usable_width = (rect.width - self.handle_size).max(0.0);
        let handle_center_x = rect.x + self.handle_size / 2.0 + usable_width * self.value;

        painter.fill_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: rect.width,
                height: self.track_height,
            },
            self.track_color,
            self.track_height / 2.0,
        );
        painter.fill_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: (handle_center_x - rect.x).max(0.0),
                height: self.track_height,
            },
            self.fill_color,
            self.track_height / 2.0,
        );
        painter.fill_rect(
            Rect {
                x: handle_center_x - self.handle_size / 2.0,
                y: rect.y + (rect.height - self.handle_size) / 2.0,
                width: self.handle_size,
                height: self.handle_size,
            },
            self.handle_color,
            self.handle_size / 2.0,
        );
    }

    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        let on_change = self.on_change.clone();
        let handle_size = self.handle_size;
        Some(Rc::new(move |local: Point, rect: Rect| {
            let usable_width = (rect.width - handle_size).max(1.0);
            let fraction = ((local.x - handle_size / 2.0) / usable_width).clamp(0.0, 1.0);
            on_change(fraction);
        }))
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }
}

/// An unstyled vertically-scrollable container. The caller owns the scroll
/// offset (typically an `f32` `Signal`, clamped however makes sense for the
/// content) and updates it from `on_scroll` — same pattern as every other
/// interactive widget here. Children are laid out at their natural height
/// (never flex-shrunk to fit the visible viewport, which would defeat the
/// point of scrolling) and clipped + offset to this widget's own rect.
pub struct RawScrollView {
    pub style: Style,
    pub scroll_y: f32,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub children: Vec<BoxedWidget>,
    pub on_scroll: Rc<dyn Fn(f32)>,
}

impl RawScrollView {
    pub fn new(style: Style, scroll_y: f32, on_scroll: impl Fn(f32) + 'static) -> Self {
        RawScrollView {
            style,
            scroll_y,
            background: None,
            corner_radius: 0.0,
            children: Vec::new(),
            on_scroll: Rc::new(on_scroll),
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

impl Widget for RawScrollView {
    fn style(&self) -> Style {
        // Always Column, regardless of what the caller passes: the sole
        // child is the content wrapper (see `children()` below), and it
        // needs Column's cross axis (width) to `align-items: stretch` to
        // the container's width while its main axis (height) stays
        // content-based — the combination that makes "as wide as the
        // viewport, as tall as the content" actually happen. Row direction
        // would stretch the wrapper's *height* instead, defeating scrolling
        // entirely (its children would get flex-shrunk to fit).
        Style {
            display: creamui_core::layout::Display::Flex,
            flex_direction: creamui_core::layout::FlexDirection::Column,
            ..self.style.clone()
        }
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        // Wrap the real children in a non-shrinking content column so they
        // keep their natural (possibly taller-than-viewport) height instead
        // of being flex-shrunk to fit — that overflow is exactly what makes
        // scrolling meaningful in the first place.
        let content_style = Style {
            display: creamui_core::layout::Display::Flex,
            flex_direction: creamui_core::layout::FlexDirection::Column,
            flex_shrink: 0.0,
            ..Default::default()
        };
        let content = RawView::new(content_style).with_children(std::mem::take(&mut self.children));
        vec![Box::new(content) as BoxedWidget]
    }

    fn clips_children(&self) -> bool {
        true
    }

    fn scroll_offset(&self) -> Point {
        Point {
            x: 0.0,
            y: self.scroll_y,
        }
    }

    fn on_scroll(&self) -> Option<Rc<dyn Fn(f32)>> {
        Some(self.on_scroll.clone())
    }
}
