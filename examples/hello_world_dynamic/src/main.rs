//! The same themed counter as `examples/hello_world` — pixel-for-pixel the
//! same widget tree (title, counter text, click/theme-toggle buttons, a
//! checkbox row, a text input, a volume slider, and a scrollable list) — but
//! linked dynamically: this binary depends on zero CreamUI crates. It
//! `dlopen`s the `cdylib` built from `creamui-ffi` at runtime and talks to
//! it purely through the `#[repr(C)] extern "C"` ABI, exactly as a C, C++,
//! or any other FFI-capable language would.
//!
//! Compare `target/release/hello_world` (static, links the whole engine
//! into the binary) against this binary's size — this one stays tiny
//! because the engine lives in the shared `libcreamui.so` instead.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use libloading::{Library, Symbol};
use std::ffi::{c_char, c_void, CString};
use std::os::raw::c_int;

#[repr(C)]
#[derive(Clone, Copy)]
struct CColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

/// Mirrors `creamui_ffi::CTheme` field-for-field.
#[repr(C)]
#[derive(Clone, Copy)]
struct CTheme {
    surface: CColor,
    surface_elevated: CColor,
    surface_hover: CColor,
    accent: CColor,
    accent_hover: CColor,
    accent_pressed: CColor,
    text_primary: CColor,
    text_secondary: CColor,
    text_disabled: CColor,
    border: CColor,
    border_strong: CColor,
    danger: CColor,
    warning: CColor,
    success: CColor,
    radius_small: f32,
    radius_medium: f32,
    radius_large: f32,
    spacing_small: f32,
    spacing_medium: f32,
    spacing_large: f32,
}

/// Mirrors `creamui_ffi::CDimension` field-for-field. `kind`: `0` = auto,
/// `1` = length (logical px), `2` = percent (`0.0..=1.0`).
#[repr(C)]
#[derive(Clone, Copy)]
struct CDimension {
    kind: u8,
    value: f32,
}

const DIM_AUTO: CDimension = CDimension { kind: 0, value: 0.0 };
fn dim_len(value: f32) -> CDimension {
    CDimension { kind: 1, value }
}

const ALIGN_UNSET: u8 = 255;
const ALIGN_CENTER: u8 = 4;
const JUSTIFY_CENTER: u8 = 4;
const FLEX_DIRECTION_ROW: u8 = 0;
const FLEX_DIRECTION_COLUMN: u8 = 1;

/// Mirrors `creamui_ffi::CStyle` field-for-field.
#[repr(C)]
#[derive(Clone, Copy)]
struct CStyle {
    flex_direction: u8,
    justify_content: u8,
    align_items: u8,
    width: CDimension,
    height: CDimension,
    min_width: CDimension,
    min_height: CDimension,
    max_width: CDimension,
    max_height: CDimension,
    padding_left: f32,
    padding_right: f32,
    padding_top: f32,
    padding_bottom: f32,
    margin_left: CDimension,
    margin_right: CDimension,
    margin_top: CDimension,
    margin_bottom: CDimension,
    gap_row: f32,
    gap_column: f32,
    flex_grow: f32,
    flex_shrink: f32,
    flex_basis: CDimension,
}

impl CStyle {
    fn default_style() -> Self {
        CStyle {
            flex_direction: FLEX_DIRECTION_ROW,
            justify_content: ALIGN_UNSET,
            align_items: ALIGN_UNSET,
            width: DIM_AUTO,
            height: DIM_AUTO,
            min_width: DIM_AUTO,
            min_height: DIM_AUTO,
            max_width: DIM_AUTO,
            max_height: DIM_AUTO,
            padding_left: 0.0,
            padding_right: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            // Zero (not auto) to match `taffy::Style::default()` — an auto
            // margin on the main axis absorbs leftover flex space, which
            // would silently defeat the parent's `gap`/`justify_content`.
            margin_left: dim_len(0.0),
            margin_right: dim_len(0.0),
            margin_top: dim_len(0.0),
            margin_bottom: dim_len(0.0),
            gap_row: 0.0,
            gap_column: 0.0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: DIM_AUTO,
        }
    }

