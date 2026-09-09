//! Live examples of CreamUI's grid layout primitives.
//!
//! Every cell sizes itself from its parent natively — percentage widths and
//! a `repeat(auto-fit, minmax(..))` grid template — instead of a pixel
//! width computed by hand in Rust. Resize the window: the column count
//! recomputes as part of ordinary layout.

use crate::prelude::*;

const GAP: f32 = 14.0;
const GALLERY_MIN_CARD_WIDTH: f32 = 176.0;
const GALLERY_CARD_HEIGHT: f32 = 96.0;

/// A card that fills whatever grid cell it's placed in, rather than
/// carrying its own pixel dimensions.
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
            .gap(GAP)
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
            .gap(GAP)
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

/// Grid examples: a bento box (explicit spans) and an auto-fit gallery —
/// every cell sized from its parent, none of it carrying a pixel width
/// computed by hand.
#[allow(non_snake_case)]
pub fn GridPanel() -> BoxedWidget {
    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader
                title={"Grid".to_owned()}
                subtitle={"Grid containers that size themselves from whatever space their parent actually has.".to_owned()}
            />
            {field_card("Grid · GridItem::at · column_span · row_span · fr tracks", bento_grid())}
            {field_card("Grid::auto_fit_columns · repeat(auto-fit, minmax(176px, 1fr))", gallery_grid())}
        </RawView>
    })
}
