//! ABI-stable C interface for consuming CreamUI as a shared library.
//!
//! This crate builds as a `cdylib`: an app can `dlopen`/link it and the
//! rest of the CreamUI engine (reactivity, layout, rendering) stays inside
//! the shared library, so multiple apps on a system can share one runtime
//! instead of statically bundling their own copy. Everything crossing the
//! boundary is either a plain `#[repr(C)]` value or an opaque pointer
//! (`*mut CWidget`) — never a Rust trait object or generic type — which is
//! what keeps the layout stable across compiler/library versions.
//!
//! All entry points are `extern "C"` and `#[no_mangle]`. Pointers returned
//! by `_new` functions are owned by the caller and must eventually be
//! passed to exactly one consuming call: [`creamui_view_add_child`] /
//! [`creamui_scroll_view_add_child`] (which take ownership of the child) or
//! [`creamui_run`] (which takes ownership of the root), or else freed with
//! [`creamui_widget_free`].

use creamui_core::layout::{
    AlignItems, Dimension, FlexDirection, JustifyContent, LengthPercentage, LengthPercentageAuto, Rect as LayoutRect,
    Size as LayoutSize, Style,
};
use creamui_core::{BoxedWidget, Size};
use creamui_reactive::Signal;
use creamui_render::WindowHandle;
use creamui_theme::{Color, Theme};
use creamui_widgets::raw::{RawText, RawView};
use creamui_widgets::themed::{
    Button as ThemedButton, Checkbox as ThemedCheckbox, ScrollView as ThemedScrollView, Slider as ThemedSlider,
    Text as ThemedText, TextInput as ThemedTextInput,
};
use std::ffi::{c_char, c_void, CStr, CString};
use std::os::raw::c_int;

/// Opaque handle to a reactive `i32` value.
///
/// Reading it (via [`creamui_signal_i32_get`]) while building a widget tree
/// inside a [`creamui_run`] `build` callback subscribes that render to
/// future writes, exactly like a native Rust `creamui_reactive::Signal` —
/// this is what lets a C click handler trigger a re-render.
pub struct CSignalI32(Signal<i32>);

/// Creates a reactive `i32` signal with an initial value.
#[no_mangle]
pub extern "C" fn creamui_signal_i32_new(initial: i32) -> *mut CSignalI32 {
    Box::into_raw(Box::new(CSignalI32(Signal::new(initial))))
}

/// Reads the current value, subscribing the enclosing render (if any) to
/// future [`creamui_signal_i32_set`] calls.
///
/// # Safety
/// `signal` must be a valid, non-null pointer from [`creamui_signal_i32_new`]
/// that has not been freed.
#[no_mangle]
pub unsafe extern "C" fn creamui_signal_i32_get(signal: *const CSignalI32) -> i32 {
    (*signal).0.get()
}

/// Writes a new value, triggering a reactive re-render in anything that
/// previously read this signal via [`creamui_signal_i32_get`].
///
/// # Safety
/// `signal` must be a valid, non-null pointer from [`creamui_signal_i32_new`]
/// that has not been freed.
#[no_mangle]
pub unsafe extern "C" fn creamui_signal_i32_set(signal: *const CSignalI32, value: i32) {
    (*signal).0.set(value);
}

/// Frees a signal created with [`creamui_signal_i32_new`].
///
/// # Safety
/// `signal` must be a valid, non-null, not-yet-freed pointer from
/// [`creamui_signal_i32_new`], and must outlive every [`creamui_run`] call
/// that might still read or write it (typically: free it only after
/// `creamui_run` returns).
#[no_mangle]
pub unsafe extern "C" fn creamui_signal_i32_free(signal: *mut CSignalI32) {
    if !signal.is_null() {
        drop(Box::from_raw(signal));
    }
}

/// A color in the C ABI: identical layout to `creamui_theme::Color`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<CColor> for Color {
    fn from(c: CColor) -> Self {
        Color::rgba(c.r, c.g, c.b, c.a)
    }
}

