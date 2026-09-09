use crate::prelude::*;

/// The "Input" panel: every `TextInput`/`TextArea` variation side by side —
/// plain, with a placeholder, and a multi-line editor. Each is bound to its
/// own [`TextController`] rather than a hand-wired `value`/`on_change` (and,
/// for the `TextArea`, `cursor`/`selection`) pair — the controller owns that
/// state and the widget just reads and writes through it.
#[component]
pub fn InputPanel(
    plain: TextController,
    with_placeholder: TextController,
    notes: TextController,
    notes_wrapped: TextController,
) -> BoxedWidget {
    let theme = use_theme();
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
            .child(Box::new(TextInput::new("", |_| {}).border(theme.danger)))
            .child(Box::new(
                Text::new("Error: this field is required")
                    .align(TextAlign::Start)
                    .color(theme.danger)
                    .style(label_style()),
            )),
    );
    let warning_field: BoxedWidget = Box::new(
        RawView::new(column(theme.spacing_small))
            .child(Box::new(TextInput::new("", |_| {}).border(theme.warning)))
            .child(Box::new(
                Text::new("Warning: verify this value")
                    .align(TextAlign::Start)
                    .color(theme.warning)
                    .style(label_style()),
            )),
    );

    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader title={"Input".to_owned()} subtitle={"Write, select, and edit. Each field keeps its own content.".to_owned()} />
            {Box::new(
                card(theme.spacing_large)
                    .child(stacked_field("Default", Box::new(jsx!{<TextInput controller={&plain} />})))
                    .child(stacked_field("With placeholder", Box::new(jsx!{<TextInput controller={&with_placeholder} placeholder={"Type something…".to_owned()} />})))
            ) as BoxedWidget}
            {Box::new(
                card(theme.spacing_medium)
                    .child(field_label("Validation states"))
                    .child(card_row(vec![error_field, warning_field]))
            ) as BoxedWidget}
            {Box::new(
                card(theme.spacing_medium)
                    .child(field_label("Text area"))
                    .child(card_row(vec![
                        stacked_field("Horizontal scrolling", Box::new(jsx!{<TextArea controller={&notes} style={textarea_style()} placeholder={"Notes…".to_owned()} />})),
                        stacked_field("Wrap to fit", Box::new(jsx!{<TextArea controller={&notes_wrapped} style={textarea_style()} wrap={true} placeholder={"Notes…".to_owned()} />})),
                    ]))
            ) as BoxedWidget}
        </RawView>
    })
}
