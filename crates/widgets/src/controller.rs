//! [`TextController`]: a shareable, persistent bundle of a text editing
//! widget's value, cursor, and selection — the one object an app creates
//! once (the same discipline a `Signal` already requires: create it in
//! `main`, then clone the handle into whatever needs it) and hands to
//! [`crate::themed::TextInput`]/[`crate::themed::TextArea`], instead of
//! wiring up `value`/`on_change`/`cursor`/`on_cursor_change`/`selection`/
//! `on_selection_change` by hand every time.
//!
//! This can't be a zero-setup "hook" the way React's `useState` is: nothing
//! in this crate rebuilds widgets in place across renders (see
//! `creamui_render`'s doc comment on `build_ui`), so there's no per-call-site
//! slot to remember state in without the app holding a handle itself. A
//! `TextController` is that handle — cheap to create, `Clone` (an `Rc`
//! underneath, exactly like `Signal`), and if a widget isn't given one it
//! creates its own default, unrestricted one internally, which is as close
//! to "free" as an immediate-mode rebuild model can get.

use crate::raw::TextSelection;
use creamui_reactive::Signal;
use std::rc::Rc;

type ChangeGuard = dyn Fn(&str, &str) -> Option<String>;

/// A controller for a text editing widget's value, cursor, and selection.
///
/// Reading [`TextController::value`] (or `.cursor()`/`.selection()`) inside
/// a reactive effect — which is what [`crate::themed::TextInput`] and
/// [`crate::themed::TextArea`] do internally when bound to one — is how you
/// "hook into changes": the same subscribe-on-read mechanism as
/// [`creamui_reactive::Signal`], since a controller is just three `Signal`s
/// under one shared handle. To additionally *veto* or rewrite a proposed
/// change (e.g. enforce a max length, strip disallowed characters), install
/// a guard with [`TextController::on_change`].
#[derive(Clone)]
pub struct TextController {
    value: Signal<String>,
    cursor: Signal<usize>,
    selection: Signal<TextSelection>,
    guard: Rc<std::cell::RefCell<Option<Box<ChangeGuard>>>>,
}

impl TextController {
    /// A new, unrestricted controller seeded with `initial`, cursor placed
    /// at its end.
    pub fn new(initial: impl Into<String>) -> Self {
        let value = initial.into();
        let end = value.len();
        TextController {
            value: Signal::new(value),
            cursor: Signal::new(end),
            selection: Signal::new(TextSelection {
                anchor: end,
                focus: end,
            }),
            guard: Rc::new(std::cell::RefCell::new(None)),
        }
    }

    /// The current value, subscribing the running reactive effect (if any)
    /// to future changes — same semantics as [`Signal::get`].
    pub fn value(&self) -> String {
        self.value.get()
    }

    /// Reads the current value without subscribing.
    pub fn peek(&self) -> String {
        self.value.peek()
    }

    pub fn cursor(&self) -> usize {
        self.cursor.get()
    }

    pub fn selection(&self) -> TextSelection {
        self.selection.get()
    }

    pub fn set_cursor(&self, cursor: usize) {
        self.cursor.set(cursor.min(self.value.peek().len()));
    }

    pub fn set_selection(&self, selection: TextSelection) {
        let len = self.value.peek().len();
        self.selection.set(TextSelection {
            anchor: selection.anchor.min(len),
            focus: selection.focus.min(len),
        });
    }

