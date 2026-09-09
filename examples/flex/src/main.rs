//! A compact gallery for CreamUI's semantic flex layout API.

use creamui_core::{BoxedWidget, Size};
use creamui_render::{run, WindowOptions};
use creamui_theme::{use_theme, Color, Theme};
use creamui_widgets::layout::{Align, Block, Flex, Grid, GridItem, Justify, StyleExt, Track};
use creamui_widgets::{RawText, Text};

fn card(title: &'static str, description: &'static str, accent: Color) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        Block::new()
            .size(190.0, 142.0)
            .padding(16.0)
            .background(theme.surface_elevated)
            .corner_radius(theme.card_radius)
            .child(Box::new(RawText::new(title, accent, 17.0).bold(true)))
            .child(Box::new(
                RawText::new(description, theme.text_secondary, 13.0)
                    .layout_style(creamui_core::layout::Style::default().grow(1.0)),
            )),
    )
}

fn main() {
    run(
        WindowOptions {
            title: "CreamUI — Flex layout".into(),
            width: 760,
            height: 500,
            theme: Theme::dark(),
            ..Default::default()
        },
        Theme::dark().surface,
        |_| {},
        move |viewport: Size| -> BoxedWidget {
            let theme = use_theme();

            let header = Flex::row()
                .align(Align::Center)
                .justify(Justify::Between)
                .child(Box::new(
                    Text::new("Flex, without the style boilerplate").font_size(24.0),
                ))
                .child(Box::new(
                    RawText::new("display: flex", theme.accent, 13.0)
                        .layout_style(creamui_core::layout::Style::default().padding_all(8.0)),
                ));

            let cards = Grid::new()
                .grow(1.0)
                .gap_x(14.0)
                .gap_y(14.0)
                .template_columns([Track::fr(1.0), Track::fr(1.0), Track::fr(1.0)])
                .align_content(Justify::Start)
                .child(Box::new(GridItem::new().child(card(
                    "Grid",
                    "Three proportional columns use fr tracks.",
                    theme.accent,
                ))))
                .child(Box::new(GridItem::new().child(card(
                    "Gap",
                    "Horizontal and vertical gaps are independent.",
                    Color::rgb(0x8b, 0x5c, 0xf6),
                ))))
                .child(Box::new(GridItem::new().child(card(
                    "Placement",
                    "Grid items can opt into an explicit cell or span.",
                    Color::rgb(0x22, 0xc5, 0x5e),
                ))))
                .child(Box::new(GridItem::new().column_span(2).child(card(
                    "Grow",
                    "The grid area takes the remaining height.",
                    Color::rgb(0xf5, 0x9e, 0x0b),
                ))));

            let footer = Flex::row()
                .justify(Justify::End)
                .align(Align::Center)
                .child(Box::new(RawText::new(
                    "Block · Flex · Grid · StyleExt",
                    theme.text_disabled,
                    12.0,
                )));

            Box::new(
                Flex::column()
                    .size(viewport.width, viewport.height)
                    .gap(20.0)
                    .padding(24.0)
                    .background(theme.surface)
                    .child(Box::new(header))
                    .child(Box::new(cards))
                    .child(Box::new(footer)),
            )
        },
    );
}
