//! The same reactive counter as the basic hello-world, written with `jsx!`.

use creamui_core::layout::{AlignItems, FlexDirection, JustifyContent, Style};
use creamui_core::{BoxedWidget, Size};
use creamui_macros::jsx;
use creamui_reactive::Signal;
use creamui_render::{run, WindowOptions};
use creamui_theme::{Theme, ThemeProvider};

fn main() {
    let theme_provider = ThemeProvider::new(Theme::dark());
    let count = Signal::new(0i32);
    run(
        WindowOptions {
            title: "CreamUI — JSX Hello World".into(),
            width: 480,
            height: 320,
            ..Default::default()
        },
        Theme::dark().surface,
        |_| {},
        move |viewport: Size| -> BoxedWidget {
            let theme = theme_provider.get();
            let count_for_click = count.clone();
            let theme_for_click = theme_provider.clone();
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
            Box::new(jsx! {
                <RawView style={root_style} background={theme.surface}>
                    <Text theme={&theme} font_size={28.0}>"Hello, CreamUI!"</Text>
                    <Text theme={&theme} secondary={true}>{format!("Clicked {} times", count.get())}</Text>
                    <Button theme={&theme} on_click={move || count_for_click.update(|value| *value += 1)}>"Click me"</Button>
                    <Button theme={&theme} on_click={move || {
                        let next = if theme_for_click.get().surface == Theme::dark().surface { Theme::light() } else { Theme::dark() };
                        theme_for_click.set(next);
                    }}>"Toggle theme"</Button>
                </RawView>
            })
        },
    );
}
