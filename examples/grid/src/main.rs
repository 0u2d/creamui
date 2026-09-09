//! Two grid patterns: an explicit Bento dashboard and an adaptive gallery.
//!
//! The first card occupies a 2×2 area while the side cards target individual
//! cells. The gallery derives its column count from the current viewport, so
//! cards fill a row until the next one would be narrower than its minimum.

use creamui_core::{BoxedWidget, Size};
use creamui_render::{run, WindowOptions};
use creamui_theme::{use_theme, Color, Theme};
use creamui_widgets::layout::{Align, Flex, Grid, GridItem, Justify, StyleExt, Track};
use creamui_widgets::{RawText, Text};

const PAGE_PADDING: f32 = 24.0;
const GRID_GAP: f32 = 14.0;
const MIN_GALLERY_CARD_WIDTH: f32 = 176.0;

fn card(title: &'static str, description: &'static str, accent: Color) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        Flex::column()
            .fill()
            .gap(10.0)
            .padding(18.0)
            .background(theme.surface_elevated)
            .corner_radius(theme.card_radius)
            .child(Box::new(RawText::new(title, accent, 18.0).bold(true)))
            .child(Box::new(
                RawText::new(description, theme.text_secondary, 13.0)
                    .layout_style(creamui_core::layout::Style::default().grow(1.0)),
            )),
    )
}

fn gallery_card(title: &'static str, description: &'static str, accent: Color) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        Flex::column()
            .full_width()
            .gap(6.0)
            .padding(14.0)
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

fn main() {
    run(
        WindowOptions {
            title: "CreamUI — Grid layout".into(),
            width: 900,
            height: 620,
            theme: Theme::dark(),
            ..Default::default()
        },
        Theme::dark().surface,
        |_| {},
        move |viewport: Size| -> BoxedWidget {
            let theme = use_theme();
            let content_width = (viewport.width - PAGE_PADDING * 2.0).max(0.0);
            let gallery_columns = ((content_width + GRID_GAP) / (MIN_GALLERY_CARD_WIDTH + GRID_GAP))
                .floor()
                .max(1.0) as usize;

            let header = Flex::row()
                .align(Align::Center)
                .justify(Justify::Between)
                .child(Box::new(
                    Text::new("Grid, placed deliberately").font_size(24.0),
                ))
                .child(Box::new(
                    RawText::new("display: grid", theme.accent, 13.0)
                        .layout_style(creamui_core::layout::Style::default().padding_all(8.0)),
                ));

            let bento = Grid::new()
                .size(content_width, 220.0)
                .gap(GRID_GAP)
                .template_columns([Track::fr(1.0), Track::fr(1.0), Track::fr(1.0)])
                .template_rows([Track::fr(1.0), Track::fr(1.0)])
                .child(Box::new(
                    GridItem::new()
                        .at(1, 1)
                        .column_span(2)
                        .row_span(2)
                        .child(card(
                        "Overview",
                        "This item starts at column 1, row 1 and spans two columns and two rows.",
                        theme.accent,
                    )),
                ))
                .child(Box::new(GridItem::new().at(3, 1).child(card(
                    "Cell",
                    "An item can target one explicit cell.",
                    Color::rgb(0x8b, 0x5c, 0xf6),
                ))))
                .child(Box::new(GridItem::new().at(3, 2).child(card(
                    "Tracks",
                    "Three proportional fr columns share the available width.",
                    Color::rgb(0x22, 0xc5, 0x5e),
                ))));

            let gallery_heading = Flex::row()
                .align(Align::Center)
                .justify(Justify::Between)
                .child(Box::new(Text::new("Adaptive gallery").font_size(16.0)))
                .child(Box::new(RawText::new(
                    format!("{gallery_columns} columns · min {MIN_GALLERY_CARD_WIDTH:.0}px"),
                    theme.text_disabled,
                    12.0,
                )));

            let gallery = Grid::new()
                .grow(1.0)
                .columns(gallery_columns)
                .gap(GRID_GAP)
                .child(Box::new(GridItem::new().min_width(0.0).child(
                    gallery_card(
                        "Auto flow",
                        "Items fill rows from left to right.",
                        theme.accent,
                    ),
                )))
                .child(Box::new(GridItem::new().min_width(0.0).child(
                    gallery_card(
                        "Minimum",
                        "Columns are added only when a card stays readable.",
                        Color::rgb(0x8b, 0x5c, 0xf6),
                    ),
                )))
                .child(Box::new(GridItem::new().min_width(0.0).child(
                    gallery_card(
                        "Stretch",
                        "Each fr track shares the remaining width.",
                        Color::rgb(0x22, 0xc5, 0x5e),
                    ),
                )))
                .child(Box::new(GridItem::new().min_width(0.0).child(
                    gallery_card(
                        "Resize",
                        "Resize the window to recompute the column count.",
                        Color::rgb(0xf5, 0x9e, 0x0b),
                    ),
                )))
                .child(Box::new(GridItem::new().min_width(0.0).child(
                    gallery_card(
                        "Next row",
                        "Extra cards continue below the current row.",
                        Color::rgb(0xec, 0x48, 0x99),
                    ),
                )))
                .child(Box::new(GridItem::new().min_width(0.0).child(
                    gallery_card(
                        "No overlap",
                        "Card content uses a vertical flex flow.",
                        Color::rgb(0x06, 0xb6, 0xd4),
                    ),
                )));

            let footer = Flex::row()
                .justify(Justify::End)
                .child(Box::new(RawText::new(
                    "Bento · Auto-flow · fr tracks · StyleExt",
                    theme.text_disabled,
                    12.0,
                )));

            Box::new(
                Flex::column()
                    .size(viewport.width, viewport.height)
                    .gap(GRID_GAP)
                    .padding(PAGE_PADDING)
                    .background(theme.surface)
                    .child(Box::new(header))
                    .child(Box::new(bento))
                    .child(Box::new(gallery_heading))
                    .child(Box::new(gallery))
                    .child(Box::new(footer)),
            )
        },
    );
}
