use super::*;
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

    pub fn customize(mut self, customize: impl FnOnce(&mut RawScrollView)) -> Self {
        customize(&mut self.inner);
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
