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
    RawButton, RawCheckbox, RawScrollView, RawSidebar, RawSlider, RawSpinner, RawSwitch, RawTab, RawTabs, RawText,
    RawTextArea, RawTextInput, RawView, TabIndicatorSide,
};
use creamui_core::layout::{
    AlignItems, JustifyContent, LengthPercentage, Rect as LayoutRect, Style,
};
use creamui_core::{BoxedWidget, CursorIcon, KeyInput, Painter, Point, Rect, TextAlign, Widget};
use creamui_theme::{SelectionStyle, Theme};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize { Xs, Sm, Md, Lg, Xl }
impl ButtonSize {
    fn padding(self) -> f32 { match self { Self::Xs => 6., Self::Sm => 9., Self::Md => 12., Self::Lg => 16., Self::Xl => 20. } }
    fn font_size(self) -> f32 { match self { Self::Xs => 11., Self::Sm => 12., Self::Md => 14., Self::Lg => 16., Self::Xl => 18. } }
    fn border_width(self) -> f32 { match self { Self::Xs | Self::Sm => 1., Self::Md => 1.25, Self::Lg => 1.5, Self::Xl => 2. } }
    fn height(self) -> f32 { match self { Self::Xs => 24., Self::Sm => 28., Self::Md => 34., Self::Lg => 40., Self::Xl => 48. } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState { Normal, Loading, Success }
/// Visual hierarchy for native application actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant { Primary, Secondary, Tertiary, Destructive, Success }

impl Button {
    /// A complete native-control button: variant, size and state are all
    /// semantic, so apps don't have to hand-pick raw rectangles.
    pub fn styled(theme: &Theme, variant: ButtonVariant, size: ButtonSize, label: impl Into<String>, state: ButtonState, on_click: impl Fn() + 'static) -> Self {
        let (background, border, foreground) = match variant {
            ButtonVariant::Primary => (theme.accent, theme.accent_hover, theme.selection_text),
            ButtonVariant::Secondary => (theme.surface_elevated, theme.border_strong, theme.text_primary),
            ButtonVariant::Tertiary => (theme.surface_hover, theme.border, theme.text_primary),
            ButtonVariant::Destructive => (theme.danger, theme.danger, theme.selection_text),
            ButtonVariant::Success => (theme.success, theme.success, theme.selection_text),
        };
        let label = match state { ButtonState::Success => format!("✓ {}", label.into()), _ => label.into() };
        let mut style = centered_box_style(size.padding());
        style.size.height = creamui_core::layout::Dimension::Length(size.height());
        let text = RawText::new(label, foreground, size.font_size());
        let child: BoxedWidget = if state == ButtonState::Loading {
            let content = Style { display: creamui_core::layout::Display::Flex, flex_direction: creamui_core::layout::FlexDirection::Row, align_items: Some(AlignItems::Center), gap: creamui_core::layout::Size { width: LengthPercentage::Length(6.0), height: LengthPercentage::Length(6.0) }, ..Default::default() };
            Box::new(RawView::new(content).child(Box::new(RawSpinner::new(foreground).size(size.font_size()))).child(Box::new(text)))
        } else { Box::new(text) };
        let mut button = Self { inner: RawButton::new(style, on_click).background(background).border(border, size.border_width()).corner_radius(theme.button_radius).child(child) };
        if state == ButtonState::Loading { button = button.disabled(true); }
        button
    }

    /// Sized primary button with built-in loading and success presentations.
    pub fn state(theme: &Theme, size: ButtonSize, label: impl Into<String>, state: ButtonState, on_click: impl Fn() + 'static) -> Self {
        Self::styled(theme, ButtonVariant::Primary, size, label, state, on_click)
    }

    /// A neutral, still-clickable button for secondary actions.
    pub fn secondary(theme: &Theme, size: ButtonSize, label: impl Into<String>, on_click: impl Fn() + 'static) -> Self {
        Self::styled(theme, ButtonVariant::Secondary, size, label, ButtonState::Normal, on_click)
    }
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
        let text = RawText::new(label, theme.selection_text, 16.0);
        let inner = RawButton::new(style, on_click)
            .background(theme.accent)
            .corner_radius(theme.button_radius)
            .child(Box::new(text));
        Button { inner }
    }

    /// While `true`, the button reports no click handler and shows a "not
    /// allowed" cursor instead of a pointer. Purely behavioral — pass a
    /// theme-derived muted background/text color to [`Button::with_style`]
    /// (or build from [`RawButton`] directly, as this crate's examples do)
    /// for a dimmed disabled look to match.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
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
        inner = inner.corner_radius(theme.checkbox_radius);
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

pub struct Spinner { inner: RawSpinner }
impl Spinner { pub fn new(theme: &Theme) -> Self { Self { inner: RawSpinner::new(theme.accent) } } pub fn phase(mut self, phase: usize) -> Self { self.inner = self.inner.phase(phase); self } pub fn size(mut self, size: f32) -> Self { self.inner = self.inner.size(size); self } }
impl Widget for Spinner { fn style(&self) -> Style { self.inner.style() } fn paint(&self, painter: &mut dyn Painter, rect: Rect) { self.inner.paint(painter, rect) } }

/// A compact sliding boolean control, complementary to [`Checkbox`].
pub struct Switch { inner: RawSwitch }
impl Switch { pub fn new(theme: &Theme, checked: bool, on_click: impl Fn() + 'static) -> Self { Self { inner: RawSwitch::new(checked, theme.accent, theme.border_strong, theme.selection_text, on_click) } } }
impl Widget for Switch { fn style(&self) -> Style { self.inner.style() } fn paint(&self, painter: &mut dyn Painter, rect: Rect) { self.inner.paint(painter, rect) } fn on_click(&self) -> Option<Rc<dyn Fn()>> { self.inner.on_click() } fn cursor_icon(&self) -> Option<CursorIcon> { self.inner.cursor_icon() } }

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
            .border(theme.border, theme.input_border_width)
            .corner_radius(theme.input_radius)
            .selection_background(theme.selection_background)
            .selection_text_color(theme.selection_text);
        TextInput { inner }
    }

    /// Grayed-out text shown when the value is empty.
    pub fn placeholder(mut self, theme: &Theme, text: impl Into<String>) -> Self {
        self.inner = self.inner.placeholder(text, theme.text_disabled);
        self
    }

    /// Overrides the outline for validation states such as warning/error.
    pub fn border(mut self, color: creamui_theme::Color) -> Self {
        let width = self.inner.border_width;
        self.inner = self.inner.border(color, width);
        self
    }

    pub fn clipboard_enabled(mut self, enabled: bool) -> Self {
        self.inner = self.inner.clipboard_enabled(enabled);
        self
    }

    /// A `TextInput` whose value is read from and written back to a
    /// [`crate::TextController`], instead of a manually wired `value` +
    /// `on_change` pair. The controller must be a handle the app keeps
    /// alive across renders (created once, e.g. in `main`, the same way a
    /// `Signal` is) — cloning it here is cheap and shares the same
    /// underlying state.
    pub fn controlled(theme: &Theme, controller: &crate::TextController) -> Self {
        Self::controlled_with_style(theme, Self::default_style(), controller)
    }

    /// Same as [`TextInput::controlled`], but with full control over layout.
    pub fn controlled_with_style(
        theme: &Theme,
        style: Style,
        controller: &crate::TextController,
    ) -> Self {
        let set = controller.clone();
        Self::with_style(theme, style, controller.value(), move |next| {
            set.set_value(next)
        })
        .cursor(controller.cursor(), { let set = controller.clone(); move |cursor| set.set_cursor(cursor) })
        .selection(controller.selection(), { let set = controller.clone(); move |selection| set.set_selection(selection) })
    }

    pub fn cursor(mut self, cursor: usize, on_change: impl Fn(usize) + 'static) -> Self {
        self.inner = self.inner.cursor(cursor, on_change); self
    }
    pub fn selection(mut self, selection: crate::raw::TextSelection, on_change: impl Fn(crate::raw::TextSelection) + 'static) -> Self {
        self.inner = self.inner.selection(selection, on_change); self
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

    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        self.inner.on_drag()
    }

    fn on_drag_start(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        self.inner.on_drag_start()
    }

    fn paint_focused_overlay(&self, painter: &mut dyn Painter, rect: Rect, caret_visible: bool) {
        self.inner
            .paint_focused_overlay(painter, rect, caret_visible);
    }
}

/// A themed multi-line text editor. It shares `TextInput`'s controlled-value
/// API while choosing an editor-friendly 14px inset and surface treatment.
pub struct TextArea {
    inner: RawTextArea,
}

impl TextArea {
    pub fn default_style() -> Style {
        Style {
            size: creamui_core::layout::Size {
                width: creamui_core::layout::Dimension::Length(400.0),
                height: creamui_core::layout::Dimension::Length(240.0),
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
    pub fn with_style(
        theme: &Theme,
        style: Style,
        value: impl Into<String>,
        on_change: impl Fn(String) + 'static,
    ) -> Self {
        Self {
            inner: RawTextArea::new(style, value, 14.0, theme.text_primary, on_change)
                .background(theme.surface_elevated)
                .border(theme.border, theme.input_border_width)
                .corner_radius(theme.textarea_radius)
                .selection_background(theme.selection_background)
                .selection_text_color(theme.selection_text),
        }
    }
    pub fn placeholder(mut self, theme: &Theme, text: impl Into<String>) -> Self {
        self.inner = self.inner.placeholder(text, theme.text_disabled);
        self
    }

    pub fn alternating_line_background(mut self, color: creamui_theme::Color) -> Self {
        self.inner = self.inner.alternating_line_background(color);
        self
    }

    pub fn active_line_background(mut self, color: creamui_theme::Color) -> Self {
        self.inner = self.inner.active_line_background(color);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.inner = self.inner.corner_radius(radius);
        self
    }

    /// Sets the editor outline width. Use `0.0` for an edge-to-edge editor.
    pub fn border_width(mut self, width: f32) -> Self {
        if let Some(color) = self.inner.border_color {
            self.inner = self.inner.border(color, width);
        }
        self
    }

    pub fn cursor(mut self, cursor: usize, on_change: impl Fn(usize) + 'static) -> Self {
        self.inner = self.inner.cursor(cursor, on_change);
        self
    }

    pub fn selection(
        mut self,
        selection: crate::raw::TextSelection,
        on_change: impl Fn(crate::raw::TextSelection) + 'static,
    ) -> Self {
        self.inner = self.inner.selection(selection, on_change);
        self
    }

    pub fn selection_background(mut self, color: creamui_theme::Color) -> Self {
        self.inner = self.inner.selection_background(color);
        self
    }

    pub fn selection_text_color(mut self, color: creamui_theme::Color) -> Self {
        self.inner = self.inner.selection_text_color(color);
        self
    }

    pub fn on_ctrl_o(mut self, callback: impl Fn() + 'static) -> Self {
        self.inner = self.inner.on_ctrl_o(callback);
        self
    }

    pub fn clipboard_enabled(mut self, enabled: bool) -> Self {
        self.inner = self.inner.clipboard_enabled(enabled);
        self
    }

    /// When `true`, long lines wrap onto a new row at the editor's width
    /// instead of scrolling past it. Off by default.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.inner = self.inner.wrap(wrap);
        self
    }

    /// A `TextArea` whose value, cursor, and selection are all read from and
    /// written back to a [`crate::TextController`] — the multi-line
    /// counterpart of [`TextInput::controlled`], and the one place this
    /// pays off most: no more separately wiring `cursor`/`on_cursor_change`
    /// and `selection`/`on_selection_change` by hand. The controller must be
    /// a handle the app keeps alive across renders (created once, e.g. in
    /// `main`, the same way a `Signal` is).
    pub fn controlled(theme: &Theme, controller: &crate::TextController) -> Self {
        Self::controlled_with_style(theme, Self::default_style(), controller)
    }

    /// Same as [`TextArea::controlled`], but with full control over layout.
    pub fn controlled_with_style(
        theme: &Theme,
        style: Style,
        controller: &crate::TextController,
    ) -> Self {
        let value_set = controller.clone();
        let cursor_set = controller.clone();
        let selection_set = controller.clone();
        Self::with_style(theme, style, controller.value(), move |next| {
            value_set.set_value(next)
        })
        .cursor(controller.cursor(), move |next| cursor_set.set_cursor(next))
        .selection(controller.selection(), move |next| {
            selection_set.set_selection(next)
        })
    }
}

impl Widget for TextArea {
    fn style(&self) -> Style {
        self.inner.style()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect)
    }
    fn focusable(&self) -> bool {
        self.inner.focusable()
    }
    fn on_key(&self) -> Option<Rc<dyn Fn(KeyInput)>> {
        self.inner.on_key()
    }
    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        self.inner.on_drag()
    }
    fn on_drag_start(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        self.inner.on_drag_start()
    }
    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.inner.cursor_icon()
    }
    fn paint_focused_overlay(&self, painter: &mut dyn Painter, rect: Rect, caret_visible: bool) {
        self.inner
            .paint_focused_overlay(painter, rect, caret_visible)
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

/// A five-step type scale shared by [`Text`] and [`Heading`], in the spirit
/// of Tailwind's `text-xs`..`text-xl` steps or HTML's h5..h1 headings: `Xs`
/// is smallest, `Xl` is largest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    Xl,
    Lg,
    Md,
    Sm,
    Xs,
}

impl TextSize {
    /// Point size for body copy ([`Text`]).
    fn text_px(self) -> f32 {
        match self {
            TextSize::Xl => 20.0,
            TextSize::Lg => 17.0,
            TextSize::Md => 14.0,
            TextSize::Sm => 12.0,
            TextSize::Xs => 11.0,
        }
    }

    /// Point size for block headings ([`Heading`]), noticeably larger than
    /// the body scale at every step since a heading needs to read as a
    /// heading even at its smallest (`Xs`, an h5-equivalent).
    fn heading_px(self) -> f32 {
        match self {
            TextSize::Xl => 28.0, // h1
            TextSize::Lg => 22.0, // h2
            TextSize::Md => 18.0, // h3
            TextSize::Sm => 15.0, // h4
            TextSize::Xs => 13.0, // h5
        }
    }
}

/// Themed body text using the theme's primary text color. Centered by
/// default (handy for standalone labels and captions); call [`Text::align`]
/// for left/right-aligned copy.
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

    /// Sets the font size from the shared [`TextSize`] scale (`Md` matches
    /// the 14px default from [`Text::new`]).
    pub fn size(mut self, size: TextSize) -> Self {
        self.inner.font_size = size.text_px();
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

/// A themed block heading, the h1-h5 equivalent of [`Text`]: same five-step
/// [`TextSize`] scale, but left-aligned by default (a heading reads as a
/// block-level title, not a centered caption) and sized up so even its
/// smallest step (`Xs`, an h5) still reads as a heading next to body copy.
pub struct Heading {
    inner: RawText,
}

impl Heading {
    /// A heading at an explicit [`TextSize`] step.
    pub fn sized(theme: &Theme, size: TextSize, text: impl Into<String>) -> Self {
        Heading {
            inner: RawText::new(text, theme.text_primary, size.heading_px())
                .align(TextAlign::Start),
        }
    }

    /// Shorthand for [`Heading::sized`] with [`TextSize::Md`] (an
    /// h3-equivalent), a reasonable default for a section heading.
    pub fn new(theme: &Theme, text: impl Into<String>) -> Self {
        Self::sized(theme, TextSize::Md, text)
    }

    /// h1-equivalent: [`TextSize::Xl`].
    pub fn xl(theme: &Theme, text: impl Into<String>) -> Self {
        Self::sized(theme, TextSize::Xl, text)
    }

    /// h2-equivalent: [`TextSize::Lg`].
    pub fn lg(theme: &Theme, text: impl Into<String>) -> Self {
        Self::sized(theme, TextSize::Lg, text)
    }

    /// h3-equivalent: [`TextSize::Md`].
    pub fn md(theme: &Theme, text: impl Into<String>) -> Self {
        Self::sized(theme, TextSize::Md, text)
    }

    /// h4-equivalent: [`TextSize::Sm`].
    pub fn sm(theme: &Theme, text: impl Into<String>) -> Self {
        Self::sized(theme, TextSize::Sm, text)
    }

    /// h5-equivalent: [`TextSize::Xs`].
    pub fn xs(theme: &Theme, text: impl Into<String>) -> Self {
        Self::sized(theme, TextSize::Xs, text)
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.inner.align = align;
        self
    }

    /// Overrides the theme's primary text color, e.g. for an accent-colored
    /// heading.
    pub fn color(mut self, color: creamui_theme::Color) -> Self {
        self.inner.color = color;
        self
    }

    /// Gives the heading a layout style for width, margin, flex/grid
    /// placement, etc.
    pub fn style(mut self, style: Style) -> Self {
        self.inner.style = style;
        self
    }
}

impl Widget for Heading {
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

/// Shared visual tokens for application menu bars and popovers. Apps can
/// derive these from a theme and override individual colors without copying
/// menu geometry throughout their UI.
#[derive(Clone, Copy)]
pub struct MenuColors {
    pub bar: creamui_theme::Color,
    pub popup: creamui_theme::Color,
    pub active: creamui_theme::Color,
    pub border: creamui_theme::Color,
    pub text: creamui_theme::Color,
    pub muted_text: creamui_theme::Color,
    pub popup_radius: f32,
    pub item_radius: f32,
}

impl MenuColors {
    pub fn dark(theme: &Theme) -> Self {
        Self {
            bar: theme.surface,
            popup: theme.surface_hover,
            active: theme.border,
            border: theme.border_strong,
            text: theme.text_primary,
            muted_text: theme.text_secondary,
            popup_radius: theme.menu_radius,
            item_radius: theme.menu_item_radius,
        }
    }
}

/// A reusable application menu-bar surface. It owns only layout and paint;
/// applications compose [`MenuItem`] children and retain their own menu state.
pub struct MenuBar {
    inner: RawView,
}

impl MenuBar {
    pub fn new(colors: MenuColors, style: Style) -> Self {
        Self {
            inner: RawView::new(style).background(colors.bar),
        }
    }
    pub fn child(mut self, child: BoxedWidget) -> Self {
        self.inner = self.inner.child(child);
        self
    }
}

impl Widget for MenuBar {
    fn style(&self) -> Style {
        self.inner.style()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect)
    }
    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }
}

/// A compact menu popover surface. Give it an absolute-positioned `Style`
/// when it should float over application content.
pub struct MenuPopup {
    inner: RawView,
}

impl MenuPopup {
    pub fn new(colors: MenuColors, style: Style) -> Self {
        Self {
            inner: RawView::new(style)
                .background(colors.border)
                .corner_radius(colors.popup_radius),
        }
    }
    pub fn child(mut self, child: BoxedWidget) -> Self {
        self.inner = self.inner.child(child);
        self
    }
}

impl Widget for MenuPopup {
    fn style(&self) -> Style {
        self.inner.style()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect)
    }
    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }
}