    /// Matches `creamui_widgets::layout::row`: a flex row with a fixed gap,
    /// vertically centered.
    fn row(gap: f32) -> Self {
        CStyle {
            flex_direction: FLEX_DIRECTION_ROW,
            align_items: ALIGN_CENTER,
            gap_row: gap,
            gap_column: gap,
            ..Self::default_style()
        }
    }
}

#[repr(C)]
struct CWindowOptions {
    title: *const c_char,
    width: u32,
    height: u32,
    resizable: c_int,
    decorations: c_int,
    transparent: c_int,
    backend: c_int,
}

type ViewNewStyledFn = unsafe extern "C" fn(CStyle) -> *mut c_void;
type ViewSetBackgroundFn = unsafe extern "C" fn(*mut c_void, CColor);
type ViewAddChildFn = unsafe extern "C" fn(*mut c_void, *mut c_void);
type ThemeFn = unsafe extern "C" fn() -> CTheme;
type ThemedTextNewFn = unsafe extern "C" fn(CTheme, *const c_char) -> *mut c_void;
type ThemedTextNewSizedFn = unsafe extern "C" fn(CTheme, *const c_char, f32) -> *mut c_void;
type ButtonNewFn = unsafe extern "C" fn(CTheme, *const c_char, extern "C" fn(*mut c_void), *mut c_void) -> *mut c_void;
type CheckboxNewFn =
    unsafe extern "C" fn(CTheme, c_int, extern "C" fn(*mut c_void), *mut c_void) -> *mut c_void;
type TextInputNewFn = unsafe extern "C" fn(
    CTheme,
    CStyle,
    *const c_char,
    extern "C" fn(*const c_char, *mut c_void),
    *mut c_void,
) -> *mut c_void;
type TextInputSetPlaceholderFn = unsafe extern "C" fn(CTheme, *mut c_void, *const c_char);
type SliderNewFn =
    unsafe extern "C" fn(CTheme, CStyle, f32, extern "C" fn(f32, *mut c_void), *mut c_void) -> *mut c_void;
type ScrollViewNewFn =
    unsafe extern "C" fn(CTheme, CStyle, f32, extern "C" fn(f32, *mut c_void), *mut c_void) -> *mut c_void;
type ScrollViewAddChildFn = unsafe extern "C" fn(*mut c_void, *mut c_void);
type SignalI32NewFn = unsafe extern "C" fn(i32) -> *mut c_void;
type SignalI32GetFn = unsafe extern "C" fn(*const c_void) -> i32;
type SignalI32SetFn = unsafe extern "C" fn(*const c_void, i32);
type SignalF32NewFn = unsafe extern "C" fn(f32) -> *mut c_void;
type SignalF32GetFn = unsafe extern "C" fn(*const c_void) -> f32;
type SignalF32SetFn = unsafe extern "C" fn(*const c_void, f32);
type SignalStringNewFn = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type SignalStringGetFn = unsafe extern "C" fn(*const c_void) -> *const c_char;
type SignalStringSetFn = unsafe extern "C" fn(*const c_void, *const c_char);
type OnWindowReadyFn = extern "C" fn(*mut c_void, *mut c_void);
type RunFn = unsafe extern "C" fn(
    CWindowOptions,
    CColor,
    extern "C" fn(f32, f32, *mut c_void) -> *mut c_void,
    Option<OnWindowReadyFn>,
    *mut c_void,
);

/// Every resolved symbol plus reactive state, bundled so the `build` and
/// callback functions (plain `extern "C" fn`s with no closure captures) can
/// reach them through one `userdata` pointer.
struct Api {
    view_new_styled: ViewNewStyledFn,
    view_set_background: ViewSetBackgroundFn,
    view_add_child: ViewAddChildFn,
    theme_dark: ThemeFn,
    theme_light: ThemeFn,
    themed_text_new: ThemedTextNewFn,
    themed_text_new_sized: ThemedTextNewSizedFn,
    themed_text_secondary_new: ThemedTextNewFn,
    button_new: ButtonNewFn,
    checkbox_new: CheckboxNewFn,
    text_input_new: TextInputNewFn,
    text_input_set_placeholder: TextInputSetPlaceholderFn,
    slider_new: SliderNewFn,
    scroll_view_new: ScrollViewNewFn,
    scroll_view_add_child: ScrollViewAddChildFn,
    signal_i32_get: SignalI32GetFn,
    signal_i32_set: SignalI32SetFn,
    signal_f32_get: SignalF32GetFn,
    signal_f32_set: SignalF32SetFn,
    signal_string_get: SignalStringGetFn,
    signal_string_set: SignalStringSetFn,
    count_signal: *mut c_void,
    /// `0` = dark, non-zero = light — reuses `creamui_signal_i32_*` rather
    /// than adding a bool-signal ABI just for this demo.
    theme_signal: *mut c_void,
    checked_signal: *mut c_void,
    name_signal: *mut c_void,
    volume_signal: *mut c_void,
    scroll_y_signal: *mut c_void,
}

