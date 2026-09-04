//! CreamUI hello-world example: a themed counter with a runtime theme toggle.
//!
//! Demonstrates the MVP pillars together: `Signal`s driving reactive
//! re-renders (including a runtime `ThemeProvider` swap), themed widgets
//! built on the headless ones, HTML/CSS-like flex layout (via `taffy`), and
//! a GPU-presented window.

use creamui_core::layout::{AlignItems, FlexDirection, JustifyContent, Style};
use creamui_core::{BoxedWidget, Size};
use creamui_reactive::Signal;
use creamui_render::{run, WindowOptions};
use creamui_theme::{Theme, ThemeProvider};
use creamui_widgets::raw::RawView;
use creamui_widgets::themed::{Button, Text};

fn main() {
    let theme_provider = ThemeProvider::new(Theme::dark());
    let count = Signal::new(0i32);

    let options = WindowOptions {
        title: "CreamUI — Hello World".to_string(),
        width: 480,
        height: 360,
        ..Default::default()
    };

    let initial_clear_color = Theme::dark().surface;

    run(options, initial_clear_color, move |viewport: Size| -> BoxedWidget {
        let theme = theme_provider.get();
        let count_for_click = count.clone();
        let theme_provider_for_click = theme_provider.clone();

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
                }))),
        )
    });
}