/// A controlled clickable menu entry. `active` is supplied by the app so a
/// menu can be rebuilt reactively without hidden widget state.
pub struct MenuItem {
    inner: RawButton,
}

impl MenuItem {
    pub fn new(
        colors: MenuColors,
        style: Style,
        label: impl Into<String>,
        active: bool,
        on_click: impl Fn() + 'static,
    ) -> Self {
        let color = if active { colors.active } else { colors.popup };
        let text = RawText::new(
            label,
            if active {
                colors.text
            } else {
                colors.muted_text
            },
            13.0,
        )
        .align(TextAlign::Start)
        .layout_style(style.clone());
        Self {
            inner: RawButton::new(style, on_click)
                .background(color)
                .corner_radius(colors.item_radius)
                .child(Box::new(text)),
        }
    }
}

impl Widget for MenuItem {
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
    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.inner.cursor_icon()
    }
}

impl View {
    pub fn new(theme: &Theme, style: Style) -> Self {
        View {
            inner: RawView::new(style)
                .background(theme.surface_elevated)
                .corner_radius(theme.card_radius),
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

/// Shared visual tokens for [`Tabs`]/[`Tab`] and [`Sidebar`]/[`SidebarItem`],
/// the same pattern as [`MenuColors`]: derive from a theme, override
/// individual colors if needed.
#[derive(Clone, Copy)]
pub struct TabColors {
    pub background: creamui_theme::Color,
    pub active_background: creamui_theme::Color,
    pub hover_background: creamui_theme::Color,
    pub indicator: creamui_theme::Color,
    pub text: creamui_theme::Color,
    pub active_text: creamui_theme::Color,
    pub muted_text: creamui_theme::Color,
    pub radius: f32,
    pub container_radius: f32,
    pub selection: SelectionStyle,
    pub indicator_thickness: f32,
    pub gap: f32,
    pub icon_size: f32,
    pub icon_radius: f32,
    pub item_gap: f32,
    pub separator: creamui_theme::Color,
}

impl TabColors {
    /// Colours and geometry for a horizontal tab group.
    pub fn dark(theme: &Theme) -> Self {
        Self {
            background: theme.surface,
            active_background: theme.accent,
            hover_background: theme.surface_hover,
            indicator: theme.accent,
            text: theme.text_primary,
            active_text: theme.selection_text,
            muted_text: theme.text_secondary,
            radius: theme.tab_radius,
            container_radius: theme.tabs_radius,
            selection: theme.tab_selection,
            indicator_thickness: theme.indicator_thickness,
            gap: theme.tab_gap,
            icon_size: 0.0,
            icon_radius: 0.0,
            item_gap: 0.0,
            separator: theme.border,
        }
    }

    /// Colours and geometry for a vertical sidebar. Kept separate because a
    /// theme may intentionally give navigation a different silhouette.
    pub fn sidebar(theme: &Theme) -> Self {
        Self {
            // Navigation remains integrated with the app canvas; the
            // encapsulated content card is the elevated material.
            background: theme.surface,
            active_background: theme.accent,
            hover_background: theme.surface_elevated,
            indicator: theme.accent,
            text: theme.text_primary,
            active_text: theme.selection_text,
            muted_text: theme.text_secondary,
            radius: theme.sidebar_item_radius,
            container_radius: theme.sidebar_radius,
            selection: theme.sidebar_selection,
            indicator_thickness: theme.indicator_thickness,
            gap: theme.sidebar_gap,
            icon_size: theme.sidebar_icon_size,
            icon_radius: theme.sidebar_icon_radius,
            item_gap: theme.sidebar_item_gap,
            separator: theme.border,
        }
    }
}

/// A themed horizontal tab bar. Like [`MenuBar`], it owns only layout and
/// paint; applications compose [`Tab`] children and keep the selected index
/// in their own `Signal`. For a vertical rail, use [`Sidebar`] instead.
pub struct Tabs {
    inner: RawTabs,
}

impl Tabs {
    pub fn new(colors: TabColors, mut style: Style) -> Self {
        style.gap = creamui_core::layout::Size {
            width: LengthPercentage::Length(colors.gap),
            height: LengthPercentage::Length(colors.gap),
        };
        Self {
            inner: RawTabs::new(style)
                .background(colors.background)
                .corner_radius(colors.container_radius),
        }
    }

    pub fn child(mut self, child: BoxedWidget) -> Self {
        self.inner = self.inner.child(child);
        self
    }

    pub fn with_children(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.inner = self.inner.with_children(widgets);
        self
    }
}

impl Widget for Tabs {
    fn style(&self) -> Style {
        self.inner.style()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect)
    }
    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }
}

/// A controlled, themed tab entry for [`Tabs`]: an accent indicator bar along
/// the bottom edge while `active`, muted text otherwise. `active` is supplied
/// by the app so a tab bar can be rebuilt reactively with no hidden widget
/// state — the same pattern as [`MenuItem`].
pub struct Tab {
    inner: RawTab,
}

impl Tab {
    pub fn new(
        colors: TabColors,
        style: Style,
        label: impl Into<String>,
        active: bool,
        on_click: impl Fn() + 'static,
    ) -> Self {
        let text_color = if active && colors.selection == SelectionStyle::Filled {
            colors.active_text
        } else if active {
            colors.text
        } else {
            colors.muted_text
        };
        let text = RawText::new(label, text_color, 14.0)
            .align(TextAlign::Center)
            .layout_style(style.clone());
        let mut inner = RawTab::new(style, active, on_click)
            .corner_radius(colors.radius)
            .child(Box::new(text));
        match colors.selection {
            SelectionStyle::Filled => {
                if active {
                    inner = inner.background(colors.active_background);
                }
            }
            SelectionStyle::Indicator => {
                inner = inner.indicator(
                    TabIndicatorSide::Bottom,
                    colors.indicator,
                    colors.indicator_thickness,
                );
            }
        }
        Self { inner }
    }
}

impl Widget for Tab {
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
    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.inner.cursor_icon()
    }
}

