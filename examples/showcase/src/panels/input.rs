use crate::prelude::*;

/// The "Input" panel: every `TextInput`/`TextArea` variation side by side —
/// plain, with a placeholder, and a multi-line editor. Each is bound to its
/// own [`TextController`] rather than a hand-wired `value`/`on_change` (and,
/// for the `TextArea`, `cursor`/`selection`) pair — the controller owns that
/// state and the widget just reads and writes through it.
#[component]
pub fn InputPanel(
    theme: Theme,
    plain: TextController,
    with_placeholder: TextController,
    notes: TextController,
    notes_wrapped: TextController,
) -> BoxedWidget {
    fn textarea_style() -> Style {
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(280.0),
                height: Dimension::Length(180.0),
            },
            ..Default::default()
        }
    }
    let error_field: BoxedWidget = Box::new(
        RawView::new(column(theme.spacing_small))
            .child(Box::new(
                TextInput::new(&theme, "", |_| {}).border(theme.danger),
            ))
            .child(Box::new(
                Text::new(&theme, "Error: this field is required")
                    .align(TextAlign::Start)
                    .color(theme.danger)
                    .style(label_style()),
            )),
    );
    let warning_field: BoxedWidget = Box::new(
        RawView::new(column(theme.spacing_small))
            .child(Box::new(
                TextInput::new(&theme, "", |_| {}).border(theme.warning),
            ))
            .child(Box::new(
                Text::new(&theme, "Warning: verify this value")
                    .align(TextAlign::Start)
                    .color(theme.warning)
                    .style(label_style()),
            )),
    );

    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Input".to_owned()} subtitle={"Write, select, and edit. Each field keeps its own content.".to_owned()} />
            {Box::new(
                card(&theme, theme.spacing_large)
                    .child(stacked_field(&theme, "Default", Box::new(jsx!{<TextInput theme={&theme} controller={&plain} />})))
                    .child(stacked_field(&theme, "With placeholder", Box::new(jsx!{<TextInput theme={&theme} controller={&with_placeholder} placeholder={"Type something…".to_owned()} />})))
            ) as BoxedWidget}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Validation states"))
                    .child(card_row(&theme, vec![error_field, warning_field]))
            ) as BoxedWidget}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Text area"))
                    .child(card_row(&theme, vec![
                        stacked_field(&theme, "Horizontal scrolling", Box::new(jsx!{<TextArea theme={&theme} controller={&notes} style={textarea_style()} placeholder={"Notes…".to_owned()} />})),
                        stacked_field(&theme, "Wrap to fit", Box::new(jsx!{<TextArea theme={&theme} controller={&notes_wrapped} style={textarea_style()} wrap={true} placeholder={"Notes…".to_owned()} />})),
                    ]))
            ) as BoxedWidget}
        </RawView>
    })
}
