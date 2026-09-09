use crate::prelude::*;

/// The "Button" panel: the default themed `Button`, a couple of
/// semantically-colored variants built straight from `RawButton`, and a
/// disabled-looking one, plus a click counter to prove the handlers fire.
#[component]
pub fn ButtonPanel(theme: Theme, clicks: Signal<i32>) -> BoxedWidget {
    let mut actions = RawView::new(row(10.));
    for (label, variant) in [
        ("Continue", creamui_widgets::ButtonVariant::Primary),
        ("Cancel", creamui_widgets::ButtonVariant::Secondary),
        ("Learn more", creamui_widgets::ButtonVariant::Tertiary),
        ("Delete", creamui_widgets::ButtonVariant::Destructive),
    ] {
        let clicks = clicks.clone();
        actions = actions.child(Box::new(Button::styled(
            &theme,
            variant,
            ButtonSize::Md,
            label,
            ButtonState::Normal,
            move || clicks.update(|c| *c += 1),
        )));
    }
    let mut sizes = RawView::new(row(10.));
    for (label, size) in [
        ("Extra small", ButtonSize::Xs),
        ("Small", ButtonSize::Sm),
        ("Medium", ButtonSize::Md),
        ("Large", ButtonSize::Lg),
    ] {
        let clicks = clicks.clone();
        sizes = sizes.child(Box::new(Button::secondary(
            &theme,
            size,
            label,
            move || clicks.update(|c| *c += 1),
        )));
    }
    Box::new(
        RawView::new(column(section_gap(&theme)))
            .child(SectionHeader(SectionHeaderProps {
                theme,
                title: "Buttons".into(),
                subtitle: "A clear hierarchy, from everyday actions to important decisions.".into(),
            }))
            .child(Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Variants"))
                    .child(Box::new(actions)),
            ))
            .child(Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "One family, four sizes"))
                    .child(Box::new(sizes)),
            ))
            .child(Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "States"))
                    .child(Box::new(
                        RawView::new(row(10.))
                            .child(Box::new(
                                Button::new(&theme, "Unavailable", || {}).disabled(true),
                            ))
                            .child(Box::new(Button::state(
                                &theme,
                                ButtonSize::Md,
                                "Working",
                                ButtonState::Loading,
                                || {},
                            )))
                            .child(Box::new(Button::state(
                                &theme,
                                ButtonSize::Md,
                                "Saved",
                                ButtonState::Success,
                                || {},
                            ))),
                    ))
                    .child(Box::new(
                        RawText::new(
                            format!("{} actions · Try Tab, then Enter or Space", clicks.get()),
                            theme.text_secondary,
                            12.,
                        )
                        .align(TextAlign::Start),
                    )),
            )),
    )
}