fn current_theme(api: &Api) -> CTheme {
    let light = unsafe { (api.signal_i32_get)(api.theme_signal) } != 0;
    unsafe { if light { (api.theme_light)() } else { (api.theme_dark)() } }
}

extern "C" fn build(width: f32, height: f32, userdata: *mut c_void) -> *mut c_void {
    let api = unsafe { &*(userdata as *const Api) };
    let theme = current_theme(api);
    unsafe {
        let root_style = CStyle {
            flex_direction: FLEX_DIRECTION_COLUMN,
            justify_content: JUSTIFY_CENTER,
            align_items: ALIGN_CENTER,
            width: dim_len(width),
            height: dim_len(height),
            gap_row: theme.spacing_large,
            gap_column: 0.0,
            ..CStyle::default_style()
        };
        let root = (api.view_new_styled)(root_style);
        (api.view_set_background)(root, theme.surface);

        let title = CString::new("Hello, CreamUI!").unwrap();
        (api.view_add_child)(root, (api.themed_text_new_sized)(theme, title.as_ptr(), 28.0));

        let count = (api.signal_i32_get)(api.count_signal);
        let counter_label = CString::new(format!("Clicked {count} times")).unwrap();
        (api.view_add_child)(root, (api.themed_text_secondary_new)(theme, counter_label.as_ptr()));

        let click_label = CString::new("Click me").unwrap();
        (api.view_add_child)(root, (api.button_new)(theme, click_label.as_ptr(), on_click, userdata));

        let toggle_label = CString::new("Toggle theme").unwrap();
        (api.view_add_child)(root, (api.button_new)(theme, toggle_label.as_ptr(), on_toggle_theme, userdata));

        let checkbox_row = (api.view_new_styled)(CStyle::row(theme.spacing_small));
        let checked = (api.signal_i32_get)(api.checked_signal) != 0;
        (api.view_add_child)(checkbox_row, (api.checkbox_new)(theme, checked as c_int, on_toggle_checked, userdata));
        let sparkle_label = CString::new("Enable extra sparkle").unwrap();
        (api.view_add_child)(checkbox_row, (api.themed_text_new)(theme, sparkle_label.as_ptr()));
        (api.view_add_child)(root, checkbox_row);

        let name_style = CStyle { width: dim_len(200.0), height: dim_len(36.0), ..CStyle::default_style() };
        let name_cstr = CString::new((api.signal_string_get)(api.name_signal).to_owned_cstr()).unwrap();
        let input = (api.text_input_new)(theme, name_style, name_cstr.as_ptr(), on_name_change, userdata);
        let placeholder = CString::new("Your name").unwrap();
        (api.text_input_set_placeholder)(theme, input, placeholder.as_ptr());
        (api.view_add_child)(root, input);

        let volume = (api.signal_f32_get)(api.volume_signal);
        let volume_label = CString::new(format!("Volume: {:.0}%", volume * 100.0)).unwrap();
        (api.view_add_child)(root, (api.themed_text_secondary_new)(theme, volume_label.as_ptr()));

        let slider_style = CStyle { width: dim_len(160.0), height: dim_len(20.0), ..CStyle::default_style() };
        (api.view_add_child)(root, (api.slider_new)(theme, slider_style, volume, on_volume_change, userdata));

        let scroll_style = CStyle { width: dim_len(300.0), height: dim_len(100.0), ..CStyle::default_style() };
        let scroll_y = (api.signal_f32_get)(api.scroll_y_signal);
        let scroll_view = (api.scroll_view_new)(theme, scroll_style, scroll_y, on_scroll, userdata);
        for i in 0..10 {
            let item_label = CString::new(format!("Scrollable item {i}")).unwrap();
            (api.scroll_view_add_child)(scroll_view, (api.themed_text_new)(theme, item_label.as_ptr()));
        }
        (api.view_add_child)(root, scroll_view);

        root
    }
}