/// A themed vertical nav rail. Same shape as [`Tabs`], but for the "sidebar
/// switches the visible view" pattern: applications compose [`SidebarItem`]
/// children and keep the selected index in their own `Signal`.
pub struct Sidebar {
    inner: RawSidebar,
}

impl Sidebar {
    pub fn new(colors: TabColors, mut style: Style) -> Self {
        style.gap = creamui_core::layout::Size {
            width: LengthPercentage::Length(colors.gap),
            height: LengthPercentage::Length(colors.gap),
        };
        Self {
            inner: RawSidebar::new(style)
                .background(colors.background)
                .corner_radius(colors.container_radius),
        }
    }

    pub fn child(mut self, child: BoxedWidget) -> Self {
        self.inner = self.inner.child(child);
        self
    }

    pub fn with_children(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.inner = self.inner.with_children(widgets);
        self
    }
}

impl Widget for Sidebar {
    fn style(&self) -> Style {
        self.inner.style()
    }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect)
    }
    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }
}

/// A controlled, themed entry for [`Sidebar`]: an accent indicator bar along
/// the left edge while `active`, muted text otherwise — the vertical
/// counterpart of [`Tab`].
pub struct SidebarItem {
    inner: RawTab,
}

/// A divider for groups inside a [`Sidebar`]. Its label is optional; when
/// supplied it becomes the small, muted section title used by settings apps.
pub struct SidebarSeparator {
    colors: TabColors,
    style: Style,
    label: Option<String>,
}