impl From<Color> for CColor {
    fn from(c: Color) -> Self {
        CColor { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}

/// A full theme-token set in the C ABI: identical field-for-field to
/// `creamui_theme::Theme`. A dynamically-linked app has no Rust-side
/// `Theme`, so this (plus [`creamui_theme_dark`]/[`creamui_theme_light`]) is
/// what lets it read the same semantic tokens themed widgets use, and pass
/// them back in to [`creamui_themed_text_new`]/[`creamui_button_new`]/etc.
/// so a runtime theme toggle works the same way the static example's does.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CTheme {
    pub surface: CColor,
    pub surface_elevated: CColor,
    pub surface_hover: CColor,

    pub accent: CColor,
    pub accent_hover: CColor,
    pub accent_pressed: CColor,

    pub text_primary: CColor,
    pub text_secondary: CColor,
    pub text_disabled: CColor,

    pub border: CColor,
    pub border_strong: CColor,

    pub danger: CColor,
    pub warning: CColor,
    pub success: CColor,

    pub radius_small: f32,
    pub radius_medium: f32,
    pub radius_large: f32,

    pub spacing_small: f32,
    pub spacing_medium: f32,
    pub spacing_large: f32,
}

impl From<Theme> for CTheme {
    fn from(t: Theme) -> Self {
        CTheme {
            surface: t.surface.into(),
            surface_elevated: t.surface_elevated.into(),
            surface_hover: t.surface_hover.into(),
            accent: t.accent.into(),
            accent_hover: t.accent_hover.into(),
            accent_pressed: t.accent_pressed.into(),
            text_primary: t.text_primary.into(),
            text_secondary: t.text_secondary.into(),
            text_disabled: t.text_disabled.into(),
            border: t.border.into(),
            border_strong: t.border_strong.into(),
            danger: t.danger.into(),
            warning: t.warning.into(),
            success: t.success.into(),
            radius_small: t.radius_small,
            radius_medium: t.radius_medium,
            radius_large: t.radius_large,
            spacing_small: t.spacing_small,
            spacing_medium: t.spacing_medium,
            spacing_large: t.spacing_large,
        }
    }
}

impl From<CTheme> for Theme {
    fn from(t: CTheme) -> Self {
        Theme {
            surface: t.surface.into(),
            surface_elevated: t.surface_elevated.into(),
            surface_hover: t.surface_hover.into(),
            accent: t.accent.into(),
            accent_hover: t.accent_hover.into(),
            accent_pressed: t.accent_pressed.into(),
            text_primary: t.text_primary.into(),
            text_secondary: t.text_secondary.into(),
            text_disabled: t.text_disabled.into(),
            border: t.border.into(),
            border_strong: t.border_strong.into(),
            danger: t.danger.into(),
            warning: t.warning.into(),
            success: t.success.into(),
            radius_small: t.radius_small,
            radius_medium: t.radius_medium,
            radius_large: t.radius_large,
            spacing_small: t.spacing_small,
            spacing_medium: t.spacing_medium,
            spacing_large: t.spacing_large,
        }
    }
}

/// Returns the bundled default dark theme's tokens.
#[no_mangle]
pub extern "C" fn creamui_theme_dark() -> CTheme {
    Theme::dark().into()
}

/// Returns the bundled default light theme's tokens.
#[no_mangle]
pub extern "C" fn creamui_theme_light() -> CTheme {
    Theme::light().into()
}

/// A length in the C ABI, tagged by `kind`:
/// `0` = auto (only meaningful for `size`/`min_size`/`max_size`/`flex_basis`
/// fields — treated as zero-length elsewhere), `1` = an absolute length in
/// logical pixels (`value`), `2` = a percentage of the containing block in
/// the `[0.0, 1.0]` range (`value`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CDimension {
    pub kind: u8,
    pub value: f32,
}

const DIMENSION_AUTO: u8 = 0;
const DIMENSION_LENGTH: u8 = 1;
const DIMENSION_PERCENT: u8 = 2;

impl CDimension {
    pub const AUTO: CDimension = CDimension { kind: DIMENSION_AUTO, value: 0.0 };

    fn to_dimension(self) -> Dimension {
        match self.kind {
            DIMENSION_LENGTH => Dimension::Length(self.value),
            DIMENSION_PERCENT => Dimension::Percent(self.value),
            _ => Dimension::Auto,
        }
    }