/// Owned-`CString` conversion for the borrowed pointer
/// [`creamui_signal_string_get`] returns, so it can be passed to a fresh
/// [`CString::new`] before the FFI call that produced it might invalidate it.
trait ToOwnedCstr {
    fn to_owned_cstr(self) -> String;
}
impl ToOwnedCstr for *const c_char {
    fn to_owned_cstr(self) -> String {
        if self.is_null() {
            return String::new();
        }
        unsafe { std::ffi::CStr::from_ptr(self) }.to_string_lossy().into_owned()
    }
}

extern "C" fn on_click(userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        let current = (api.signal_i32_get)(api.count_signal);
        (api.signal_i32_set)(api.count_signal, current + 1);
    }
}

extern "C" fn on_toggle_theme(userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        let light = (api.signal_i32_get)(api.theme_signal) != 0;
        (api.signal_i32_set)(api.theme_signal, if light { 0 } else { 1 });
    }
}

extern "C" fn on_toggle_checked(userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        let checked = (api.signal_i32_get)(api.checked_signal) != 0;
        (api.signal_i32_set)(api.checked_signal, (!checked) as c_int);
    }
}

extern "C" fn on_name_change(value: *const c_char, userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        (api.signal_string_set)(api.name_signal, value);
    }
}

extern "C" fn on_volume_change(value: f32, userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        (api.signal_f32_set)(api.volume_signal, value);
    }
}

extern "C" fn on_scroll(delta: f32, userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        let current = (api.signal_f32_get)(api.scroll_y_signal);
        (api.signal_f32_set)(api.scroll_y_signal, (current + delta).clamp(0.0, 300.0));
    }
}

fn cdylib_path() -> std::path::PathBuf {
    if let Ok(override_path) = std::env::var("CREAMUI_LIB_PATH") {
        return override_path.into();
    }
    let mut debug_dir = std::env::current_exe().expect("failed to resolve current executable path");
    debug_dir.pop(); // target/debug/ (or target/release/)
    let name = if cfg!(target_os = "macos") {
        "libcreamui.dylib"
    } else if cfg!(target_os = "windows") {
        "creamui.dll"
    } else {
        "libcreamui.so"
    };

    // `cargo build` uplifts the cdylib next to sibling binaries in this same
    // directory; if only `cargo test -p creamui-ffi` built it, it only
    // exists one level down in deps/ (unhashed, since cdylib filenames
    // aren't hash-suffixed) — check both.
    let sibling = debug_dir.join(name);
    if sibling.exists() {
        sibling
    } else {
        debug_dir.join("deps").join(name)
    }
}

