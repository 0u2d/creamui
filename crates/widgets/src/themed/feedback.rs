use super::*;
use crate::layout::{column, fixed, padding, row};
use creamui_core::layout::{AlignItems, Dimension, JustifyContent, Position, Style};

/// A full-parent dimmer used as the base of modal dialogs and transient
/// overlays. Place it after ordinary application content so it paints and
/// receives hit testing above that content.
pub struct Overlay {
    style: Style,
    dismiss: Rc<dyn Fn()>,
    children: Vec<BoxedWidget>,
}

impl Overlay {
    pub fn new(_theme: &Theme, style: Style, on_dismiss: impl Fn() + 'static) -> Self {
        Self {
            style,
            dismiss: Rc::new(on_dismiss),
            children: Vec::new(),
        }
    }

    /// A flex-centered overlay that fills its positioned parent.
    pub fn fullscreen(theme: &Theme, on_dismiss: impl Fn() + 'static) -> Self {
        let style = Style {
            position: Position::Absolute,
            inset: creamui_core::layout::Rect {
                left: creamui_core::layout::LengthPercentageAuto::Length(0.0),
                right: creamui_core::layout::LengthPercentageAuto::Length(0.0),
                top: creamui_core::layout::LengthPercentageAuto::Length(0.0),
                bottom: creamui_core::layout::LengthPercentageAuto::Length(0.0),
            },
            size: creamui_core::layout::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            ..column(0.0)
        };
        Self::new(theme, style, on_dismiss)
    }

    pub fn child(mut self, child: BoxedWidget) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: Vec<BoxedWidget>) -> Self {
        self.children = children;
        self
    }
}

impl Widget for Overlay {
    fn style(&self) -> Style {
        self.style.clone()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        // The alpha deliberately leaves enough of the surrounding app visible
        // to preserve context while making the active layer unambiguous.
        painter.fill_rect(rect, Color::rgba(0, 0, 0, 112), 0.0);
    }
    fn children(&mut self) -> Vec<BoxedWidget> {
        std::mem::take(&mut self.children)
    }
    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        Some(self.dismiss.clone())
    }
    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Default)
    }
}

/// A floating, click-shielding surface. Its own empty click handler prevents
/// an enclosing [`Overlay`] from treating clicks inside the popover as an
/// outside dismissal; interactive descendants still win hit testing.
pub struct Popover {
    theme: Theme,
    style: Style,
    children: Vec<BoxedWidget>,
}

impl Popover {
    pub fn new(theme: &Theme, style: Style) -> Self {
        Self {
            theme: *theme,
            style,
            children: Vec::new(),
        }
    }
    pub fn child(mut self, child: BoxedWidget) -> Self {
        self.children.push(child);
        self
    }
    pub fn with_children(mut self, children: Vec<BoxedWidget>) -> Self {
        self.children = children;
        self
    }
}

impl Widget for Popover {
    fn style(&self) -> Style {
        self.style.clone()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        for spread in (1..=5).rev() {
            let spread = spread as f32;
            painter.fill_rect(
                Rect {
                    x: rect.x - spread,
                    y: rect.y - spread + 3.0,
                    width: rect.width + spread * 2.0,
                    height: rect.height + spread * 2.0,
                },
                Color::rgba(0, 0, 0, 5),
                self.theme.menu_radius + spread,
            );
        }
        painter.fill_rect(rect, self.theme.surface_elevated, self.theme.menu_radius);
        painter.stroke_rect(rect, self.theme.border_strong, 1.0, self.theme.menu_radius);
    }
    fn children(&mut self) -> Vec<BoxedWidget> {
        std::mem::take(&mut self.children)
    }
    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        Some(Rc::new(|| {}))
    }
}

/// A modal dialog. The application keeps its visibility in a `Signal` and
/// conditionally includes this widget in its root tree.
pub struct Dialog {
    theme: Theme,
    title: String,
    message: String,
    dismiss: Rc<dyn Fn()>,
    actions: Vec<(String, ButtonVariant, Rc<dyn Fn()>)>,
}

impl Dialog {
    pub fn new(
        theme: &Theme,
        title: impl Into<String>,
        message: impl Into<String>,
        on_dismiss: impl Fn() + 'static,
    ) -> Self {
        Self {
            theme: *theme,
            title: title.into(),
            message: message.into(),
            dismiss: Rc::new(on_dismiss),
            actions: Vec::new(),
        }
    }

    pub fn action(
        mut self,
        label: impl Into<String>,
        variant: ButtonVariant,
        on_click: impl Fn() + 'static,
    ) -> Self {
        self.actions
            .push((label.into(), variant, Rc::new(on_click)));
        self
    }

    pub fn dismiss_action(mut self, label: impl Into<String>) -> Self {
        let dismiss = self.dismiss.clone();
        self.actions
            .push((label.into(), ButtonVariant::Secondary, dismiss));
        self
    }

    fn overlay_style() -> Style {
        Overlay::fullscreen(&Theme::dark(), || {}).style
    }
}