impl SidebarSeparator {
    pub fn new(colors: TabColors, style: Style) -> Self {
        Self { colors, style, label: None }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Widget for SidebarSeparator {
    fn style(&self) -> Style { self.style.clone() }
    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(label) = &self.label {
            painter.fill_text(rect, label, self.colors.muted_text, 11.0, TextAlign::Start);
        } else {
            let y = rect.y + rect.height / 2.0;
            painter.fill_rect(Rect { x: rect.x, y, width: rect.width, height: 1.0 }, self.colors.separator, 0.0);
        }
    }
}

impl SidebarItem {
    pub fn new(
        colors: TabColors,
        style: Style,
        label: impl Into<String>,
        active: bool,
        on_click: impl Fn() + 'static,
    ) -> Self {
        Self::build(colors, style, label.into(), None, active, false, None, on_click)
    }

    /// Adds a small rounded square icon before the item label.
    pub fn with_icon(
        colors: TabColors,
        style: Style,
        label: impl Into<String>,
        icon_color: creamui_theme::Color,
        active: bool,
        on_click: impl Fn() + 'static,
    ) -> Self {
        Self::build(colors, style, label.into(), Some(icon_color), active, false, None, on_click)
    }

    /// A sidebar item with a real pointer-hover state. Keep `hovered` in a
    /// [`creamui_reactive::Signal`] and pass its setter here; the renderer
    /// calls it on pointer entry/exit and the item is rebuilt with the soft
    /// hover background on the next reactive frame.
    pub fn with_hover(
        colors: TabColors,
        style: Style,
        label: impl Into<String>,
        active: bool,
        hovered: bool,
        on_hover: impl Fn(bool) + 'static,
        on_click: impl Fn() + 'static,
    ) -> Self {
        Self::build(
            colors,
            style,
            label.into(),
            None,
            active,
            hovered,
            Some(Rc::new(on_hover)),
            on_click,
        )
    }

