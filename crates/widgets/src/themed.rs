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
    RawButton, RawCheckbox, RawScrollView, RawSidebar, RawSlider, RawTab, RawTabs, RawText,
    RawTextArea, RawTextInput, RawView, TabIndicatorSide,
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

    pub fn clipboard_enabled(mut self, enabled: bool) -> Self {
        self.inner = self.inner.clipboard_enabled(enabled);
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
                .border(theme.border, 1.0)
                .corner_radius(theme.radius_medium)
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
                .corner_radius(4.0),
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
                .corner_radius(3.0)
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

/// Shared visual tokens for [`Tabs`]/[`Tab`] and [`Sidebar`]/[`SidebarItem`],
/// the same pattern as [`MenuColors`]: derive from a theme, override
/// individual colors if needed.
#[derive(Clone, Copy)]
pub struct TabColors {
    pub background: creamui_theme::Color,
    pub active_background: creamui_theme::Color,
    pub indicator: creamui_theme::Color,
    pub text: creamui_theme::Color,
    pub muted_text: creamui_theme::Color,
    pub radius: f32,
}

impl TabColors {
    pub fn dark(theme: &Theme) -> Self {
        Self {
            background: theme.surface,
            active_background: theme.surface_hover,
            indicator: theme.accent,
            text: theme.text_primary,
            muted_text: theme.text_secondary,
            radius: theme.radius_small,
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
    pub fn new(colors: TabColors, style: Style) -> Self {
        Self {
            inner: RawTabs::new(style).background(colors.background),
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
        let text_color = if active { colors.indicator } else { colors.muted_text };
        let text = RawText::new(label, text_color, 14.0)
            .align(TextAlign::Center)
            .layout_style(style.clone());
        Self {
            inner: RawTab::new(style, active, on_click)
                .indicator(TabIndicatorSide::Bottom, colors.indicator, 2.5)
                .child(Box::new(text)),
        }
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
    pub fn new(colors: TabColors, style: Style) -> Self {
        Self {
            inner: RawSidebar::new(style).background(colors.background),
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

impl SidebarItem {
    pub fn new(
        colors: TabColors,
        style: Style,
        label: impl Into<String>,
        active: bool,
        on_click: impl Fn() + 'static,
    ) -> Self {
        // No background fill: a full-row block popping in and out on every
        // click reads as the whole row changing size, not just selection.
        // Only the text color and a thin indicator bar change, the same
        // restrained treatment as `Tab`'s underline.
        let text_color = if active { colors.text } else { colors.muted_text };
        let text = RawText::new(label, text_color, 14.0)
            .align(TextAlign::Start)
            .layout_style(style.clone());
        let mut inner = RawTab::new(style, active, on_click).child(Box::new(text));
        if active {
            inner = inner.indicator(TabIndicatorSide::Left, colors.indicator, 3.0);
        }
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