impl Widget for Dialog {
    fn style(&self) -> Style {
        Self::overlay_style()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        painter.fill_rect(rect, Color::rgba(0, 0, 0, 112), 0.0);
    }
    fn children(&mut self) -> Vec<BoxedWidget> {
        let card_style = padding(
            Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Length(380.0),
                    height: Dimension::Auto,
                },
                ..column(self.theme.spacing_large)
            },
            self.theme.spacing_large,
        );
        let mut content = RawView::new(column(self.theme.spacing_small))
            .child(Box::new(Heading::md(&self.theme, self.title.clone())))
            .child(Box::new(
                Text::secondary(&self.theme, self.message.clone()).align(TextAlign::Start),
            ));
        let mut actions = RawView::new(Style {
            justify_content: Some(JustifyContent::End),
            ..row(self.theme.spacing_medium)
        });
        for (label, variant, action) in &self.actions {
            let action = action.clone();
            actions = actions.child(Box::new(Button::styled(
                &self.theme,
                *variant,
                ButtonSize::Md,
                label.clone(),
                ButtonState::Normal,
                move || action(),
            )));
        }
        content = content.child(Box::new(actions));
        vec![Box::new(
            Popover::new(&self.theme, card_style).child(Box::new(content)),
        )]
    }
    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        Some(self.dismiss.clone())
    }
}

/// Convenience dialog for a short message and one or two explicit actions.
pub struct AlertDialog {
    inner: Dialog,
}

impl AlertDialog {
    pub fn new(
        theme: &Theme,
        title: impl Into<String>,
        message: impl Into<String>,
        on_dismiss: impl Fn() + 'static,
    ) -> Self {
        Self {
            inner: Dialog::new(theme, title, message, on_dismiss),
        }
    }
    pub fn confirm(mut self, label: impl Into<String>, on_confirm: impl Fn() + 'static) -> Self {
        self.inner = self.inner.action(label, ButtonVariant::Primary, on_confirm);
        self
    }
    pub fn dismiss_button(mut self, label: impl Into<String>) -> Self {
        self.inner = self.inner.dismiss_action(label);
        self
    }
}

impl Widget for AlertDialog {
    fn style(&self) -> Style {
        self.inner.style()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect)
    }
    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }
    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        self.inner.on_click()
    }
}

/// A horizontal determinate or indeterminate progress indicator.
pub struct ProgressBar {
    theme: Theme,
    value: Option<f32>,
    style: Style,
}

impl ProgressBar {
    pub fn new(theme: &Theme, value: f32) -> Self {
        Self {
            theme: *theme,
            value: Some(value),
            style: Self::default_style(),
        }
    }
    pub fn indeterminate(theme: &Theme) -> Self {
        Self {
            theme: *theme,
            value: None,
            style: Self::default_style(),
        }
    }
    pub fn default_style() -> Style {
        Style {
            size: fixed(200.0, 10.0),
            ..Default::default()
        }
    }
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for ProgressBar {
    fn style(&self) -> Style {
        self.style.clone()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        let radius = rect.height / 2.0;
        painter.fill_rect(rect, self.theme.surface_hover, radius);
        let (x, width) = match self.value {
            Some(value) => (rect.x, rect.width * value.clamp(0.0, 1.0)),
            None => {
                let width = rect.width * 0.32;
                let travel = (rect.width - width).max(0.0);
                let phase = (painter.animation_time() * 0.8).fract();
                (rect.x + travel * phase, width)
            }
        };
        if width > 0.0 {
            painter.fill_rect(
                Rect {
                    x,
                    y: rect.y,
                    width,
                    height: rect.height,
                },
                self.theme.accent,
                radius,
            );
        }
    }
}

/// A circular determinate or indeterminate progress indicator.
pub struct ProgressRing {
    theme: Theme,
    value: Option<f32>,
    size: f32,
}

impl ProgressRing {
    pub fn new(theme: &Theme, value: f32) -> Self {
        Self {
            theme: *theme,
            value: Some(value),
            size: 28.0,
        }
    }
    pub fn indeterminate(theme: &Theme) -> Self {
        Self {
            theme: *theme,
            value: None,
            size: 28.0,
        }
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Widget for ProgressRing {
    fn style(&self) -> Style {
        Style {
            size: fixed(self.size, self.size),
            ..Default::default()
        }
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        let center = Point {
            x: rect.x + rect.width / 2.0,
            y: rect.y + rect.height / 2.0,
        };
        let radius = rect.width.min(rect.height) * 0.38;
        let stroke = (self.size * 0.12).max(2.0);
        let draw_arc = |p: &mut dyn Painter, start: f32, amount: f32, color: Color| {
            let steps = 32;
            let point = |t: f32| Point {
                x: center.x + radius * t.cos(),
                y: center.y + radius * t.sin(),
            };
            for index in 0..steps {
                let a = start + amount * index as f32 / steps as f32;
                let b = start + amount * (index + 1) as f32 / steps as f32;
                p.stroke_line(point(a), point(b), color, stroke);
            }
        };
        draw_arc(
            painter,
            0.0,
            std::f32::consts::TAU,
            self.theme.surface_hover,
        );
        let (start, amount) = match self.value {
            Some(value) => (
                -std::f32::consts::FRAC_PI_2,
                std::f32::consts::TAU * value.clamp(0.0, 1.0),
            ),
            None => (
                painter.animation_time() * std::f32::consts::TAU,
                std::f32::consts::TAU * 0.28,
            ),
        };
        draw_arc(painter, start, amount, self.theme.accent);
    }
}
