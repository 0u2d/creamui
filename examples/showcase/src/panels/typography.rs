use crate::prelude::*;

/// A fixed-width, secondary-colored row caption — like [`field_label`] but
/// sized to sit beside its control in a row instead of stacked above it
/// full-width.
fn row_caption(text: &str, width: f32) -> BoxedWidget {
    Box::new(Text::secondary(text).align(TextAlign::Start).style(Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(width),
            height: Dimension::Length(18.0),
        },
        flex_shrink: 0.0,
        ..Default::default()
    }))
}

/// The "Typography" panel: the heading scale (h1-h5) plus one heading per
/// semantic theme color, then every inline text treatment — weight, slant,
/// underline, strikethrough, a blockquote, a preformatted code block, and
/// clickable links.
#[component]
pub fn TypographyPanel(link_clicks: Signal<i32>) -> BoxedWidget {
    let theme = use_theme();
    let row_style = Style {
        align_items: Some(AlignItems::Center),
        ..row(theme.spacing_medium)
    };

    let sizes = [
        (TextSize::Xl, "Xl · h1"),
        (TextSize::Lg, "Lg · h2"),
        (TextSize::Md, "Md · h3"),
        (TextSize::Sm, "Sm · h4"),
        (TextSize::Xs, "Xs · h5"),
    ];
    let mut scale_children: Vec<BoxedWidget> = vec![field_label("Heading scale")];
    for (size, label) in sizes {
        scale_children.push(Box::new(
            RawView::new(row_style.clone())
                .child(row_caption(label, 60.0))
                .child(Box::new(Heading::sized(size, "Heading"))),
        ));
    }

    let semantic_colors: [(&str, Color); 7] = [
        ("Primary", theme.text_primary),
        ("Secondary", theme.text_secondary),
        ("Disabled", theme.text_disabled),
        ("Accent", theme.accent),
        ("Danger", theme.danger),
        ("Warning", theme.warning),
        ("Success", theme.success),
    ];
    let mut color_children: Vec<BoxedWidget> =
        vec![field_label("Heading · one per semantic color")];
    for (label, color) in semantic_colors {
        color_children.push(Box::new(
            RawView::new(row_style.clone())
                .child(row_caption(label, 78.0))
                .child(Box::new(Heading::new("The quick brown fox").color(color))),
        ));
    }

    const SAMPLE: &str = "The quick brown fox jumps over the lazy dog.";
    let mut text_children: Vec<BoxedWidget> = vec![field_label("Text · weight and decoration")];
    for (label, text) in [
        ("Normal", Text::new(SAMPLE).align(TextAlign::Start)),
        ("Bold", Text::new(SAMPLE).align(TextAlign::Start).bold(true)),
        (
            "Italic",
            Text::new(SAMPLE).align(TextAlign::Start).italic(true),
        ),
        (
            "Underline",
            Text::new(SAMPLE).align(TextAlign::Start).underline(true),
        ),
        (
            "Strikethrough",
            Text::new(SAMPLE)
                .align(TextAlign::Start)
                .strikethrough(true),
        ),
    ] {
        text_children.push(Box::new(
            RawView::new(row_style.clone())
                .child(row_caption(label, 100.0))
                .child(Box::new(text)),
        ));
    }

    const CODE: &str = "fn main() {\n    println!(\"Hello, CreamUI!\");\n}";
    let clicks = link_clicks.get();
    let link_a = link_clicks.clone();
    let link_b = link_clicks.clone();

    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader title={"Typography".to_owned()} subtitle={"Every heading size and semantic color, plus every inline text treatment.".to_owned()} />
            {Box::new(card(theme.spacing_medium).with_children(scale_children)) as BoxedWidget}
            {Box::new(card(theme.spacing_medium).with_children(color_children)) as BoxedWidget}
            {Box::new(card(theme.spacing_medium).with_children(text_children)) as BoxedWidget}
            {field_card("Quote", Box::new(Quote::new("Design is not just what it looks like and feels like. Design is how it works.")))}
            {field_card("Pre / code", Box::new(Pre::new(CODE)))}
            {Box::new(
                card(theme.spacing_medium)
                    .child(field_label("Links"))
                    .child(Box::new(
                        RawView::new(row(theme.spacing_large))
                            .child(Box::new(Link::new("Documentation", move || link_a.update(|v| *v += 1))))
                            .child(Box::new(Link::new("Source on GitHub", move || link_b.update(|v| *v += 1)))),
                    ))
                    .child(Box::new(Text::secondary(format!("{clicks} link clicks")).align(TextAlign::Start)))
            ) as BoxedWidget}
        </RawView>
    })
}