    fn to_length_percentage(self) -> LengthPercentage {
        match self.kind {
            DIMENSION_PERCENT => LengthPercentage::Percent(self.value),
            _ => LengthPercentage::Length(self.value),
        }
    }

    fn to_length_percentage_auto(self) -> LengthPercentageAuto {
        match self.kind {
            DIMENSION_LENGTH => LengthPercentageAuto::Length(self.value),
            DIMENSION_PERCENT => LengthPercentageAuto::Percent(self.value),
            _ => LengthPercentageAuto::Auto,
        }
    }
}

/// A sentinel for `justify_content`/`align_items` meaning "unset" (`None`),
/// distinct from any real alignment value.
const ALIGN_UNSET: u8 = 255;

fn decode_justify_content(code: u8) -> Option<JustifyContent> {
    Some(match code {
        0 => JustifyContent::Start,
        1 => JustifyContent::End,
        2 => JustifyContent::FlexStart,
        3 => JustifyContent::FlexEnd,
        4 => JustifyContent::Center,
        5 => JustifyContent::Stretch,
        6 => JustifyContent::SpaceBetween,
        7 => JustifyContent::SpaceAround,
        8 => JustifyContent::SpaceEvenly,
        _ => return None,
    })
}

fn decode_align_items(code: u8) -> Option<AlignItems> {
    Some(match code {
        0 => AlignItems::Start,
        1 => AlignItems::End,
        2 => AlignItems::FlexStart,
        3 => AlignItems::FlexEnd,
        4 => AlignItems::Center,
        5 => AlignItems::Stretch,
        9 => AlignItems::Baseline,
        _ => return None,
    })
}

fn decode_flex_direction(code: u8) -> FlexDirection {
    match code {
        1 => FlexDirection::Column,
        2 => FlexDirection::RowReverse,
        3 => FlexDirection::ColumnReverse,
        _ => FlexDirection::Row,
    }
}

/// Full flex-layout style control in the C ABI — the field-for-field subset
/// of `taffy::Style` (via `creamui_core::layout::Style`) that CreamUI's
/// widgets actually use. Build one with [`creamui_style_default`] (which
/// matches `Style::default()`) and override only the fields you need.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CStyle {
    /// `0` = row, `1` = column, `2` = row-reverse, `3` = column-reverse.
    pub flex_direction: u8,
    /// One of the `JustifyContent`/`AlignContent` codes documented on
    /// [`decode_justify_content`], or [`ALIGN_UNSET`] (255) for "unset".
    pub justify_content: u8,
    /// One of the `AlignItems` codes documented on [`decode_align_items`],
    /// or [`ALIGN_UNSET`] (255) for "unset".
    pub align_items: u8,
    pub width: CDimension,
    pub height: CDimension,
    pub min_width: CDimension,
    pub min_height: CDimension,
    pub max_width: CDimension,
    pub max_height: CDimension,
    pub padding_left: f32,
    pub padding_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    /// `kind` `0` (auto) collapses to `taffy`'s `Auto` margin.
    pub margin_left: CDimension,
    pub margin_right: CDimension,
    pub margin_top: CDimension,
    pub margin_bottom: CDimension,
    pub gap_row: f32,
    pub gap_column: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: CDimension,
}

/// Returns a [`CStyle`] matching `Style::default()`: row direction, no
/// forced alignment, auto size/margin, zero padding/gap, `flex_grow: 0`,
/// `flex_shrink: 1`, `flex_basis: auto`.
#[no_mangle]
pub extern "C" fn creamui_style_default() -> CStyle {
    CStyle {
        flex_direction: 0,
        justify_content: ALIGN_UNSET,
        align_items: ALIGN_UNSET,
        width: CDimension::AUTO,
        height: CDimension::AUTO,
        min_width: CDimension::AUTO,
        min_height: CDimension::AUTO,
        max_width: CDimension::AUTO,
        max_height: CDimension::AUTO,
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        margin_left: CDimension::AUTO,
        margin_right: CDimension::AUTO,
        margin_top: CDimension::AUTO,
        margin_bottom: CDimension::AUTO,
        gap_row: 0.0,
        gap_column: 0.0,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: CDimension::AUTO,
    }
}

