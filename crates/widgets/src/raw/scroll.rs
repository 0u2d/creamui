use super::*;
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
