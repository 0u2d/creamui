//! CreamUI hello-world example: a themed counter with a runtime theme
//! toggle, a checkbox, a keyboard-driven text input, a draggable slider,
//! and a scrollable list.
//!
//! Demonstrates the MVP pillars together: `Signal`s driving reactive
//! re-renders (including a runtime `ThemeProvider` swap), themed widgets
//! built on the headless ones, HTML/CSS-like flex layout (via `taffy`),
//! keyboard focus, pointer-drag, and scroll-wheel input, and a
//! GPU-presented window.

use creamui_core::layout::{AlignItems, FlexDirection, JustifyContent, Style};
use creamui_core::{BoxedWidget, Size};
use creamui_reactive::Signal;
use creamui_render::{run, WindowOptions};
use creamui_theme::{Theme, ThemeProvider};
use creamui_widgets::raw::RawView;
use creamui_widgets::themed::{Button, Checkbox, ScrollView, Slider, Text, TextInput};

fn main() {
    let theme_provider = ThemeProvider::new(Theme::dark());
    let count = Signal::new(0i32);
    let checked = Signal::new(false);
    let name = Signal::new(String::new());
    let volume = Signal::new(0.5f32);
    let scroll_y = Signal::new(0.0f32);

    let options = WindowOptions {
        title: "CreamUI — Hello World".to_string(),
        width: 480,
        height: 600,
        ..Default::default()
    };

    let initial_clear_color = Theme::dark().surface;

    run(options, initial_clear_color, move |viewport: Size| -> BoxedWidget {
        let theme = theme_provider.get();
        let count_for_click = count.clone();
        let theme_provider_for_click = theme_provider.clone();
        let checked_for_click = checked.clone();
        let name_for_change = name.clone();
        let volume_for_change = volume.clone();
        let scroll_y_for_scroll = scroll_y.clone();

        let root_style = Style {
            size: creamui_core::layout::Size {
                width: creamui_core::layout::Dimension::Length(viewport.width),
                height: creamui_core::layout::Dimension::Length(viewport.height),
            },
            flex_direction: FlexDirection::Column,
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            gap: creamui_core::layout::Size {
                width: creamui_core::layout::LengthPercentage::Length(0.0),
                height: creamui_core::layout::LengthPercentage::Length(theme.spacing_large),
            },
            ..Default::default()
        };

        Box::new(
            RawView::new(root_style)
                .background(theme.surface)
                .child(Box::new(Text::new(&theme, "Hello, CreamUI!").font_size(28.0)))
                .child(Box::new(Text::secondary(
                    &theme,
                    format!("Clicked {} times", count.get()),
                )))
                .child(Box::new(Button::new(&theme, "Click me", move || {
                    count_for_click.update(|c| *c += 1);
                })))
                .child(Box::new(Button::new(&theme, "Toggle theme", move || {
                    let next = if theme_provider_for_click.get().surface == Theme::dark().surface {
                        Theme::light()
                    } else {
                        Theme::dark()
                    };
                    theme_provider_for_click.set(next);
                })))
                .child(Box::new(
                    RawView::new(creamui_widgets::layout::row(theme.spacing_small))
                        .child(Box::new(Checkbox::new(&theme, checked.get(), move || {
                            checked_for_click.update(|c| *c = !*c);
                        })))
                        .child(Box::new(Text::new(&theme, "Enable extra sparkle"))),
                ))
                .child(Box::new(
                    TextInput::new(&theme, name.get(), move |next| name_for_change.set(next))
                        .placeholder(&theme, "Your name"),
                ))
                .child(Box::new(Text::secondary(&theme, format!("Volume: {:.0}%", volume.get() * 100.0))))
                .child(Box::new(Slider::new(&theme, volume.get(), move |next| {
                    volume_for_change.set(next);
                })))
                .child(Box::new(
                    ScrollView::new(
                        &theme,
                        Style {
                            size: creamui_core::layout::Size {
                                width: creamui_core::layout::Dimension::Length(300.0),
                                height: creamui_core::layout::Dimension::Length(100.0),
                            },
                            ..Default::default()
                        },
                        scroll_y.get(),
                        move |delta| scroll_y_for_scroll.update(|y| *y = (*y + delta).clamp(0.0, 300.0)),
                    )
                    .with_children(
                        (0..10)
                            .map(|i| Box::new(Text::new(&theme, format!("Scrollable item {i}"))) as BoxedWidget)
                            .collect(),
                    ),
                )),
        )
    });
}