    /// Proposes `next` as the new value. If a guard is installed (see
    /// [`TextController::on_change`]), it decides what actually happens:
    /// returning `None` rejects the change outright (`value()` keeps its
    /// current contents, as if the keystroke never happened); returning
    /// `Some(text)` accepts `text` — usually `next` unchanged, but the guard
    /// may rewrite it (e.g. truncate, strip characters). With no guard
    /// installed, `next` is always accepted as-is.
    pub fn set_value(&self, next: impl Into<String>) {
        let next = next.into();
        let current = self.value.peek();
        let accepted = match self.guard.borrow().as_ref() {
            Some(guard) => guard(&current, &next),
            None => Some(next),
        };
        let Some(text) = accepted else { return };
        let len = text.len();
        self.value.set(text);
        if self.cursor.peek() > len {
            self.cursor.set(len);
        }
        let selection = self.selection.peek();
        if selection.anchor > len || selection.focus > len {
            self.selection.set(TextSelection {
                anchor: selection.anchor.min(len),
                focus: selection.focus.min(len),
            });
        }
    }

    /// Installs a hook run before every [`TextController::set_value`] call
    /// (including edits typed into a bound `TextInput`/`TextArea`): given
    /// `(current, proposed)`, return `Some(text)` to accept the change
    /// (optionally rewriting it), or `None` to reject it and keep `current`.
    /// Replaces any previously installed guard.
    pub fn on_change(&self, guard: impl Fn(&str, &str) -> Option<String> + 'static) {
        *self.guard.borrow_mut() = Some(Box::new(guard));
    }

    /// Removes any guard installed via [`TextController::on_change`],
    /// returning to unrestricted edits.
    pub fn clear_guard(&self) {
        *self.guard.borrow_mut() = None;
    }
}

impl Default for TextController {
    /// An empty, unrestricted controller — what a `TextInput`/`TextArea`
    /// creates internally when constructed without one.
    fn default() -> Self {
        TextController::new(String::new())
    }
}

/// Shared selected-index state for one tab bar and the content it controls.
#[derive(Clone)]
pub struct TabController {
    selected: Signal<usize>,
}

impl TabController {
    pub fn new(selected: usize) -> Self {
        Self {
            selected: Signal::new(selected),
        }
    }

    /// Reads the selected index and subscribes the current reactive render.
    pub fn selected(&self) -> usize {
        self.selected.get()
    }

    /// Reads the selected index without subscribing.
    pub fn peek(&self) -> usize {
        self.selected.peek()
    }

    pub fn select(&self, index: usize) {
        self.selected.set(index);
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected() == index
    }
}

impl Default for TabController {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_value_updates_cursor_and_selection_when_they_fall_out_of_range() {
        let controller = TextController::new("hello");
        controller.set_cursor(5);
        controller.set_selection(TextSelection {
            anchor: 2,
            focus: 5,
        });
        controller.set_value("hi");
        assert_eq!(controller.value(), "hi");
        assert_eq!(controller.cursor(), 2);
        assert_eq!(
            controller.selection(),
            TextSelection {
                anchor: 2,
                focus: 2
            }
        );
    }

    #[test]
    fn on_change_guard_can_reject_a_change() {
        let controller = TextController::new("ok");
        controller.on_change(|_current, proposed| {
            if proposed.len() > 5 {
                None
            } else {
                Some(proposed.to_owned())
            }
        });
        controller.set_value("still ok");
        assert_eq!(
            controller.value(),
            "ok",
            "change longer than 5 chars should be rejected"
        );
        controller.set_value("short");
        assert_eq!(controller.value(), "short");
    }

    #[test]
    fn on_change_guard_can_rewrite_a_change() {
        let controller = TextController::new("");
        controller.on_change(|_current, proposed| Some(proposed.to_uppercase()));
        controller.set_value("shout");
        assert_eq!(controller.value(), "SHOUT");
    }

    #[test]
    fn default_controller_is_empty_and_unrestricted() {
        let controller = TextController::default();
        assert_eq!(controller.value(), "");
        controller.set_value("anything");
        assert_eq!(controller.value(), "anything");
    }

    #[test]
    fn tab_controller_tracks_one_shared_selection() {
        let tabs = TabController::new(1);
        assert!(tabs.is_selected(1));
        tabs.select(2);
        assert_eq!(tabs.selected(), 2);
    }
}
