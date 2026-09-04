//! CreamUI hello-world example: a themed counter.
//!
//! Demonstrates the four MVP pillars together: a [`Signal`] driving
//! reactive re-renders, themed widgets built on the headless ones, HTML/CSS-like
//! flex layout (via `taffy`), and a GPU-presented window.

use creamui_core::layout::{AlignItems, FlexDirection, JustifyContent, Style};
use creamui_core::{BoxedWidget, Size};
use creamui_reactive::Signal;
use creamui_render::{run, WindowOptions};
use creamui_theme::Theme;
use creamui_widgets::raw::RawView;
use creamui_widgets::themed::{Button, Text};

fn main() {
    let theme = Theme::dark();
    let count = Signal::new(0i32);

    let options = WindowOptions {
        title: "CreamUI — Hello World".to_string(),
        width: 480,
        height: 320,
        ..Default::default()
    };

    run(options, theme.surface, move |viewport: Size| -> BoxedWidget {
        let count_for_click = count.clone();

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
                }))),
        )
    });
}
