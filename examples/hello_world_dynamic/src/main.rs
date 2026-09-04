//! The same counter as `examples/hello_world`, but linked dynamically: this
//! binary depends on zero CreamUI crates. It `dlopen`s the `cdylib` built
//! from `creamui-ffi` at runtime and talks to it purely through the
//! `#[repr(C)] extern "C"` ABI, exactly as a C, C++, or any other
//! FFI-capable language would.
//!
//! Also demonstrates the rest of the C ABI's desktop-shell surface: a
//! runtime theme toggle (via `creamui_theme_dark`/`creamui_theme_light`,
//! now that the ABI exposes theme tokens instead of hardcoding dark) and an
//! always-on-top toggle (via the `WindowHandle` handed to `on_window_ready`).
//!
//! Compare `target/release/hello_world` (static, links the whole engine
//! into the binary) against this binary's size — this one stays tiny
//! because the engine lives in the shared `libcreamui.so` instead.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use libloading::{Library, Symbol};
use std::cell::Cell;
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

#[repr(C)]
struct CWindowOptions {
    title: *const c_char,
    width: u32,
    height: u32,
    resizable: c_int,
    decorations: c_int,
    transparent: c_int,
}

type ViewNewFn = unsafe extern "C" fn() -> *mut c_void;
type ViewSetBackgroundFn = unsafe extern "C" fn(*mut c_void, CColor);
type ViewAddChildFn = unsafe extern "C" fn(*mut c_void, *mut c_void);
type ThemeFn = unsafe extern "C" fn() -> CTheme;
type ThemedTextNewFn = unsafe extern "C" fn(CTheme, *const c_char) -> *mut c_void;
type ButtonNewFn = unsafe extern "C" fn(CTheme, *const c_char, extern "C" fn(*mut c_void), *mut c_void) -> *mut c_void;
type SignalNewFn = unsafe extern "C" fn(i32) -> *mut c_void;
type SignalGetFn = unsafe extern "C" fn(*const c_void) -> i32;
type SignalSetFn = unsafe extern "C" fn(*const c_void, i32);
type WindowSetAlwaysOnTopFn = unsafe extern "C" fn(*const c_void, c_int);
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
    view_new: ViewNewFn,
    view_set_background: ViewSetBackgroundFn,
    view_add_child: ViewAddChildFn,
    theme_dark: ThemeFn,
    theme_light: ThemeFn,
    themed_text_new: ThemedTextNewFn,
    button_new: ButtonNewFn,
    signal_get: SignalGetFn,
    signal_set: SignalSetFn,
    window_set_always_on_top: WindowSetAlwaysOnTopFn,
    count_signal: *mut c_void,
    /// `0` = dark, non-zero = light — reuses `creamui_signal_i32_*` rather
    /// than adding a bool-signal ABI just for this demo.
    theme_signal: *mut c_void,
    /// Set once, from `on_window_ready`; read by `toggle_pin` afterward. A
    /// plain `Cell` is enough since the render loop is single-threaded.
    window_handle: Cell<*mut c_void>,
    pinned: Cell<bool>,
}

fn current_theme(api: &Api) -> CTheme {
    let light = unsafe { (api.signal_get)(api.theme_signal) } != 0;
    unsafe { if light { (api.theme_light)() } else { (api.theme_dark)() } }
}

extern "C" fn build(_width: f32, _height: f32, userdata: *mut c_void) -> *mut c_void {
    let api = unsafe { &*(userdata as *const Api) };
    let theme = current_theme(api);
    unsafe {
        let root = (api.view_new)();
        (api.view_set_background)(root, theme.surface);

        let title = CString::new("Hello from the dynamically-linked cdylib!").unwrap();
        (api.view_add_child)(root, (api.themed_text_new)(theme, title.as_ptr()));

        let count = (api.signal_get)(api.count_signal);
        let label = CString::new(format!("Clicked {count} times")).unwrap();
        (api.view_add_child)(root, (api.themed_text_new)(theme, label.as_ptr()));

        let button_label = CString::new("Click me").unwrap();
        let button = (api.button_new)(theme, button_label.as_ptr(), on_click, userdata);
        (api.view_add_child)(root, button);

        let toggle_label = CString::new("Toggle theme").unwrap();
        let toggle = (api.button_new)(theme, toggle_label.as_ptr(), on_toggle_theme, userdata);
        (api.view_add_child)(root, toggle);

        let pin_label = CString::new("Toggle always-on-top").unwrap();
        let pin = (api.button_new)(theme, pin_label.as_ptr(), on_toggle_pin, userdata);
        (api.view_add_child)(root, pin);

        root
    }
}

