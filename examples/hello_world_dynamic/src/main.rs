//! The same counter as `examples/hello_world`, but linked dynamically: this
//! binary depends on zero CreamUI crates. It `dlopen`s the `cdylib` built
//! from `creamui-ffi` at runtime and talks to it purely through the
//! `#[repr(C)] extern "C"` ABI, exactly as a C, C++, or any other
//! FFI-capable language would.
//!
//! Compare `target/release/hello_world` (static, links the whole engine
//! into the binary) against this binary's size — this one stays tiny
//! because the engine lives in the shared `libcreamui.so` instead.

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
type ThemedTextNewFn = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type ButtonNewFn = unsafe extern "C" fn(*const c_char, extern "C" fn(*mut c_void), *mut c_void) -> *mut c_void;
type SignalGetFn = unsafe extern "C" fn(*const c_void) -> i32;
type SignalSetFn = unsafe extern "C" fn(*const c_void, i32);
type RunFn = unsafe extern "C" fn(CWindowOptions, CColor, extern "C" fn(f32, f32, *mut c_void) -> *mut c_void, *mut c_void);

/// Every resolved symbol plus the counter signal, bundled so the `build`
/// and click callbacks (plain `extern "C" fn`s with no closure captures)
/// can reach them through one `userdata` pointer.
struct Api {
    view_new: ViewNewFn,
    view_set_background: ViewSetBackgroundFn,
    view_add_child: ViewAddChildFn,
    themed_text_new: ThemedTextNewFn,
    button_new: ButtonNewFn,
    signal_get: SignalGetFn,
    signal_set: SignalSetFn,
    count_signal: *mut c_void,
}

/// Mirrors `creamui_theme::Theme::dark().surface` — the dynamic ABI has no
/// theme-token access yet (see ROADMAP.md), so this example hardcodes it.
const DARK_SURFACE: CColor = CColor { r: 0x1a, g: 0x1b, b: 0x1e, a: 255 };

extern "C" fn build(_width: f32, _height: f32, userdata: *mut c_void) -> *mut c_void {
    let api = unsafe { &*(userdata as *const Api) };
    unsafe {
        let root = (api.view_new)();
        (api.view_set_background)(root, DARK_SURFACE);

        let title = CString::new("Hello from the dynamically-linked cdylib!").unwrap();
        (api.view_add_child)(root, (api.themed_text_new)(title.as_ptr()));

        let count = (api.signal_get)(api.count_signal);
        let label = CString::new(format!("Clicked {count} times")).unwrap();
        (api.view_add_child)(root, (api.themed_text_new)(label.as_ptr()));

        let button_label = CString::new("Click me").unwrap();
        let button = (api.button_new)(button_label.as_ptr(), on_click, userdata);
        (api.view_add_child)(root, button);

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

fn cdylib_path() -> std::path::PathBuf {
    if let Ok(override_path) = std::env::var("CREAMUI_LIB_PATH") {
        return override_path.into();
    }
    let mut path = std::env::current_exe().expect("failed to resolve current executable path");
    path.pop(); // target/debug/
    let name = if cfg!(target_os = "macos") {
        "libcreamui.dylib"
    } else if cfg!(target_os = "windows") {
        "creamui.dll"
    } else {
        "libcreamui.so"
    };
    path.join(name)
}

fn main() {
    let lib_path = cdylib_path();
    let lib = unsafe { Library::new(&lib_path) }
        .unwrap_or_else(|e| panic!("failed to dlopen {lib_path:?}: {e} (set CREAMUI_LIB_PATH to override)"));

    unsafe {
        let view_new: Symbol<ViewNewFn> = lib.get(b"creamui_view_new").unwrap();
        let view_set_background: Symbol<ViewSetBackgroundFn> = lib.get(b"creamui_view_set_background").unwrap();
        let view_add_child: Symbol<ViewAddChildFn> = lib.get(b"creamui_view_add_child").unwrap();
        let themed_text_new: Symbol<ThemedTextNewFn> = lib.get(b"creamui_themed_text_new").unwrap();
        let button_new: Symbol<ButtonNewFn> = lib.get(b"creamui_button_new").unwrap();
        let signal_i32_new: Symbol<unsafe extern "C" fn(i32) -> *mut c_void> =
            lib.get(b"creamui_signal_i32_new").unwrap();
        let signal_get: Symbol<SignalGetFn> = lib.get(b"creamui_signal_i32_get").unwrap();
        let signal_set: Symbol<SignalSetFn> = lib.get(b"creamui_signal_i32_set").unwrap();
        let run: Symbol<RunFn> = lib.get(b"creamui_run").unwrap();

        let count_signal = signal_i32_new(0);

        // Leaked deliberately: this app runs for its whole lifetime, and
        // `build`/`on_click` (plain `extern "C" fn`s) need a `'static`
        // pointer to reach these resolved symbols.
        let api: &'static Api = Box::leak(Box::new(Api {
            view_new: *view_new,
            view_set_background: *view_set_background,
            view_add_child: *view_add_child,
            themed_text_new: *themed_text_new,
            button_new: *button_new,
            signal_get: *signal_get,
            signal_set: *signal_set,
            count_signal,
        }));

        let title = CString::new("CreamUI — Hello World (dynamic)").unwrap();
        let options = CWindowOptions {
            title: title.as_ptr(),
            width: 480,
            height: 320,
            resizable: 1,
            decorations: 1,
            transparent: 0,
        };

        run(options, DARK_SURFACE, build, api as *const Api as *mut c_void);
    }
}
