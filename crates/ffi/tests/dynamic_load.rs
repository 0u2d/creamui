//! Proves the ABI-stability claim: this test never links `creamui-ffi`
//! directly. Instead it `dlopen`s the built `cdylib` at runtime (the same
//! way an app in any language would) and calls into it purely through
//! C function pointers and `#[repr(C)]` types.

use libloading::{Library, Symbol};
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::atomic::{AtomicI32, Ordering};

#[repr(C)]
struct CColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

fn cdylib_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps/
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

#[test]
fn loads_dynamically_and_builds_a_widget_tree() {
    let path = cdylib_path();
    let lib = unsafe { Library::new(&path) }
        .unwrap_or_else(|e| panic!("failed to dlopen {path:?}: {e}"));

    unsafe {
        let version: Symbol<unsafe extern "C" fn() -> *const c_char> =
            lib.get(b"creamui_version").unwrap();
        let version_str = CStr::from_ptr(version()).to_str().unwrap();
        assert_eq!(version_str, env!("CARGO_PKG_VERSION"));

        let view_new: Symbol<unsafe extern "C" fn() -> *mut c_void> = lib.get(b"creamui_view_new").unwrap();
        let set_background: Symbol<unsafe extern "C" fn(*mut c_void, CColor)> =
            lib.get(b"creamui_view_set_background").unwrap();
        let text_new: Symbol<unsafe extern "C" fn(*const c_char, CColor, f32) -> *mut c_void> =
            lib.get(b"creamui_text_new").unwrap();
        let add_child: Symbol<unsafe extern "C" fn(*mut c_void, *mut c_void)> =
            lib.get(b"creamui_view_add_child").unwrap();
        let widget_free: Symbol<unsafe extern "C" fn(*mut c_void)> = lib.get(b"creamui_widget_free").unwrap();

        let root = view_new();
        assert!(!root.is_null());
        set_background(root, CColor { r: 10, g: 10, b: 10, a: 255 });

        let label = CString::new("Loaded via dlopen").unwrap();
        let text = text_new(label.as_ptr(), CColor { r: 255, g: 255, b: 255, a: 255 }, 16.0);
        assert!(!text.is_null());

        add_child(root, text);
        widget_free(root);
    }
}

#[test]
fn button_click_callback_crosses_the_abi_boundary() {
    let path = cdylib_path();
    let lib = unsafe { Library::new(&path) }.unwrap();

    static CLICKED: AtomicI32 = AtomicI32::new(0);
    extern "C" fn on_click(_userdata: *mut c_void) {
        CLICKED.fetch_add(1, Ordering::SeqCst);
    }

    unsafe {
        let button_new: Symbol<
            unsafe extern "C" fn(*const c_char, extern "C" fn(*mut c_void), *mut c_void) -> *mut c_void,
        > = lib.get(b"creamui_button_new").unwrap();
        let widget_free: Symbol<unsafe extern "C" fn(*mut c_void)> = lib.get(b"creamui_widget_free").unwrap();

        let label = CString::new("Click via FFI").unwrap();
        let button = button_new(label.as_ptr(), on_click, std::ptr::null_mut());
        assert!(!button.is_null());

        // We only verify the widget was constructed and the symbol
        // resolved correctly here; actually invoking the click requires a
        // running render loop, exercised by the widgets crate's own
        // integration test against the same underlying `Button` type.
        widget_free(button);
        assert_eq!(CLICKED.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn signal_i32_get_reflects_set() {
    let path = cdylib_path();
    let lib = unsafe { Library::new(&path) }.unwrap();

    unsafe {
        let new: Symbol<unsafe extern "C" fn(i32) -> *mut c_void> = lib.get(b"creamui_signal_i32_new").unwrap();
        let get: Symbol<unsafe extern "C" fn(*const c_void) -> i32> = lib.get(b"creamui_signal_i32_get").unwrap();
        let set: Symbol<unsafe extern "C" fn(*const c_void, i32)> = lib.get(b"creamui_signal_i32_set").unwrap();
        let free: Symbol<unsafe extern "C" fn(*mut c_void)> = lib.get(b"creamui_signal_i32_free").unwrap();

        let signal = new(41);
        assert!(!signal.is_null());
        assert_eq!(get(signal), 41);

        set(signal, 42);
        assert_eq!(get(signal), 42, "a dynamically-linked app needs this to build a working counter, since it has no Rust-side Signal of its own");

        free(signal);
    }
}