    /// Combines a coloured icon with the reactive hover state.
    pub fn with_icon_hover(
        colors: TabColors,
        style: Style,
        label: impl Into<String>,
        icon_color: creamui_theme::Color,
        active: bool,
        hovered: bool,
        on_hover: impl Fn(bool) + 'static,
        on_click: impl Fn() + 'static,
    ) -> Self {
        Self::build(
            colors, style, label.into(), Some(icon_color), active, hovered,
            Some(Rc::new(on_hover)), on_click,
        )
    }

    fn build(
        colors: TabColors,
        style: Style,
        label: String,
        icon_color: Option<creamui_theme::Color>,
        active: bool,
        hovered: bool,
        on_hover: Option<Rc<dyn Fn(bool)>>,
        on_click: impl Fn() + 'static,
    ) -> Self {
        let text_color = if active && colors.selection == SelectionStyle::Filled {
            colors.active_text
        } else if active {
            colors.text
        } else {
            colors.muted_text
        };
        let text = RawText::new(label, text_color, 14.0).align(TextAlign::Start);
        let content: BoxedWidget = if let Some(icon_color) = icon_color {
            let content_style = Style {
                size: creamui_core::layout::Size {
                    width: creamui_core::layout::Dimension::Percent(1.0),
                    height: creamui_core::layout::Dimension::Percent(1.0),
                },
                display: creamui_core::layout::Display::Flex,
                flex_direction: creamui_core::layout::FlexDirection::Row,
                align_items: Some(AlignItems::Center),
                gap: creamui_core::layout::Size {
                    width: LengthPercentage::Length(colors.item_gap),
                    height: LengthPercentage::Length(colors.item_gap),
                },
                ..Default::default()
            };
            let icon_style = Style {
                size: creamui_core::layout::Size {
                    width: creamui_core::layout::Dimension::Length(colors.icon_size),
                    height: creamui_core::layout::Dimension::Length(colors.icon_size),
                },
                flex_shrink: 0.0,
                ..Default::default()
            };
            let text_style = Style { flex_grow: 1.0, ..Default::default() };
            Box::new(
                RawView::new(content_style)
                    .child(Box::new(RawView::new(icon_style).background(icon_color).corner_radius(colors.icon_radius)))
                    .child(Box::new(text.layout_style(text_style))),
            )
        } else {
            Box::new(text.layout_style(style.clone()))
        };
        let mut inner = RawTab::new(style, active, on_click)
            .corner_radius(colors.radius)
            .child(content);
        match colors.selection {
            SelectionStyle::Filled => {
                if active {
                    inner = inner.background(colors.active_background);
                } else if hovered {
                    inner = inner.background(colors.hover_background);
                }
            }
            SelectionStyle::Indicator => {
                inner = inner.indicator(
                    TabIndicatorSide::Left,
                    colors.indicator,
                    colors.indicator_thickness,
                );
            }
        }
        inner.on_hover = on_hover;
        Self { inner }
    }
}

impl Widget for SidebarItem {
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
    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.inner.cursor_icon()
    }
    fn on_hover(&self) -> Option<Rc<dyn Fn(bool)>> {
        self.inner.on_hover.clone()
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
