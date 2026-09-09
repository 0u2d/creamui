use crate::prelude::*;

/// Determinate/indeterminate feedback plus a regular floating popover and a
/// modal alert. The alert itself is attached at the root below.
#[component]
pub fn FeedbackPanel(
    theme: Theme,
    progress: Signal<f32>,
    show_popover: Signal<bool>,
    show_alert: Signal<bool>,
) -> BoxedWidget {
    let value = progress.get();
    let set_progress = progress.clone();
    let open_popover = show_popover.clone();
    let open_alert = show_alert.clone();
    let popover = if show_popover.get() {
        Box::new(
            Popover::new(
                &theme,
                padding(column(theme.spacing_small), theme.spacing_medium),
            )
            .child(Box::new(
                RawText::new("Popover", theme.text_primary, 13.).bold(true),
            ))
            .child(Box::new(RawText::new(
                "A floating surface can contain any widget tree.",
                theme.text_secondary,
                12.,
            ))),
        ) as BoxedWidget
    } else {
        Box::new(RawView::new(Style::default())) as BoxedWidget
    };
    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Feedback & overlays".to_owned()} subtitle={"Show work in progress, surface contextual detail, and ask for confirmation without losing context.".to_owned()} />
            {field_card(&theme, &format!("Determinate progress · {:.0}%", value * 100.0), Box::new(
                RawView::new(column(theme.spacing_small))
                    .child(Box::new(ProgressBar::new(&theme, value)))
                    .child(Box::new(jsx!{<Slider theme={&theme} value={value} on_change={move |next| set_progress.set(next)} />})),
            ))}
            {card_row(&theme, vec![
                field_card(&theme, "Progress ring", Box::new(
                    RawView::new(row(theme.spacing_medium))
                        .child(Box::new(ProgressRing::new(&theme, value).size(32.0)))
                        .child(Box::new(ProgressRing::indeterminate(&theme).size(32.0))),
                )),
                field_card(&theme, "Indeterminate bar", Box::new(ProgressBar::indeterminate(&theme))),
            ])}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Overlays"))
                    .child(Box::new(
                        RawView::new(row(theme.spacing_medium))
                            .child(Box::new(Button::secondary(&theme, ButtonSize::Md, "Toggle popover", move || open_popover.update(|open| *open = !*open))))
                            .child(Box::new(Button::new(&theme, "Open alert dialog", move || open_alert.set(true)))),
                    ))
                    .child(popover)
            ) as BoxedWidget}
        </RawView>
    })
}