extern "C" fn on_click(userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        let current = (api.signal_get)(api.count_signal);
        (api.signal_set)(api.count_signal, current + 1);
    }
}

extern "C" fn on_toggle_theme(userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        let light = (api.signal_get)(api.theme_signal) != 0;
        (api.signal_set)(api.theme_signal, if light { 0 } else { 1 });
    }
}

extern "C" fn on_toggle_pin(userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    let handle = api.window_handle.get();
    if handle.is_null() {
        return;
    }
    let next_pinned = !api.pinned.get();
    api.pinned.set(next_pinned);
    unsafe {
        (api.window_set_always_on_top)(handle, next_pinned as c_int);
    }
}

/// Called once by `creamui_run`, as soon as the window exists — stashes the
/// window handle so `on_toggle_pin` can use it later.
extern "C" fn on_window_ready(handle: *mut c_void, userdata: *mut c_void) {
    let api = unsafe { &*(userdata as *const Api) };
    api.window_handle.set(handle);
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
        let view_new: Symbol<ViewNewFn> = lib.get(b"creamui_view_new").unwrap();
        let view_set_background: Symbol<ViewSetBackgroundFn> = lib.get(b"creamui_view_set_background").unwrap();
        let view_add_child: Symbol<ViewAddChildFn> = lib.get(b"creamui_view_add_child").unwrap();
        let theme_dark: Symbol<ThemeFn> = lib.get(b"creamui_theme_dark").unwrap();
        let theme_light: Symbol<ThemeFn> = lib.get(b"creamui_theme_light").unwrap();
        let themed_text_new: Symbol<ThemedTextNewFn> = lib.get(b"creamui_themed_text_new").unwrap();
        let button_new: Symbol<ButtonNewFn> = lib.get(b"creamui_button_new").unwrap();
        let signal_i32_new: Symbol<SignalNewFn> = lib.get(b"creamui_signal_i32_new").unwrap();
        let signal_get: Symbol<SignalGetFn> = lib.get(b"creamui_signal_i32_get").unwrap();
        let signal_set: Symbol<SignalSetFn> = lib.get(b"creamui_signal_i32_set").unwrap();
        let window_set_always_on_top: Symbol<WindowSetAlwaysOnTopFn> =
            lib.get(b"creamui_window_set_always_on_top").unwrap();
        let run: Symbol<RunFn> = lib.get(b"creamui_run").unwrap();

        let count_signal = signal_i32_new(0);
        let theme_signal = signal_i32_new(0);

        // Leaked deliberately: this app runs for its whole lifetime, and
        // `build`/the click callbacks (plain `extern "C" fn`s) need a
        // `'static` pointer to reach these resolved symbols.
        let api: &'static Api = Box::leak(Box::new(Api {
            view_new: *view_new,
            view_set_background: *view_set_background,
            view_add_child: *view_add_child,
            theme_dark: *theme_dark,
            theme_light: *theme_light,
            themed_text_new: *themed_text_new,
            button_new: *button_new,
            signal_get: *signal_get,
            signal_set: *signal_set,
            window_set_always_on_top: *window_set_always_on_top,
            count_signal,
            theme_signal,
            window_handle: Cell::new(std::ptr::null_mut()),
            pinned: Cell::new(false),
        }));

        let title = CString::new("CreamUI — Hello World (dynamic)").unwrap();
        let options = CWindowOptions {
            title: title.as_ptr(),
            width: 480,
            height: 360,
            resizable: 1,
            decorations: 1,
            transparent: 0,
        };

        let initial_theme = (api.theme_dark)();
        run(
            options,
            initial_theme.surface,
            build,
            Some(on_window_ready),
            api as *const Api as *mut c_void,
        );
    }
}
