use crate::geometry::{Point, Rect};
use std::ops::Range;
use std::rc::Rc;
use taffy::geometry::Size;
use taffy::style::AvailableSpace;

/// A logical key, decoded from the backend's native key event into
/// something widgets can match on without depending on a specific backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Backspace,
    Delete,
    Enter,
    Tab,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
}

/// A keyboard event delivered to whichever widget currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInput {
    pub key: Key,
    pub modifiers: Modifiers,
}

/// Keyboard modifiers carried with every [`KeyInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
}

/// A widget's intrinsic-size function, used by `taffy`'s layout algorithm
/// for leaves whose size depends on content (e.g. text) rather than being
/// fully determined by their `Style`. Called with the dimensions already
/// pinned down by layout (`known_dimensions`) and the space available on
/// each axis; must return a definite size.
pub type MeasureFn = Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>>;

/// Horizontal text alignment within a widget's painted rect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Center,
    Start,
    End,
}

/// The system mouse cursor shape to show while the pointer hovers a widget.
/// Backend-agnostic, mirroring [`Key`]'s role for keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorIcon {
    #[default]
    Default,
    /// An I-beam, shown over editable text (e.g. a text input).
    Text,
    /// A hand/pointer, shown over clickable widgets.
    Pointer,
    /// A "no" circle-with-a-line, shown over a disabled control.
    NotAllowed,
}

/// Backend-agnostic drawing surface a [`Widget`] paints itself onto.
///
/// Implemented once per rendering backend (e.g. the `tiny-skia` + `wgpu`
/// backend in `creamui-render`); widgets never depend on a specific backend.
pub trait Painter {
    fn fill_rect(&mut self, rect: Rect, color: creamui_theme::Color, corner_radius: f32);
    fn stroke_rect(
        &mut self,
        rect: Rect,
        color: creamui_theme::Color,
        width: f32,
        corner_radius: f32,
    );
    fn fill_text(
        &mut self,
        rect: Rect,
        text: &str,
        color: creamui_theme::Color,
        font_size: f32,
        align: TextAlign,
    );

    /// Draws a text run whose glyph layout stays intact while a byte range
    /// receives a different foreground color. Backends that do not support
    /// per-glyph coloring may use the stable normal-color fallback.
    fn fill_text_selected(
        &mut self,
        rect: Rect,
        text: &str,
        color: creamui_theme::Color,
        selected_color: creamui_theme::Color,
        selected: Range<usize>,
        font_size: f32,
        align: TextAlign,
    ) {
        let _ = (selected_color, selected);
        self.fill_text(rect, text, color, font_size, align);
    }

    /// Restricts all subsequent drawing (until the matching [`Painter::pop_clip`])
    /// to `rect`, intersected with any already-active clip. Used by
    /// scrollable containers to hide content outside their own bounds.
    /// Default: a no-op, for backends/tests that don't need real clipping.
    fn push_clip(&mut self, _rect: Rect) {}

    /// Removes the most recently pushed clip. Must be paired 1:1 with
    /// [`Painter::push_clip`] calls. Default: a no-op.
    fn pop_clip(&mut self) {}
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

    /// Optional intrinsic-size function for content-sized leaves (e.g.
    /// text). Default: `None`, meaning this widget's size is fully
    /// determined by its `Style` (the common case for containers).
    fn measure(&self) -> Option<MeasureFn> {
        None
    }

    /// Whether this widget can receive keyboard focus. A click inside a
    /// focusable widget's rect gives it focus; a click elsewhere clears
    /// focus. Default: `false`.
    fn focusable(&self) -> bool {
        false
    }

    /// Optional keyboard handler, called with each [`KeyInput`] while this
    /// widget has focus (see [`Widget::focusable`]). Default: `None`.
    fn on_key(&self) -> Option<Rc<dyn Fn(KeyInput)>> {
        None
    }

    /// Optional drag handler for press-and-drag interactions (e.g. a
    /// slider). Called on the initial press and on every subsequent pointer
    /// move while the button stays held, with the pointer's position in
    /// this widget's own local coordinates (relative to its rect's
    /// top-left corner — negative or beyond the rect's width/height once
    /// the drag continues past the widget's own bounds) and the widget's
    /// current rect (so e.g. a slider can divide by its own resolved width
    /// without needing to know it ahead of time). Default: `None`.
    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        None
    }

    /// Optional handler for the initial pointer press of a drag gesture.
    /// Kept separate from [`Widget::on_drag`] so text editors can record a
    /// selection anchor before subsequent pointer moves extend the focus.
    fn on_drag_start(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        None
    }

    /// Optional scroll-wheel handler, called with the vertical scroll delta
    /// (in logical pixels; positive scrolls content up, i.e. reveals
    /// content further down) when the pointer is over this widget's rect.
    /// The caller owns the actual scroll offset (see [`Widget::scroll_offset`])
    /// and is responsible for clamping it to whatever range makes sense for
    /// its content. Default: `None`.
    fn on_scroll(&self) -> Option<Rc<dyn Fn(f32)>> {
        None
    }

    /// Whether this widget clips its children to its own rect. Default:
    /// `false`. A scroll view returns `true`.
    fn clips_children(&self) -> bool {
        false
    }

    /// Shifts every child's painted/hit-tested position by this offset
    /// (subtracted from the child's origin — a positive `y` here scrolls
    /// content up, revealing what's further down). Default: no offset.
    /// Meaningless unless [`Widget::clips_children`] is also `true`.
    fn scroll_offset(&self) -> Point {
        Point::default()
    }

    /// The system mouse cursor to show while the pointer hovers this
    /// widget's rect. Default: `None`, meaning this widget expresses no
    /// preference (an ancestor's or the platform default applies instead).
    fn cursor_icon(&self) -> Option<CursorIcon> {
        None
    }

    /// Paints an overlay on top of this widget's normal [`Widget::paint`]
    /// output, called only on the frame's currently focused widget (see
    /// [`Widget::focusable`]). Used for a text input's blinking caret.
    /// `caret_visible` is the current blink phase. Default: a no-op.
    fn paint_focused_overlay(&self, _painter: &mut dyn Painter, _rect: Rect, _caret_visible: bool) {
    }
}

pub type BoxedWidget = Box<dyn Widget>;