impl From<CStyle> for Style {
    fn from(s: CStyle) -> Self {
        Style {
            display: creamui_core::layout::Display::Flex,
            flex_direction: decode_flex_direction(s.flex_direction),
            justify_content: decode_justify_content(s.justify_content),
            align_items: decode_align_items(s.align_items),
            size: LayoutSize { width: s.width.to_dimension(), height: s.height.to_dimension() },
            min_size: LayoutSize { width: s.min_width.to_dimension(), height: s.min_height.to_dimension() },
            max_size: LayoutSize { width: s.max_width.to_dimension(), height: s.max_height.to_dimension() },
            padding: LayoutRect {
                left: LengthPercentage::Length(s.padding_left),
                right: LengthPercentage::Length(s.padding_right),
                top: LengthPercentage::Length(s.padding_top),
                bottom: LengthPercentage::Length(s.padding_bottom),
            },
            margin: LayoutRect {
                left: s.margin_left.to_length_percentage_auto(),
                right: s.margin_right.to_length_percentage_auto(),
                top: s.margin_top.to_length_percentage_auto(),
                bottom: s.margin_bottom.to_length_percentage_auto(),
            },
            gap: LayoutSize {
                width: CDimension { kind: DIMENSION_LENGTH, value: s.gap_column }.to_length_percentage(),
                height: CDimension { kind: DIMENSION_LENGTH, value: s.gap_row }.to_length_percentage(),
            },
            flex_grow: s.flex_grow,
            flex_shrink: s.flex_shrink,
            flex_basis: s.flex_basis.to_dimension(),
            ..Default::default()
        }
    }
}

/// Window creation options in the C ABI. `title` must be a valid
/// NUL-terminated UTF-8 string for the duration of the [`creamui_run`] call.
#[repr(C)]
pub struct CWindowOptions {
    pub title: *const c_char,
    pub width: u32,
    pub height: u32,
    pub resizable: c_int,
    pub decorations: c_int,
    pub transparent: c_int,
}

enum WidgetKind {
    View(RawView),
    Text(RawText),
    ThemedText(ThemedText),
    ThemedButton(ThemedButton),
    ThemedCheckbox(ThemedCheckbox),
    ThemedTextInput(ThemedTextInput),
    ThemedSlider(ThemedSlider),
    ThemedScrollView(ThemedScrollView),
}

impl WidgetKind {
    fn into_boxed(self) -> BoxedWidget {
        match self {
            WidgetKind::View(w) => Box::new(w),
            WidgetKind::Text(w) => Box::new(w),
            WidgetKind::ThemedText(w) => Box::new(w),
            WidgetKind::ThemedButton(w) => Box::new(w),
            WidgetKind::ThemedCheckbox(w) => Box::new(w),
            WidgetKind::ThemedTextInput(w) => Box::new(w),
            WidgetKind::ThemedSlider(w) => Box::new(w),
            WidgetKind::ThemedScrollView(w) => Box::new(w),
        }
    }
}

/// Opaque handle to a not-yet-attached widget subtree.
pub struct CWidget(WidgetKind);

unsafe fn cstr_to_string(s: *const c_char) -> String {
    if s.is_null() {
        return String::new();
    }
    CStr::from_ptr(s).to_string_lossy().into_owned()
}

/// Returns the engine's version string (e.g. `"0.1.0"`), NUL-terminated,
/// valid for the lifetime of the process.
#[no_mangle]
pub extern "C" fn creamui_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Creates a plain container widget that stacks children top-to-bottom,
/// centered, filling its parent. For full layout control (arbitrary flex
/// direction, sizing, padding, etc.) use [`creamui_view_new_styled`]
/// instead.
#[no_mangle]
pub extern "C" fn creamui_view_new() -> *mut CWidget {
    let style = Style {
        display: creamui_core::layout::Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
        size: LayoutSize { width: Dimension::Percent(1.0), height: Dimension::Percent(1.0) },
        ..Default::default()
    };
    let widget = CWidget(WidgetKind::View(RawView::new(style)));
    Box::into_raw(Box::new(widget))
}

