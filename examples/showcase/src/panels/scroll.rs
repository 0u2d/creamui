use crate::prelude::*;

/// A row of numbered list items long enough to overflow a fixed-height
/// scroll view, used by both lists in [`ScrollPanel`].
fn scroll_rows(
    theme: &Theme,
    count: usize,
    row_style: Style,
    text_color: Color,
) -> Vec<BoxedWidget> {
    let text_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Percent(1.0),
        },
        ..Default::default()
    };
    (0..count)
        .map(|i| {
            let background = if i % 2 == 0 {
                theme.surface_elevated
            } else {
                theme.surface
            };
            Box::new(
                RawView::new(row_style.clone())
                    .background(background)
                    .child(Box::new(
                        RawText::new(format!("Row {:02}", i + 1), text_color, 13.0)
                            .align(TextAlign::Start)
                            .layout_style(text_style.clone()),
                    )),
            ) as BoxedWidget
        })
        .collect()
}

/// The "Scroll" panel: a themed `ScrollView` and a hand-colored
/// `RawScrollView` side by side, each holding a long enough list to show
/// off the draggable `RawScrollbar` overlay — drag either thumb, or turn
/// the mouse wheel over either list, and they stay in sync.
#[component]
pub fn ScrollPanel(
    theme: Theme,
    themed_scroll: ScrollController,
    custom_scroll: ScrollController,
) -> BoxedWidget {
    const ROWS: usize = 28;
    let list_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(240.0),
            height: Dimension::Length(280.0),
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    let row_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(34.0),
            },
            align_items: Some(AlignItems::Center),
            ..Default::default()
        },
        theme.spacing_medium,
    );

    let themed_list =
        ScrollView::controlled(&theme, list_style.clone(), themed_scroll).with_children(
            scroll_rows(&theme, ROWS, row_style.clone(), theme.text_primary),
        );

    const NEON: Color = Color::rgb(0x5c, 0xe1, 0xff);
    let custom_list = RawScrollView::controlled(list_style, custom_scroll)
        .background(Color::rgb(0x0c, 0x14, 0x1a))
        .corner_radius(theme.radius_medium)
        .scrollbar_width(7.0)
        .scrollbar_color(Color::rgba(NEON.r, NEON.g, NEON.b, 150))
        .scrollbar_hover_color(Color::rgba(NEON.r, NEON.g, NEON.b, 220))
        .with_children(scroll_rows(
            &theme,
            ROWS,
            row_style,
            Color::rgb(0xbf, 0xef, 0xff),
        ));

    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Scroll".to_owned()} subtitle={"A draggable scrollbar thumb tracks the mouse wheel automatically, and vice versa.".to_owned()} />
            {card_row(&theme, vec![
                field_card(&theme, "Themed · ScrollView", Box::new(themed_list)),
                field_card(&theme, "Custom · RawScrollView", Box::new(custom_list)),
            ])}
        </RawView>
    })
}
