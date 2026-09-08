use super::*;
/// An unstyled clickable region. Paints only its `background`/`border` if
/// set; combine with [`RawText`] as a child for a labeled button.
pub struct RawButton {
    pub hover_background: Option<Color>,
    pub pressed_background: Option<Color>,
    pub focus_color: Option<Color>,
    pub style: Style,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub border: Option<(Color, f32)>,
    pub children: Vec<BoxedWidget>,
    pub on_click: Rc<dyn Fn()>,
    pub disabled: bool,
}

impl RawButton {
    pub fn new(style: Style, on_click: impl Fn() + 'static) -> Self {
        RawButton {
            hover_background: None,
            pressed_background: None,
            focus_color: None,
            style,
            background: None,
            corner_radius: 0.0,
            border: None,
            children: Vec::new(),
            on_click: Rc::new(on_click),
            disabled: false,
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
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border = Some((color, width));
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

    /// While `true`, the button reports no click handler (so it truly can't
    /// be activated, not just visually dimmed) and shows a "not allowed"
    /// cursor instead of the usual pointer.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for RawButton {
    fn focusable(&self) -> bool {
        !self.disabled
    }
    fn on_key(&self) -> Option<Rc<dyn Fn(KeyInput)>> {
        if self.disabled {
            return None;
        }
        let click = self.on_click.clone();
        Some(Rc::new(move |input| {
            if !input.modifiers.ctrl && matches!(input.key, Key::Enter | Key::Char(' ')) {
                click();
            }
        }))
    }
    fn paint_focused_overlay(&self, painter: &mut dyn Painter, rect: Rect, _: bool) {
        if let Some(color) = self.focus_color {
            painter.stroke_rect(
                Rect {
                    x: rect.x - 2.,
                    y: rect.y - 2.,
                    width: rect.width + 4.,
                    height: rect.height + 4.,
                },
                color,
                2.,
                self.corner_radius + 2.,
            );
        }
    }
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        let background = if !self.disabled && painter.pressed(rect) {
            self.pressed_background.or(self.background)
        } else if !self.disabled && painter.hovered(rect) {
            self.hover_background.or(self.background)
        } else {
            self.background
        };
        if let Some(color) = background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
        if let Some((color, width)) = self.border {
            painter.stroke_rect(rect, color, width, self.corner_radius);
        }
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        std::mem::take(&mut self.children)
    }

    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        if self.disabled {
            None
        } else {
            Some(self.on_click.clone())
        }
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(if self.disabled {
            CursorIcon::NotAllowed
        } else {
            CursorIcon::Pointer
        })
    }
}