/// Creates a plain container widget with a caller-supplied [`CStyle`],
/// giving full control over flex direction, sizing, padding, margin, gap,
/// and flex-grow/shrink/basis — the same style surface `creamui_widgets`'
/// `View`/`RawView` expose natively.
#[no_mangle]
pub extern "C" fn creamui_view_new_styled(style: CStyle) -> *mut CWidget {
    let widget = CWidget(WidgetKind::View(RawView::new(style.into())));
    Box::into_raw(Box::new(widget))
}

/// Sets a view's background color. `view` must be a live pointer from
/// [`creamui_view_new`]/[`creamui_view_new_styled`] that has not yet been
/// consumed.
///
/// # Safety
/// `view` must be a valid, non-null pointer returned by
/// [`creamui_view_new`]/[`creamui_view_new_styled`] and not yet passed to
/// [`creamui_view_add_child`], [`creamui_run`], or [`creamui_widget_free`].
#[no_mangle]
pub unsafe extern "C" fn creamui_view_set_background(view: *mut CWidget, color: CColor) {
    if view.is_null() {
        return;
    }
    if let WidgetKind::View(v) = &mut (*view).0 {
        v.background = Some(color.into());
    }
}

/// Sets a view's corner radius, in logical pixels.
///
/// # Safety
/// Same contract as [`creamui_view_set_background`].
#[no_mangle]
pub unsafe extern "C" fn creamui_view_set_corner_radius(view: *mut CWidget, radius: f32) {
    if view.is_null() {
        return;
    }
    if let WidgetKind::View(v) = &mut (*view).0 {
        v.corner_radius = radius;
    }
}

/// Attaches `child` to `view`, taking ownership of `child` (it must not be
/// used or freed again after this call). Works for both
/// [`creamui_view_new`]/[`creamui_view_new_styled`] and
/// [`creamui_scroll_view_new`] parents.
///
/// # Safety
/// `view` and `child` must be valid, non-null, not-yet-consumed pointers
/// from this crate's `_new` functions, and must not alias each other.
#[no_mangle]
pub unsafe extern "C" fn creamui_view_add_child(view: *mut CWidget, child: *mut CWidget) {
    if view.is_null() || child.is_null() {
        return;
    }
    let child = *Box::from_raw(child);
    match &mut (*view).0 {
        WidgetKind::View(v) => v.children.push(child.0.into_boxed()),
        WidgetKind::ThemedScrollView(v) => push_scroll_view_child(v, child.0.into_boxed()),
        _ => {}
    }
}

/// `ThemedScrollView`'s children live behind a consuming builder method
/// (`.child()`), not a public field — this takes ownership out of the `&mut`
/// reference via a throwaway placeholder so both `creamui_view_add_child`
/// and [`creamui_scroll_view_add_child`] can share this one code path.
fn push_scroll_view_child(view: &mut ThemedScrollView, child: BoxedWidget) {
    let taken = std::mem::replace(view, ThemedScrollView::new(&Theme::dark(), Style::default(), 0.0, |_| {}));
    *view = taken.child(child);
}

/// Attaches `child` to a scroll view created by [`creamui_scroll_view_new`],
/// taking ownership of `child`.
///
/// # Safety
/// Same contract as [`creamui_view_add_child`]; `view` must specifically be
/// a not-yet-consumed pointer from [`creamui_scroll_view_new`].
#[no_mangle]
pub unsafe extern "C" fn creamui_scroll_view_add_child(view: *mut CWidget, child: *mut CWidget) {
    if view.is_null() || child.is_null() {
        return;
    }
    let child = *Box::from_raw(child);
    if let WidgetKind::ThemedScrollView(v) = &mut (*view).0 {
        push_scroll_view_child(v, child.0.into_boxed());
    }
}

