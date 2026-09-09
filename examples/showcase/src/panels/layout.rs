//! Live examples of CreamUI's flex and grid layout primitives.
//!
//! Every container on this page sizes itself from its parent natively —
//! percentage widths, `flex-wrap`, and a `repeat(auto-fit, minmax(..))`
//! grid template — instead of a pixel width computed by hand in Rust.
//! Resize the window: the column counts and wrap points recompute as part
//! of ordinary layout, with no measured-width prop threaded down from the
//! app and no per-frame Rust arithmetic to keep in sync.

use crate::prelude::*;

const GRID_GAP: f32 = 14.0;
const GALLERY_MIN_CARD_WIDTH: f32 = 176.0;
const GALLERY_CARD_HEIGHT: f32 = 96.0;
const FLEX_CARD_MIN_WIDTH: f32 = 200.0;
const FLEX_CARD_HEIGHT: f32 = 132.0;

/// A card that fills whatever box it's placed in — a grid cell, a flex
/// item's resolved size — rather than carrying its own pixel dimensions.
fn demo_card(title: &str, description: &str, accent: Color) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        Flex::column()
            .fill()
            .gap(theme.spacing_small)
            .padding(theme.spacing_large)
            .background(theme.surface_elevated)
            .corner_radius(theme.card_radius)
            .child(Box::new(RawText::new(title, accent, 16.0).bold(true)))
            .child(Box::new(
                RawText::new(description, theme.text_secondary, 13.0)
                    .layout_style(Style::default().grow(1.0)),
            )),
    )
}

/// A fixed-height gallery card that stretches to whatever width its
/// `auto_fit_columns` track resolves to.
fn gallery_card(title: &str, description: &str, accent: Color) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        Flex::column()
            .full_width()
            .height(GALLERY_CARD_HEIGHT)
            .gap(6.0)
            .padding(theme.spacing_medium)
            .background(theme.surface_elevated)
            .corner_radius(theme.card_radius)
            .child(Box::new(RawText::new(title, accent, 15.0).bold(true)))
            .child(Box::new(RawText::new(
                description,
                theme.text_secondary,
                12.0,
            ))),
    )
}

const LAYOUT_TABS: [&str; 2] = ["Grid", "Flex"];

fn layout_tab_bar(controller: TabController) -> BoxedWidget {
    let theme = use_theme();
    let mut colors = TabColors::dark();
    colors.gap = theme.spacing_medium;
    let styles = tab_styles(&LAYOUT_TABS, TabSizing::Content, 36.0, theme.spacing_medium);
    let bar_style = Style {
        flex_shrink: 0.0,
        align_self: Some(creamui_core::layout::AlignSelf::Start),
        ..row(colors.gap)
    };
    let mut bar = Tabs::new(colors, bar_style);
    for (index, label) in LAYOUT_TABS.iter().enumerate() {
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

/// A bento box: three explicit cells on a 3-column, 2-row `fr` grid. Spans
/// are the one thing that genuinely needs an explicit template — there's no
/// way to say "twice as wide as your neighbor" without naming a track — but
/// every cell still just says `fill()` and lets the grid resolve the actual
/// pixels.
fn bento_grid() -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        Grid::new()
            .full_width()
            .height(220.0)
            .gap(GRID_GAP)
            .template_columns([Track::fr(1.0), Track::fr(1.0), Track::fr(1.0)])
            .template_rows([Track::fr(1.0), Track::fr(1.0)])
            .child(Box::new(
                GridItem::new()
                    .at(1, 1)
                    .column_span(2)
                    .row_span(2)
                    .child(demo_card(
                        "Overview",
                        "Column 1, row 1 · spans 2 columns and 2 rows.",
                        theme.accent,
                    )),
            ))
            .child(Box::new(GridItem::new().at(3, 1).child(demo_card(
                "Cell",
                "Targets one explicit cell.",
                Color::rgb(0x8b, 0x5c, 0xf6),
            ))))
            .child(Box::new(GridItem::new().at(3, 2).child(demo_card(
                "Tracks",
                "Three 1fr columns share the width evenly.",
                Color::rgb(0x22, 0xc5, 0x5e),
            )))),
    )
}

