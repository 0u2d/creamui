//! The same themed counter as `examples/hello_world` — pixel-for-pixel the
//! same widget tree (title, counter text, click/theme-toggle buttons, a
//! checkbox row, a text input, a volume slider, and a scrollable list) — but
//! linked dynamically: this binary depends on zero CreamUI *engine* crates.
//! It `dlopen`s the `cdylib` built from `creamui-ffi` at runtime through
//! `creamui-dynamic`, a thin safe wrapper that resolves the C ABI once and
//! exposes it with the same ergonomics as the native `creamui-widgets` API
//! — no hand-declared `#[repr(C)]` structs or `Symbol<...>` lookups here.
//!
//! Compare `target/release/hello_world` (static, links the whole engine
//! into the binary) against this binary's size — this one stays tiny
//! because the engine lives in the shared `libcreamui.so` instead.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use creamui_dynamic::{
    button, checkbox, run, scroll_view, slider, text_input, themed_text, themed_text_secondary,
    themed_text_sized, view_styled, Context, Dimension, Runtime, SignalF32, SignalI32, SignalString, Size, Style,
    Theme, WindowOptions, ALIGN_CENTER, FLEX_DIRECTION_COLUMN, JUSTIFY_CENTER,
};

/// `count`/`checked`/`name`/`volume`/`scroll_y` mirror the static example's
/// plain `Signal`s; `light_theme` reuses a bool signal rather than adding a
/// theme-signal type just for this demo's runtime toggle.
struct AppState {
    count: SignalI32,
    light_theme: SignalI32,
    checked: SignalI32,
    name: SignalString,
    volume: SignalF32,
    scroll_y: SignalF32,
}

fn current_theme(rt: &Runtime, state: &AppState) -> Theme {
    if state.light_theme.get() != 0 {
        rt.theme_light()
    } else {
        rt.theme_dark()
    }
}

fn build_ui(ctx: &Context, size: Size, state: &AppState) -> creamui_dynamic::Widget {
    let rt = ctx.runtime().clone();
    let theme = current_theme(&rt, state);

    let root_style = Style {
        flex_direction: FLEX_DIRECTION_COLUMN,
        justify_content: JUSTIFY_CENTER,
        align_items: ALIGN_CENTER,
        width: Dimension::length(size.width),
        height: Dimension::length(size.height),
        gap_row: theme.spacing_large,
        ..Style::default()
    };

    let count_for_click = state.count.clone();
    let light_theme_for_click = state.light_theme.clone();
    let checked_for_click = state.checked.clone();
    let name_for_change = state.name.clone();
    let volume_for_change = state.volume.clone();
    let scroll_y_for_scroll = state.scroll_y.clone();

    let checkbox_row = view_styled(ctx, Style::row(theme.spacing_small))
        .child(checkbox(ctx, theme, state.checked.get() != 0, move || {
            let next = checked_for_click.get() == 0;
            checked_for_click.set(next as i32);
        }))
        .child(themed_text(ctx, theme, "Enable extra sparkle"));

    let name_style = Style { width: Dimension::length(200.0), height: Dimension::length(36.0), ..Style::default() };
    let input = text_input(ctx, theme, name_style, &state.name.get(), move |next| name_for_change.set(&next))
        .placeholder(theme, "Your name");

    let volume = state.volume.get();
    let slider_style = Style { width: Dimension::length(160.0), height: Dimension::length(20.0), ..Style::default() };

    let scroll_style = Style { width: Dimension::length(300.0), height: Dimension::length(100.0), ..Style::default() };
    let mut scroll = scroll_view(ctx, theme, scroll_style, state.scroll_y.get(), move |delta| {
        let current = scroll_y_for_scroll.get();
        scroll_y_for_scroll.set((current + delta).clamp(0.0, 300.0));
    });
    for i in 0..10 {
        scroll = scroll.child(themed_text(ctx, theme, &format!("Scrollable item {i}")));
    }

    view_styled(ctx, root_style)
        .background(theme.surface)
        .child(themed_text_sized(ctx, theme, "Hello, CreamUI!", 28.0))
        .child(themed_text_secondary(ctx, theme, &format!("Clicked {} times", state.count.get())))
        .child(button(ctx, theme, "Click me", move || {
            count_for_click.set(count_for_click.get() + 1);
        }))
        .child(button(ctx, theme, "Toggle theme", move || {
            let next = light_theme_for_click.get() == 0;
            light_theme_for_click.set(next as i32);
        }))
        .child(checkbox_row)
        .child(input)
        .child(themed_text_secondary(ctx, theme, &format!("Volume: {:.0}%", volume * 100.0)))
        .child(slider(ctx, theme, slider_style, volume, move |next| {
            volume_for_change.set(next);
        }))
        .child(scroll)
}

fn main() {
    let rt = Runtime::load_default();

    let state = AppState {
        count: SignalI32::new(&rt, 0),
        light_theme: SignalI32::new(&rt, 0),
        checked: SignalI32::new(&rt, 0),
        name: SignalString::new(&rt, ""),
        volume: SignalF32::new(&rt, 0.5),
        scroll_y: SignalF32::new(&rt, 0.0),
    };

    let options = WindowOptions {
        title: "CreamUI — Hello World (dynamic)".to_string(),
        width: 480,
        height: 600,
        ..Default::default()
    };
    let initial_clear_color = rt.theme_dark().surface;

    run(&rt, options, initial_clear_color, |_handle| {}, move |ctx, size| build_ui(ctx, size, &state));
}