/// Creates an unthemed, single-line text label with an explicit color and
/// size.
///
/// # Safety
/// `text` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn creamui_text_new(text: *const c_char, color: CColor, font_size: f32) -> *mut CWidget {
    let text = cstr_to_string(text);
    let widget = CWidget(WidgetKind::Text(RawText::new(text, color.into(), font_size)));
    Box::into_raw(Box::new(widget))
}

/// Creates a themed text label using `theme`'s primary text color (get one
/// from [`creamui_theme_dark`]/[`creamui_theme_light`]).
///
/// # Safety
/// `text` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn creamui_themed_text_new(theme: CTheme, text: *const c_char) -> *mut CWidget {
    let text = cstr_to_string(text);
    let theme: Theme = theme.into();
    let widget = CWidget(WidgetKind::ThemedText(ThemedText::new(&theme, text)));
    Box::into_raw(Box::new(widget))
}

/// Creates a themed button labeled `text`, invoking `on_click(userdata)` on
/// every click.
///
/// # Safety
/// `text` must be a valid NUL-terminated UTF-8 string. `on_click` must be
/// safe to call with `userdata` for as long as the returned widget (and any
/// tree it is attached to) is alive.
#[no_mangle]
pub unsafe extern "C" fn creamui_button_new(
    theme: CTheme,
    text: *const c_char,
    on_click: extern "C" fn(*mut c_void),
    userdata: *mut c_void,
) -> *mut CWidget {
    let text = cstr_to_string(text);
    // SAFETY contract above: the caller guarantees `userdata` stays valid
    // and `on_click` stays callable for as long as this widget tree lives.
    struct SendPtr(*mut c_void);
    unsafe impl Send for SendPtr {}
    let userdata = SendPtr(userdata);

    let theme: Theme = theme.into();
    let button = ThemedButton::new(&theme, text, move || {
        on_click(userdata.0);
    });
    Box::into_raw(Box::new(CWidget(WidgetKind::ThemedButton(button))))
}

/// Creates a themed checkbox, invoking `on_click(userdata)` on every click
/// (same "caller owns the checked state" pattern as the native `Checkbox`:
/// toggle your own state in `on_click` and pass the new value back in on
/// the next `build` call).
///
/// # Safety
/// `on_click` must be safe to call with `userdata` for as long as the
/// returned widget (and any tree it is attached to) is alive.
#[no_mangle]
pub unsafe extern "C" fn creamui_checkbox_new(
    theme: CTheme,
    checked: c_int,
    on_click: extern "C" fn(*mut c_void),
    userdata: *mut c_void,
) -> *mut CWidget {
    struct SendPtr(*mut c_void);
    unsafe impl Send for SendPtr {}
    let userdata = SendPtr(userdata);

    let theme: Theme = theme.into();
    let checkbox = ThemedCheckbox::new(&theme, checked != 0, move || {
        on_click(userdata.0);
    });
    Box::into_raw(Box::new(CWidget(WidgetKind::ThemedCheckbox(checkbox))))
}

/// Creates a themed single-line text input with a caller-supplied [`CStyle`]
/// (e.g. its width/height). `on_change(new_value, userdata)` fires on every
/// keystroke with a NUL-terminated UTF-8 string owned by the callee — valid
/// only for the duration of the call.
///
/// # Safety
/// `value` must be a valid NUL-terminated UTF-8 string. `on_change` must be
/// safe to call with `userdata` for as long as the returned widget (and any
/// tree it is attached to) is alive.
#[no_mangle]
pub unsafe extern "C" fn creamui_text_input_new(
    theme: CTheme,
    style: CStyle,
    value: *const c_char,
    on_change: extern "C" fn(*const c_char, *mut c_void),
    userdata: *mut c_void,
) -> *mut CWidget {
    struct SendPtr(*mut c_void);
    unsafe impl Send for SendPtr {}
    let userdata = SendPtr(userdata);

    let value = cstr_to_string(value);
    let theme_owned: Theme = theme.into();
    let inner = ThemedTextInput::with_style(&theme_owned, style.into(), value, move |next: String| {
        // CString::new fails only on interior NULs, which a text input's
        // keystroke-built value can never contain (Key::Char never yields
        // '\0'), so this is infallible in practice.
        if let Ok(c_next) = CString::new(next) {
            on_change(c_next.as_ptr(), userdata.0);
        }
    });
    Box::into_raw(Box::new(CWidget(WidgetKind::ThemedTextInput(inner))))
}

