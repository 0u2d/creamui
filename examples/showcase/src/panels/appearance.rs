use crate::prelude::*;

/// A pill-shaped selectable button, used for the dark/light and accent
/// pickers on the Appearance page. Selection is fully controlled: it carries
/// no state of its own.
fn pill(theme: &Theme, label: &str, active: bool, on_click: impl Fn() + 'static) -> BoxedWidget {
    Box::new(Choice::new(theme, label, active, on_click))
}

/// A single accent color swatch: a rounded square filled with the color
/// itself, with a ring drawn around whichever one is active.
fn swatch(theme: &Theme, color: Color, active: bool, on_click: impl Fn() + 'static) -> BoxedWidget {
    let outer = Style {
        size: fixed(40.0, 40.0),
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
        flex_shrink: 0.0,
        ..Default::default()
    };
    let inner_size = if active { 28.0 } else { 32.0 };
    let inner = Style {
        size: fixed(inner_size, inner_size),
        ..Default::default()
    };
    let ring = if active {
        theme.text_primary
    } else {
        theme.surface
    };
    Box::new(jsx! {
        <RawButton style={outer} background={ring} corner_radius={20.0} on_click={on_click}>
            <RawView style={inner} background={color} corner_radius={16.0} />
        </RawButton>
    })
}

/// The "Appearance" panel: toggles dark/light mode and picks an accent
/// color, both stored in `Signal`s owned by `main`, so changes are visible
/// immediately across every other section.
#[component]
pub fn AppearancePanel(
    theme: Theme,
    dark_mode: Signal<bool>,
    accent_index: Signal<usize>,
) -> BoxedWidget {
    let is_dark = dark_mode.get();
    let selected_accent = accent_index.get();

    let dark_flag = dark_mode.clone();
    let light_flag = dark_mode.clone();
    let mode_pills: Vec<BoxedWidget> = vec![
        pill(&theme, "Dark", is_dark, move || dark_flag.set(true)),
        pill(&theme, "Light", !is_dark, move || light_flag.set(false)),
    ];

    let mut swatches: Vec<BoxedWidget> = Vec::new();
    for (index, (_, color)) in ACCENTS.iter().enumerate() {
        let set_accent = accent_index.clone();
        swatches.push(swatch(
            &theme,
            *color,
            index == selected_accent,
            move || set_accent.set(index),
        ));
    }

    let accent_name = ACCENTS[selected_accent].0;
    let summary = format!(
        "{} mode · {} accent",
        if is_dark { "Dark" } else { "Light" },
        accent_name
    );

    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Appearance".to_owned()} subtitle={"Explore the same components in a different light.".to_owned()} />
            {Box::new(
                card(&theme, theme.spacing_large)
                    .child(Box::new(
                        card(&theme, theme.spacing_small)
                            .child(field_label(&theme, "Mode"))
                            .child(Box::new(RawView::new(row(theme.spacing_medium)).with_children(mode_pills))),
                    ))
                    .child(Box::new(
                        card(&theme, theme.spacing_small)
                            .child(field_label(&theme, "Accent color"))
                            .child(Box::new(RawView::new(row(theme.spacing_medium)).with_children(swatches))),
                    ))
                    .child(Box::new(Text::new(&theme, summary).align(TextAlign::Start).color(theme.text_disabled).style(label_style())))
            ) as BoxedWidget}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(Box::new(Heading::new(&theme, "Component preview")))
                    .child(Box::new(Text::secondary(&theme, "Open a category to explore sizes, states, and interactions.").align(TextAlign::Start)))
                    .child(card_row(&theme, vec![
                        Box::new(Button::new(&theme, "Primary", { let mode = dark_mode.clone(); move || mode.update(|v| *v = !*v) })),
                        Box::new(Button::secondary(&theme, ButtonSize::Md, "Secondary", { let mode = dark_mode.clone(); move || mode.update(|v| *v = !*v) })),
                        Box::new(Button::new(&theme, "Disabled", || {}).disabled(true)),
                    ]))
                    .child(Box::new(Text::secondary(&theme, "These preview buttons switch the color scheme.").align(TextAlign::Start)))
            ) as BoxedWidget}
        </RawView>
    })
}
