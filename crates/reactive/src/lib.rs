//! Minimal single-threaded reactive primitives: [`Signal`] and [`create_effect`].
//!
//! This is the reactivity layer CreamUI components are built on. It is
//! intentionally small (no schedulers, no async) since it only needs to
//! drive synchronous UI re-renders on the main thread.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

thread_local! {
    static EFFECT_STACK: RefCell<Vec<Rc<EffectState>>> = const { RefCell::new(Vec::new()) };
}

struct EffectState {
    run: RefCell<Box<dyn FnMut()>>,
}

/// Handle to a running [`create_effect`] closure. Drop it to stop the effect
/// from reacting to further signal changes.
pub struct Effect {
    _state: Rc<EffectState>,
}

/// Runs `f` immediately, then re-runs it whenever any [`Signal`] read during
/// its execution is later changed via [`Signal::set`] or [`Signal::update`].
///
/// The returned [`Effect`] must be kept alive for as long as the effect
/// should keep reacting; dropping it unsubscribes from all signals it read.
pub fn create_effect(f: impl FnMut() + 'static) -> Effect {
    let state = Rc::new(EffectState {
        run: RefCell::new(Box::new(f)),
    });
    run_effect(&state);
    Effect { _state: state }
}

fn run_effect(state: &Rc<EffectState>) {
    EFFECT_STACK.with(|stack| stack.borrow_mut().push(state.clone()));
    (state.run.borrow_mut())();
    EFFECT_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
}

struct SignalInner<T> {
    value: RefCell<T>,
    subscribers: RefCell<Vec<Weak<EffectState>>>,
}

/// A reactive value cell.
///
/// Reading [`Signal::get`] inside a [`create_effect`] body subscribes that
/// effect to future writes; [`Signal::peek`] reads without subscribing.
pub struct Signal<T> {
    inner: Rc<SignalInner<T>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal {
            inner: Rc::new(SignalInner {
                value: RefCell::new(value),
                subscribers: RefCell::new(Vec::new()),
            }),
        }
    }

    /// Reads the current value, subscribing the currently running effect (if any).
    pub fn get(&self) -> T {
        self.track();
        self.inner.value.borrow().clone()
    }

    /// Reads the current value without subscribing the running effect.
    pub fn peek(&self) -> T {
        self.inner.value.borrow().clone()
    }

    /// Replaces the value and notifies subscribers.
    pub fn set(&self, value: T) {
        *self.inner.value.borrow_mut() = value;
        self.notify();
    }

    /// Mutates the value in place and notifies subscribers.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.value.borrow_mut());
        self.notify();
    }

    fn track(&self) {
        EFFECT_STACK.with(|stack| {
            if let Some(current) = stack.borrow().last() {
                let mut subs = self.inner.subscribers.borrow_mut();
                let already = subs
                    .iter()
                    .any(|w| w.upgrade().is_some_and(|s| Rc::ptr_eq(&s, current)));
                if !already {
                    subs.push(Rc::downgrade(current));
                }
            }
        });
    }

    fn notify(&self) {
        let subs: Vec<Rc<EffectState>> = {
            let mut subs = self.inner.subscribers.borrow_mut();
            subs.retain(|w| w.strong_count() > 0);
            subs.iter().filter_map(|w| w.upgrade()).collect()
        };
        for state in subs {
            run_effect(&state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn get_set_roundtrip() {
        let s = Signal::new(1);
        assert_eq!(s.get(), 1);
        s.set(42);
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn effect_runs_immediately_and_on_change() {
        let s = Signal::new(0);
        let runs = Rc::new(Cell::new(0));
        let runs_clone = runs.clone();
        let s_clone = s.clone();
        let _effect = create_effect(move || {
            let _ = s_clone.get();
            runs_clone.set(runs_clone.get() + 1);
        });
        assert_eq!(runs.get(), 1);
        s.set(1);
        assert_eq!(runs.get(), 2);
        s.set(2);
        assert_eq!(runs.get(), 3);
    }

    #[test]
    fn effect_does_not_run_for_unrelated_signal() {
        let a = Signal::new(0);
        let b = Signal::new(0);
        let runs = Rc::new(Cell::new(0));
        let runs_clone = runs.clone();
        let a_clone = a.clone();
        let _effect = create_effect(move || {
            let _ = a_clone.get();
            runs_clone.set(runs_clone.get() + 1);
        });
        assert_eq!(runs.get(), 1);
        b.set(99);
        assert_eq!(runs.get(), 1, "unrelated signal must not trigger a re-run");
    }

    #[test]
    fn peek_does_not_subscribe() {
        let s = Signal::new(0);
        let runs = Rc::new(Cell::new(0));
        let runs_clone = runs.clone();
        let s_clone = s.clone();
        let _effect = create_effect(move || {
            let _ = s_clone.peek();
            runs_clone.set(runs_clone.get() + 1);
        });
        assert_eq!(runs.get(), 1);
        s.set(1);
        assert_eq!(runs.get(), 1, "peek must not subscribe the effect");
    }

    #[test]
    fn dropping_effect_stops_updates() {
        let s = Signal::new(0);
        let runs = Rc::new(Cell::new(0));
        let runs_clone = runs.clone();
        let s_clone = s.clone();
        let effect = create_effect(move || {
            let _ = s_clone.get();
            runs_clone.set(runs_clone.get() + 1);
        });
        drop(effect);
        s.set(1);
        assert_eq!(runs.get(), 1, "dropped effect must not react anymore");
    }
}