/// Sets the placeholder text (and its themed disabled-text color) shown
/// when a text input's value is empty.
///
/// # Safety
/// `input` must be a valid, non-null, not-yet-consumed pointer from
/// [`creamui_text_input_new`]. `text` must be a valid NUL-terminated UTF-8
/// string.
#[no_mangle]
pub unsafe extern "C" fn creamui_text_input_set_placeholder(theme: CTheme, input: *mut CWidget, text: *const c_char) {
    if input.is_null() {
        return;
    }
    let text = cstr_to_string(text);
    let theme: Theme = theme.into();
    if let WidgetKind::ThemedTextInput(w) = &mut (*input).0 {
        let taken = std::mem::replace(w, ThemedTextInput::new(&theme, String::new(), |_| {}));
        *w = taken.placeholder(&theme, text);
    }
}

/// Creates a themed horizontal slider with a caller-supplied [`CStyle`].
/// `on_change(value, userdata)` fires with the new `0.0..=1.0` value as the
/// handle is dragged.
///
/// # Safety
/// `on_change` must be safe to call with `userdata` for as long as the
/// returned widget (and any tree it is attached to) is alive.
#[no_mangle]
pub unsafe extern "C" fn creamui_slider_new(
    theme: CTheme,
    style: CStyle,
    value: f32,
    on_change: extern "C" fn(f32, *mut c_void),
    userdata: *mut c_void,
) -> *mut CWidget {
    struct SendPtr(*mut c_void);
    unsafe impl Send for SendPtr {}
    let userdata = SendPtr(userdata);

    let theme: Theme = theme.into();
    let slider = ThemedSlider::with_style(&theme, style.into(), value, move |next| {
        on_change(next, userdata.0);
    });
    Box::into_raw(Box::new(CWidget(WidgetKind::ThemedSlider(slider))))
}

/// Creates a themed vertically-scrollable container with a caller-supplied
/// [`CStyle`] (typically a fixed `width`/`height` viewport). Attach children
/// with [`creamui_scroll_view_add_child`]. `on_scroll(delta_y, userdata)`
/// fires on every wheel event over the view; the caller owns and clamps the
/// scroll offset, same as the native `ScrollView`.
///
/// # Safety
/// `on_scroll` must be safe to call with `userdata` for as long as the
/// returned widget (and any tree it is attached to) is alive.
#[no_mangle]
pub unsafe extern "C" fn creamui_scroll_view_new(
    theme: CTheme,
    style: CStyle,
    scroll_y: f32,
    on_scroll: extern "C" fn(f32, *mut c_void),
    userdata: *mut c_void,
) -> *mut CWidget {
    struct SendPtr(*mut c_void);
    unsafe impl Send for SendPtr {}
    let userdata = SendPtr(userdata);

    let theme: Theme = theme.into();
    let scroll_view = ThemedScrollView::new(&theme, style.into(), scroll_y, move |delta| {
        on_scroll(delta, userdata.0);
    });
    Box::into_raw(Box::new(CWidget(WidgetKind::ThemedScrollView(scroll_view))))
}

/// Frees a widget subtree that was never attached via
/// [`creamui_view_add_child`], [`creamui_scroll_view_add_child`], or
/// [`creamui_run`].
///
/// # Safety
/// `widget` must be a valid, non-null, not-yet-consumed pointer from one of
/// this crate's `_new` functions, and must not be used again afterward.
#[no_mangle]
pub unsafe extern "C" fn creamui_widget_free(widget: *mut CWidget) {
    if !widget.is_null() {
        drop(Box::from_raw(widget));
    }
}

/// Opaque handle for issuing window-level operations (resize, move,
/// always-on-top) from C, e.g. from a click handler. Valid for the lifetime
/// of the [`creamui_run`] call that produced it via `on_window_ready`; do
/// not use after `creamui_run` returns.
pub struct CWindowHandle(WindowHandle);

