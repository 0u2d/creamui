//! Live examples of CreamUI's flex layout primitives.
//!
//! Every container sizes itself from its parent natively — percentage
//! widths and `flex-wrap` — instead of a pixel width computed by hand in
//! Rust. Resize the window: the wrap points recompute as part of ordinary
//! layout.

use crate::prelude::*;

const GAP: f32 = 14.0;
const CARD_MIN_WIDTH: f32 = 200.0;
const CARD_HEIGHT: f32 = 132.0;

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
            .gap(GAP)
            .wrap(Wrap::Wrap)
            .align_content(Justify::Start)
            .with_children(
                cards
                    .into_iter()
                    .map(|(title, description, accent)| {
                        Box::new(
                            Flex::column()
                                .basis(CARD_MIN_WIDTH)
                                .grow(1.0)
                                .min_width(CARD_MIN_WIDTH)
                                .height(CARD_HEIGHT)
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

/// Flex examples: a wrapping card row and a toolbar — every container
/// stretches or shrinks with its parent instead of a pixel width computed
/// by hand.
#[allow(non_snake_case)]
pub fn FlexPanel() -> BoxedWidget {
    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader
                title={"Flex".to_owned()}
                subtitle={"Flex containers that size themselves from whatever space their parent actually has.".to_owned()}
            />
            {field_card("Flex::row · Wrap::Wrap · gap · align_content", flex_wrap_row())}
            {field_card("Toolbar · justify-content · align-items", toolbar())}
        </RawView>
    })
}
