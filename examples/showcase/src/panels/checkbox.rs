use crate::prelude::*;

fn checkbox_row(label: &str, checked: Signal<bool>) -> BoxedWidget {
    let theme = use_theme();
    let is_checked = checked.get();
    let toggle = checked.clone();
    let state_text = if is_checked { "On" } else { "Off" };
    let row_style = Style {
        align_items: Some(AlignItems::Center),
        ..row(theme.spacing_medium)
    };
    let text_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Auto,
            height: Dimension::Length(18.0),
        },
        ..Default::default()
    };
    Box::new(jsx! {
        <RawView style={row_style}>
            <Checkbox checked={is_checked} on_click={move || toggle.update(|c| *c = !*c)} />
            <Text align={TextAlign::Start} style={text_style}>{format!("{} — {}", label, state_text)}</Text>
        </RawView>
    })
}

/// The "Checkbox" panel: three checkboxes, each toggling its own boolean
/// `Signal` and reflecting the current state in its label.
#[component]
pub fn CheckboxPanel(
    notifications: Signal<bool>,
    auto_save: Signal<bool>,
    beta_features: Signal<bool>,
) -> BoxedWidget {
    let theme = use_theme();
    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader title={"Checkbox".to_owned()} subtitle={"Small preferences, clearly expressed.".to_owned()} />
            {Box::new(
                card(theme.spacing_medium)
                    .child(field_label("Preferences"))
                    .child(checkbox_row("Notifications", notifications))
                    .child(checkbox_row("Auto-save", auto_save.clone()))
                    .child(checkbox_row("Beta features", beta_features))
            ) as BoxedWidget}
            {Box::new(
                card(theme.spacing_medium)
                    .child(field_label("Switch presentation"))
                    .child(Box::new(
                        RawView::new(Style { align_items: Some(AlignItems::Center), ..row(theme.spacing_medium) })
                            .child(Box::new(Switch::new(auto_save.get(), { let set = auto_save.clone(); move || set.update(|value| *value = !*value) })))
                            .child(Box::new(Text::secondary("The same boolean, shown as a switch instead of a checkbox.").align(TextAlign::Start))),
                    ))
            ) as BoxedWidget}
        </RawView>
    })
}