/// Requests a new logical-pixel window size.
///
/// # Safety
/// `handle` must be a valid, non-null pointer from a [`creamui_run`]
/// `on_window_ready` callback, still within that `creamui_run` call.
#[no_mangle]
pub unsafe extern "C" fn creamui_window_resize(handle: *const CWindowHandle, width: u32, height: u32) {
    if handle.is_null() {
        return;
    }
    (*handle).0.resize(width, height);
}

/// Moves the window's top-left corner to a logical-pixel screen position.
///
/// # Safety
/// Same contract as [`creamui_window_resize`].
#[no_mangle]
pub unsafe extern "C" fn creamui_window_set_position(handle: *const CWindowHandle, x: i32, y: i32) {
    if handle.is_null() {
        return;
    }
    (*handle).0.set_position(x, y);
}

/// Pins (`enabled != 0`) or unpins the window above all others.
///
/// # Safety
/// Same contract as [`creamui_window_resize`].
#[no_mangle]
pub unsafe extern "C" fn creamui_window_set_always_on_top(handle: *const CWindowHandle, enabled: c_int) {
    if handle.is_null() {
        return;
    }
    (*handle).0.set_always_on_top(enabled != 0);
}

/// Frees a handle obtained from a [`creamui_run`] `on_window_ready`
/// callback. Optional — the handle is also cleaned up when `creamui_run`
/// returns — but calling this lets an app stop holding onto it earlier.
///
/// # Safety
/// `handle` must be a valid, non-null, not-yet-freed pointer produced by a
/// `creamui_run` `on_window_ready` callback.
#[no_mangle]
pub unsafe extern "C" fn creamui_window_handle_free(handle: *mut CWindowHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

type CBuildFn = extern "C" fn(width: f32, height: f32, userdata: *mut c_void) -> *mut CWidget;
type CWindowReadyFn = extern "C" fn(handle: *mut CWindowHandle, userdata: *mut c_void);

/// Opens a window and runs the render loop until closed, calling `build`
/// once up front and again on every reactive change to construct the
/// widget tree for the current viewport size. If `on_window_ready` is
/// non-null, it is called exactly once, as soon as the window exists, with
/// a heap-allocated [`CWindowHandle`] the app can stash in its own
/// `userdata` and use later (e.g. from a click handler) to resize, move, or
/// pin the window — see [`creamui_window_resize`] and friends. Blocks until
/// the window is closed.
///
/// # Safety
/// `options.title` must be a valid NUL-terminated UTF-8 string for the
/// duration of this call. `build` must return a valid, non-null pointer
/// from one of this crate's widget `_new` functions each time it is
/// called, and must be safe to call with `userdata` for the lifetime of
/// this call. `on_window_ready`, if non-null, must likewise be safe to call
/// with `userdata`.
#[no_mangle]
pub unsafe extern "C" fn creamui_run(
    options: CWindowOptions,
    background: CColor,
    build: CBuildFn,
    on_window_ready: Option<CWindowReadyFn>,
    userdata: *mut c_void,
) {
    struct SendPtr(*mut c_void);
    unsafe impl Send for SendPtr {}
    let build_userdata = SendPtr(userdata);
    let ready_userdata = SendPtr(userdata);

    let window_options = creamui_render::WindowOptions {
        title: cstr_to_string(options.title),
        width: options.width,
        height: options.height,
        resizable: options.resizable != 0,
        decorations: options.decorations != 0,
        transparent: options.transparent != 0,
    };

    creamui_render::run(
        window_options,
        background.into(),
        move |handle: WindowHandle| {
            if let Some(on_ready) = on_window_ready {
                let boxed = Box::into_raw(Box::new(CWindowHandle(handle)));
                on_ready(boxed, ready_userdata.0);
            }
        },
        move |size: Size| -> BoxedWidget {
            let raw = build(size.width, size.height, build_userdata.0);
            assert!(!raw.is_null(), "creamui_run: build callback returned a null widget");
            // SAFETY: `build` is contractually required to return an owned,
            // freshly-allocated widget pointer each call; we take ownership here.
            let widget = *Box::from_raw(raw);
            widget.0.into_boxed()
        },
    );
}