fn main() {
    let lib_path = cdylib_path();
    let lib = unsafe { Library::new(&lib_path) }
        .unwrap_or_else(|e| panic!("failed to dlopen {lib_path:?}: {e} (set CREAMUI_LIB_PATH to override)"));

    unsafe {
        let view_new_styled: Symbol<ViewNewStyledFn> = lib.get(b"creamui_view_new_styled").unwrap();
        let view_set_background: Symbol<ViewSetBackgroundFn> = lib.get(b"creamui_view_set_background").unwrap();
        let view_add_child: Symbol<ViewAddChildFn> = lib.get(b"creamui_view_add_child").unwrap();
        let theme_dark: Symbol<ThemeFn> = lib.get(b"creamui_theme_dark").unwrap();
        let theme_light: Symbol<ThemeFn> = lib.get(b"creamui_theme_light").unwrap();
        let themed_text_new: Symbol<ThemedTextNewFn> = lib.get(b"creamui_themed_text_new").unwrap();
        let themed_text_new_sized: Symbol<ThemedTextNewSizedFn> = lib.get(b"creamui_themed_text_new_sized").unwrap();
        let themed_text_secondary_new: Symbol<ThemedTextNewFn> =
            lib.get(b"creamui_themed_text_secondary_new").unwrap();
        let button_new: Symbol<ButtonNewFn> = lib.get(b"creamui_button_new").unwrap();
        let checkbox_new: Symbol<CheckboxNewFn> = lib.get(b"creamui_checkbox_new").unwrap();
        let text_input_new: Symbol<TextInputNewFn> = lib.get(b"creamui_text_input_new").unwrap();
        let text_input_set_placeholder: Symbol<TextInputSetPlaceholderFn> =
            lib.get(b"creamui_text_input_set_placeholder").unwrap();
        let slider_new: Symbol<SliderNewFn> = lib.get(b"creamui_slider_new").unwrap();
        let scroll_view_new: Symbol<ScrollViewNewFn> = lib.get(b"creamui_scroll_view_new").unwrap();
        let scroll_view_add_child: Symbol<ScrollViewAddChildFn> =
            lib.get(b"creamui_scroll_view_add_child").unwrap();
        let signal_i32_new: Symbol<SignalI32NewFn> = lib.get(b"creamui_signal_i32_new").unwrap();
        let signal_i32_get: Symbol<SignalI32GetFn> = lib.get(b"creamui_signal_i32_get").unwrap();
        let signal_i32_set: Symbol<SignalI32SetFn> = lib.get(b"creamui_signal_i32_set").unwrap();
        let signal_f32_new: Symbol<SignalF32NewFn> = lib.get(b"creamui_signal_f32_new").unwrap();
        let signal_f32_get: Symbol<SignalF32GetFn> = lib.get(b"creamui_signal_f32_get").unwrap();
        let signal_f32_set: Symbol<SignalF32SetFn> = lib.get(b"creamui_signal_f32_set").unwrap();
        let signal_string_new: Symbol<SignalStringNewFn> = lib.get(b"creamui_signal_string_new").unwrap();
        let signal_string_get: Symbol<SignalStringGetFn> = lib.get(b"creamui_signal_string_get").unwrap();
        let signal_string_set: Symbol<SignalStringSetFn> = lib.get(b"creamui_signal_string_set").unwrap();
        let run: Symbol<RunFn> = lib.get(b"creamui_run").unwrap();

        let count_signal = signal_i32_new(0);
        let theme_signal = signal_i32_new(0);
        let checked_signal = signal_i32_new(0);
        let empty_name = CString::new("").unwrap();
        let name_signal = signal_string_new(empty_name.as_ptr());
        let volume_signal = signal_f32_new(0.5);
        let scroll_y_signal = signal_f32_new(0.0);

        // Leaked deliberately: this app runs for its whole lifetime, and
        // `build`/the click callbacks (plain `extern "C" fn`s) need a
        // `'static` pointer to reach these resolved symbols.
        let api: &'static Api = Box::leak(Box::new(Api {
            view_new_styled: *view_new_styled,
            view_set_background: *view_set_background,
            view_add_child: *view_add_child,
            theme_dark: *theme_dark,
            theme_light: *theme_light,
            themed_text_new: *themed_text_new,
            themed_text_new_sized: *themed_text_new_sized,
            themed_text_secondary_new: *themed_text_secondary_new,
            button_new: *button_new,
            checkbox_new: *checkbox_new,
            text_input_new: *text_input_new,
            text_input_set_placeholder: *text_input_set_placeholder,
            slider_new: *slider_new,
            scroll_view_new: *scroll_view_new,
            scroll_view_add_child: *scroll_view_add_child,
            signal_i32_get: *signal_i32_get,
            signal_i32_set: *signal_i32_set,
            signal_f32_get: *signal_f32_get,
            signal_f32_set: *signal_f32_set,
            signal_string_get: *signal_string_get,
            signal_string_set: *signal_string_set,
            count_signal,
            theme_signal,
            checked_signal,
            name_signal,
            volume_signal,
            scroll_y_signal,
        }));

        let title = CString::new("CreamUI — Hello World (dynamic)").unwrap();
        let options = CWindowOptions {
            title: title.as_ptr(),
            width: 480,
            height: 600,
            resizable: 1,
            decorations: 1,
            transparent: 0,
            backend: 1, // CUI_RENDER_BACKEND_GPU; override with CUI_OVERRIDE_RENDER_BACKEND=cpu
        };

        let initial_theme = (api.theme_dark)();
        run(options, initial_theme.surface, build, None, api as *const Api as *mut c_void);
    }
}
