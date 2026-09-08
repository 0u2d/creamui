//! Headless widgets: fully unstyled building blocks with no opinion on
//! color, radius, or spacing. Themed widgets (see [`crate::themed`]) wrap
//! these and fill in appearance from a [`creamui_theme::Theme`]; apps that
//! want a completely custom look can use these directly instead.

use creamui_core::layout::Style;
use creamui_core::{
    BoxedWidget, CursorIcon, Key, KeyInput, Painter, Point, Rect, TextAlign, Widget,
};
use creamui_theme::Color;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    // On X11/Wayland the clipboard owner must remain alive after the write;
    // creating and dropping `arboard::Clipboard` inside a key callback makes
    // clipboard managers lose the contents immediately.
    static SYSTEM_CLIPBOARD: RefCell<Option<arboard::Clipboard>> = const { RefCell::new(None) };
}

fn activate_on_key(click: Rc<dyn Fn()>) -> Rc<dyn Fn(KeyInput)> {
    Rc::new(move |input| {
        if !input.modifiers.ctrl && matches!(input.key, Key::Enter | Key::Char(' ')) {
            click();
        }
    })
}

fn clipboard_write(text: String) {
    SYSTEM_CLIPBOARD.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = arboard::Clipboard::new().ok();
        }
        if let Some(clipboard) = slot.as_mut() {
            let _ = clipboard.set_text(text);
        }
    });
}

fn clipboard_read() -> Option<String> {
    SYSTEM_CLIPBOARD.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = arboard::Clipboard::new().ok();
        }
        slot.as_mut()
            .and_then(|clipboard| clipboard.get_text().ok())
    })
}

mod button;
mod controls;
mod foundation;
mod navigation;
mod scroll;
mod spinner;
mod text_input;

pub use button::*;
pub use controls::*;
pub use foundation::*;
pub use navigation::*;
pub use scroll::*;
pub use spinner::*;
pub use text_input::*;
