use crate::prelude::*;

const TAB_LABELS: [&str; 3] = ["Overview", "Activity", "Settings"];
fn tab_bar(
    labels: &[&str],
    controller: TabController,
    colors: TabColors,
    sizing: TabSizing,
    height: f32,
    tab_padding: f32,
    inset: f32,
) -> BoxedWidget {
    let bar_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: if sizing == TabSizing::Fill {
                    Dimension::Percent(1.0)
                } else {
                    Dimension::Auto
                },
                height: Dimension::Auto,
            },
            flex_shrink: 0.0,
            align_self: if sizing == TabSizing::Fill {
                None
            } else {
                Some(creamui_core::layout::AlignSelf::Start)
            },
            ..row(colors.gap)
        },
        inset,
    );
    let styles = tab_styles(labels, sizing, height, tab_padding);
    let mut bar = Tabs::new(colors, bar_style);
    for (index, label) in labels.iter().enumerate() {
        let tabs = controller.clone();
        bar = bar.child(Box::new(Tab::new(
            colors,
            styles[index].clone(),
            *label,
            controller.is_selected(index),
            move || tabs.select(index),
        )));
    }
    Box::new(bar)
}

fn tab_example(label: &str, bar: BoxedWidget) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        RawView::new(column(theme.spacing_small))
            .child(Box::new(
                RawText::new(label, theme.text_secondary, theme.typography.caption)
                    .bold(true)
                    .align(TextAlign::Start),
            ))
            .child(bar),
    )
}

/// Filled, pill, and indicator treatments plus a content-linked tab set.
#[component]
pub fn TabsPanel(
    filled: TabController,
    pill: TabController,
    indicator: TabController,
    content: TabController,
) -> BoxedWidget {
    let theme = use_theme();
    let mut filled_colors = TabColors::dark();
    filled_colors.gap = theme.spacing_medium;

    let mut pill_colors = filled_colors;
    pill_colors.inactive_background = Some(theme.surface_hover);
    pill_colors.active_background = theme.surface_elevated;
    pill_colors.active_text = theme.text_primary;
    pill_colors.radius = 16.0;
    pill_colors.container_radius = 20.0;
    pill_colors.gap = theme.spacing_medium;

    let mut indicator_colors = filled_colors;
    indicator_colors.background = theme.surface;
    indicator_colors.inactive_background = None;
    indicator_colors.selection = SelectionStyle::Indicator;
    indicator_colors.radius = 0.0;
    indicator_colors.container_radius = 0.0;
    indicator_colors.gap = theme.spacing_large;

    let content_selected = content.selected().min(TAB_LABELS.len() - 1);
    let preview_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Auto,
            },
            flex_grow: 1.0,
            ..column(theme.spacing_small)
        },
        theme.spacing_large,
    );
    let current_label = TAB_LABELS[content_selected];
    let (title, detail) = match content_selected {
        0 => (
            "Everything in one place",
            "A calm overview of what matters right now.",
        ),
        1 => (
            "You are all caught up",
            "New activity will appear here as it happens.",
        ),
        _ => (
            "Make it yours",
            "Preferences stay close without leaving this view.",
        ),
    };
    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader title={"Tabs".to_owned()} subtitle={"Three visual styles, followed by a tab bar connected to its content.".to_owned()} />
            {Box::new(
                RawView::new(column(theme.spacing_large))
                    .child(tab_example("Filled tabs · content width", tab_bar(&TAB_LABELS, filled, filled_colors, TabSizing::Content, 38.0, theme.spacing_medium, theme.spacing_small)))
                    .child(tab_example("Pill tabs · equal width", tab_bar(&TAB_LABELS, pill, pill_colors, TabSizing::Equal, 34.0, theme.spacing_medium, theme.spacing_small)))
                    .child(tab_example("Indicator tabs · content width", tab_bar(&TAB_LABELS, indicator, indicator_colors, TabSizing::Content, 34.0, theme.spacing_medium, theme.spacing_small)))
            ) as BoxedWidget}
            {Box::new(Surface::new(SurfaceRole::Inset, padding(Style {
                size: creamui_core::layout::Size { width: Dimension::Length(488.0), height: Dimension::Length(184.0) },
                ..column(theme.spacing_large)
            }, theme.spacing_medium))
                .child(tab_bar(&TAB_LABELS, content, filled_colors, TabSizing::Equal, 36.0, theme.spacing_medium, theme.spacing_small))
                .child(Box::new(View::new(preview_style)
                    .child(Box::new(Heading::new(title)))
                    .child(Box::new(Text::secondary(detail)))
                    .child(Box::new(RawText::new(current_label, theme.accent, 12.).bold(true).align(TextAlign::Start)))))
            ) as BoxedWidget}
        </RawView>
    })
}
