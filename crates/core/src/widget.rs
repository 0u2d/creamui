use crate::geometry::Rect;
use std::rc::Rc;

/// Horizontal text alignment within a widget's painted rect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Center,
    Start,
    End,
}

/// Backend-agnostic drawing surface a [`Widget`] paints itself onto.
///
/// Implemented once per rendering backend (e.g. the `tiny-skia` + `wgpu`
/// backend in `creamui-render`); widgets never depend on a specific backend.
pub trait Painter {
    fn fill_rect(&mut self, rect: Rect, color: creamui_theme::Color, corner_radius: f32);
    fn stroke_rect(&mut self, rect: Rect, color: creamui_theme::Color, width: f32, corner_radius: f32);
    fn fill_text(&mut self, rect: Rect, text: &str, color: creamui_theme::Color, font_size: f32, align: TextAlign);
}

/// A node in a CreamUI widget tree.
///
/// Widgets are cheap, immutable descriptions rebuilt on every reactive
/// re-render (similar to an immediate-mode `view()` function); the expensive
/// work — layout and painting — only happens through [`crate::render_frame`].
/// This keeps the model simple now and leaves room for a future retained /
/// diffed tree without changing the trait.
pub trait Widget {
    /// This widget's layout box, expressed as a `taffy` flex/grid style.
    fn style(&self) -> taffy::style::Style;

    /// Paints this widget's own appearance into `rect` (already laid out in
    /// the parent's coordinate space). Does not paint children.
    fn paint(&self, painter: &mut dyn Painter, rect: Rect);

    /// Takes ownership of this widget's children, in layout order, leaving
    /// it childless. Takes `&mut self` (rather than consuming the widget)
    /// so implementors can `std::mem::take` an owned `Vec<BoxedWidget>`
    /// field while keeping the rest of `self` around for [`Widget::paint`].
    /// Default: a leaf widget with no children.
    fn children(&mut self) -> Vec<BoxedWidget> {
        Vec::new()
    }

    /// Optional click handler. When present, this widget's laid-out rect
    /// becomes part of hit-testing for pointer clicks.
    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        None
    }
}

pub type BoxedWidget = Box<dyn Widget>;
