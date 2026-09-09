use crate::prelude::*;

#[component]
pub fn ImagesPanel(png: ImageData, jpeg: ImageData, webp: ImageData) -> BoxedWidget {
    let theme = use_theme();
    let square = Style {
        size: fixed(168., 168.),
        flex_shrink: 0.,
        ..Default::default()
    };
    let landscape = Style {
        size: fixed(210., 148.),
        flex_shrink: 0.,
        ..Default::default()
    };
    Box::new(
        RawView::new(column(section_gap()))
            .child(SectionHeader(SectionHeaderProps {
                title: "Images".into(),
                subtitle:
                    "Local PNG, JPEG, and WebP assets, each cropped with a different fit and shape."
                        .into(),
            }))
            .child(card_row(vec![
                field_card(
                    "PNG · square",
                    Box::new(Image::with_style(png, square.clone()).fit(ImageFit::Cover)),
                ),
                field_card(
                    "JPEG · rounded corners",
                    Box::new(
                        Image::with_style(jpeg, landscape)
                            .fit(ImageFit::Cover)
                            .corner_radius(theme.card_radius),
                    ),
                ),
                field_card(
                    "WebP · full circle",
                    Box::new(
                        Image::with_style(webp, square)
                            .fit(ImageFit::Cover)
                            .corner_radius(84.),
                    ),
                ),
            ])),
    )
}
