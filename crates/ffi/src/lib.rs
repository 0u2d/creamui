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
//! passed to exactly one consuming call: [`creamui_view_add_child`] (which
//! takes ownership of the child) or [`creamui_run`] (which takes ownership
//! of the root), or else freed with [`creamui_widget_free`].

use creamui_core::layout::{AlignItems, FlexDirection, JustifyContent, Style};
use creamui_core::{BoxedWidget, Size};
use creamui_reactive::Signal;
use creamui_theme::{Color, Theme};
use creamui_widgets::raw::{RawText, RawView};
use creamui_widgets::themed::{Button as ThemedButton, Text as ThemedText};
use std::ffi::{c_char, c_void, CStr};
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
}

impl WidgetKind {
    fn into_boxed(self) -> BoxedWidget {
        match self {
            WidgetKind::View(w) => Box::new(w),
            WidgetKind::Text(w) => Box::new(w),
            WidgetKind::ThemedText(w) => Box::new(w),
            WidgetKind::ThemedButton(w) => Box::new(w),
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

/// Creates a plain container widget that stacks children top-to-bottom.
#[no_mangle]
pub extern "C" fn creamui_view_new() -> *mut CWidget {
    let style = Style {
        display: creamui_core::layout::Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
        size: creamui_core::layout::Size {
            width: creamui_core::layout::Dimension::Percent(1.0),
            height: creamui_core::layout::Dimension::Percent(1.0),
        },
        ..Default::default()
    };
    let widget = CWidget(WidgetKind::View(RawView::new(style)));
    Box::into_raw(Box::new(widget))
}

/// Sets a view's background color. `view` must be a live pointer from
/// [`creamui_view_new`] that has not yet been consumed.
///
/// # Safety
/// `view` must be a valid, non-null pointer returned by
/// [`creamui_view_new`] and not yet passed to [`creamui_view_add_child`],
/// [`creamui_run`], or [`creamui_widget_free`].
#[no_mangle]
pub unsafe extern "C" fn creamui_view_set_background(view: *mut CWidget, color: CColor) {
    if view.is_null() {
        return;
    }
    if let WidgetKind::View(v) = &mut (*view).0 {
        v.background = Some(color.into());
    }
}

/// Attaches `child` to `view`, taking ownership of `child` (it must not be
/// used or freed again after this call).
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
    if let WidgetKind::View(v) = &mut (*view).0 {
        v.children.push(child.0.into_boxed());
    }
}

/// Creates a themed, single-line text label.
///
/// # Safety
/// `text` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn creamui_text_new(text: *const c_char, color: CColor, font_size: f32) -> *mut CWidget {
    let text = cstr_to_string(text);
    let widget = CWidget(WidgetKind::Text(RawText::new(text, color.into(), font_size)));
    Box::into_raw(Box::new(widget))
}

/// Creates a themed text label using the default dark theme's primary text
/// color.
///
/// # Safety
/// `text` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn creamui_themed_text_new(text: *const c_char) -> *mut CWidget {
    let text = cstr_to_string(text);
    let theme = Theme::dark();
    let widget = CWidget(WidgetKind::ThemedText(ThemedText::new(&theme, text)));
    Box::into_raw(Box::new(widget))
}

/// Creates a themed button labeled `text` using the default dark theme,
/// invoking `on_click(userdata)` on every click.
///
/// # Safety
/// `text` must be a valid NUL-terminated UTF-8 string. `on_click` must be
/// safe to call with `userdata` for as long as the returned widget (and any
/// tree it is attached to) is alive.
#[no_mangle]
pub unsafe extern "C" fn creamui_button_new(
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

    let theme = Theme::dark();
    let button = ThemedButton::new(&theme, text, move || {
        on_click(userdata.0);
    });
    Box::into_raw(Box::new(CWidget(WidgetKind::ThemedButton(button))))
}

/// Frees a widget subtree that was never attached via
/// [`creamui_view_add_child`] or [`creamui_run`].
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

type CBuildFn = extern "C" fn(width: f32, height: f32, userdata: *mut c_void) -> *mut CWidget;

/// Opens a window and runs the render loop until closed, calling `build`
/// once up front and again on every reactive change to construct the
/// widget tree for the current viewport size. Blocks until the window is
/// closed.
///
/// # Safety
/// `options.title` must be a valid NUL-terminated UTF-8 string for the
/// duration of this call. `build` must return a valid, non-null pointer
/// from one of this crate's widget `_new` functions each time it is
/// called, and must be safe to call with `userdata` for the lifetime of
/// this call.
#[no_mangle]
pub unsafe extern "C" fn creamui_run(options: CWindowOptions, background: CColor, build: CBuildFn, userdata: *mut c_void) {
    struct SendPtr(*mut c_void);
    unsafe impl Send for SendPtr {}
    let userdata = SendPtr(userdata);

    let window_options = creamui_render::WindowOptions {
        title: cstr_to_string(options.title),
        width: options.width,
        height: options.height,
        resizable: options.resizable != 0,
        decorations: options.decorations != 0,
        transparent: options.transparent != 0,
    };

    creamui_render::run(window_options, background.into(), move |size: Size| -> BoxedWidget {
        let raw = build(size.width, size.height, userdata.0);
        assert!(!raw.is_null(), "creamui_run: build callback returned a null widget");
        // SAFETY: `build` is contractually required to return an owned,
        // freshly-allocated widget pointer each call; we take ownership here.
        let widget = *Box::from_raw(raw);
        widget.0.into_boxed()
    });
}
