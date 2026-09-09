use crate::prelude::*;

/// The "Sidebar" panel: a small, self-contained `Sidebar`/`SidebarItem` demo
/// with its own selection state, next to the panel it controls.
#[component]
pub fn SidebarPanel(active: Signal<usize>) -> BoxedWidget {
    let theme = use_theme();
    const ITEMS: [&str; 3] = ["Inbox", "Drafts", "Sent"];
    let colors = TabColors::sidebar();
    let item_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(36.0),
            },
            align_items: Some(AlignItems::Center),
            ..Default::default()
        },
        theme.spacing_medium,
    );
    let rail_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(176.0),
                height: Dimension::Percent(1.0),
            },
            flex_shrink: 0.0,
            ..column(theme.spacing_small)
        },
        theme.spacing_medium,
    );
    let mut rail = Sidebar::new(colors, rail_style);
    for (index, label) in ITEMS.iter().enumerate() {
        let is_active = active.get() == index;
        let select = active.clone();
        rail = rail.child(Box::new(SidebarItem::new(
            colors,
            item_style.clone(),
            *label,
            is_active,
            move || select.set(index),
        )));
    }
    let preview_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(264.0),
                height: Dimension::Percent(1.0),
            },
            ..column(theme.spacing_medium)
        },
        theme.spacing_large,
    );
    let current_label = ITEMS[active.get()].to_owned();
    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader title={"Sidebar".to_owned()} subtitle={"A compact navigation rail with independent selection.".to_owned()} />
            {Box::new(Surface::new(SurfaceRole::Inset, padding(Style {
                size: creamui_core::layout::Size { width: Dimension::Length(488.0), height: Dimension::Length(192.0) },
                align_items: Some(AlignItems::Stretch),
                ..row(theme.spacing_large)
            }, theme.spacing_medium))
                .child(Box::new(rail))
                .child(Box::new(Card::new(preview_style)
                    .child(Box::new(Heading::new(current_label.clone())))
                    .child(Box::new(Text::secondary("The selected section is shown here.")))))
            ) as BoxedWidget}
        </RawView>
    })
}