/// A card gallery on `repeat(auto-fit, minmax(176px, 1fr))`: as many
/// 176px-or-wider columns as fit the row, sharing whatever's left evenly,
/// collapsing unused tracks instead of leaving gaps. Taffy resolves the
/// column count during layout — this function never sees a pixel width.
fn gallery_grid() -> BoxedWidget {
    let cards = [
        (
            "Auto-fit",
            "repeat(auto-fit, minmax(176px, 1fr)) — no column count in Rust.",
            None,
        ),
        (
            "Shrink-safe",
            "min-size defaults to 0, so a card never overflows a tight row.",
            Some(Color::rgb(0x8b, 0x5c, 0xf6)),
        ),
        (
            "Stretch",
            "Each column shares the remaining width evenly.",
            Some(Color::rgb(0x22, 0xc5, 0x5e)),
        ),
        (
            "Resize me",
            "Resize the window — the column count recomputes during layout.",
            Some(Color::rgb(0xf5, 0x9e, 0x0b)),
        ),
        (
            "Collapses",
            "Too few cards for a row, and the extra track is just unused.",
            Some(Color::rgb(0xec, 0x48, 0x99)),
        ),
        (
            "No overlap",
            "Each card is a plain vertical Flex column underneath.",
            Some(Color::rgb(0x06, 0xb6, 0xd4)),
        ),
    ];
    let theme = use_theme();
    Box::new(
        Grid::new()
            .full_width()
            .auto_fit_columns(GALLERY_MIN_CARD_WIDTH)
            .gap(GRID_GAP)
            .with_children(
                cards
                    .into_iter()
                    .map(|(title, description, accent)| {
                        Box::new(GridItem::new().child(gallery_card(
                            title,
                            description,
                            accent.unwrap_or(theme.accent),
                        ))) as BoxedWidget
                    })
                    .collect(),
            ),
    )
}

/// A wrapping flex row: each card is `flex: 1 1 200px` with an explicit
/// `min_width` opting back into "never shrink below this" — the one case on
/// this page where that CSS default is actually what you want, since a
/// narrower card would clip its own title.
fn flex_wrap_row() -> BoxedWidget {
    let theme = use_theme();
    let cards = [
        (
            "Wrap",
            "Cards flow onto another line once the row runs out of width.",
            theme.accent,
        ),
        (
            "Gap",
            "Horizontal and vertical gaps can be set independently.",
            Color::rgb(0x8b, 0x5c, 0xf6),
        ),
        (
            "Alignment",
            "Items can be centered on both axes.",
            Color::rgb(0x22, 0xc5, 0x5e),
        ),
        (
            "Grow",
            "flex: 1 1 200px — grows to fill the row, wraps once it can't.",
            Color::rgb(0xf5, 0x9e, 0x0b),
        ),
    ];
    Box::new(
        Flex::row()
            .full_width()
            .gap(GRID_GAP)
            .wrap(Wrap::Wrap)
            .align_content(Justify::Start)
            .with_children(
                cards
                    .into_iter()
                    .map(|(title, description, accent)| {
                        Box::new(
                            Flex::column()
                                .basis(FLEX_CARD_MIN_WIDTH)
                                .grow(1.0)
                                .min_width(FLEX_CARD_MIN_WIDTH)
                                .height(FLEX_CARD_HEIGHT)
                                .gap(8.0)
                                .padding(16.0)
                                .background(theme.surface_elevated)
                                .corner_radius(theme.card_radius)
                                .child(Box::new(
                                    RawText::new(title, accent, 17.0).bold(true),
                                ))
                                .child(Box::new(RawText::new(
                                    description,
                                    theme.text_secondary,
                                    13.0,
                                ))),
                        ) as BoxedWidget
                    })
                    .collect(),
            ),
    )
}

/// A toolbar: `justify-content: space-between` with centered items,
/// stretched to the panel's width instead of a pixel width.
fn toolbar() -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        Flex::row()
            .full_width()
            .height(50.0)
            .align(Align::Center)
            .justify(Justify::Between)
            .padding(theme.spacing_medium)
            .background(theme.surface_elevated)
            .corner_radius(theme.card_radius)
            .child(Box::new(
                Text::new("Flex, without the style boilerplate").font_size(17.0),
            ))
            .child(Box::new(
                RawText::new("display: flex", theme.accent, 13.0)
                    .layout_style(Style::default().padding_all(8.0)),
            )),
    )
}

/// Flex and grid examples, split into a Grid tab (bento box, auto-fit
/// gallery) and a Flex tab (wrapping row, toolbar) — every container sized
/// from its parent, none of them carrying a pixel width computed by hand.
#[component]
pub fn LayoutPanel(layout_tab: TabController) -> BoxedWidget {
    let grid_selected = layout_tab.selected() == 0;
    let content: BoxedWidget = if grid_selected {
        Box::new(
            RawView::new(column(section_gap()))
                .child(field_card(
                    "Grid · GridItem::at · column_span · row_span · fr tracks",
                    bento_grid(),
                ))
                .child(field_card(
                    "Grid::auto_fit_columns · repeat(auto-fit, minmax(176px, 1fr))",
                    gallery_grid(),
                )),
        )
    } else {
        Box::new(
            RawView::new(column(section_gap()))
                .child(field_card(
                    "Flex::row · Wrap::Wrap · gap · align_content",
                    flex_wrap_row(),
                ))
                .child(field_card("Toolbar · justify-content · align-items", toolbar())),
        )
    };
    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader
                title={"Layout".to_owned()}
                subtitle={"Flex and grid containers that size themselves from whatever space their parent actually has.".to_owned()}
            />
            {layout_tab_bar(layout_tab)}
            {content}
        </RawView>
    })
}
