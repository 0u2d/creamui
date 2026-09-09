//! Shared design tokens and layout helpers used across every panel.

use crate::prelude::*;

/// Sidebar categories in display order.
pub const NAV_LABELS: [&str; 16] = [
    "Appearance",
    "Typography",
    "Input",
    "Pickers",
    "Images",
    "Button",
    "Slider",
    "Checkbox",
    "Selection",
    "Feedback",
    "Sidebar",
    "Tabs",
    "Scroll",
    "Tree",
    "Table",
    "Layout",
];
pub const ACCENTS: [(&str, Color); 5] = [
    ("Lilac", Color::rgb(181, 139, 255)),
    ("Sky", Color::rgb(118, 192, 255)),
    ("Mint", Color::rgb(105, 218, 166)),
    ("Berry", Color::rgb(248, 135, 181)),
    ("Apricot", Color::rgb(255, 177, 109)),
];

/// Shifts each color channel by `delta`, clamping at the `u8` bounds. Used to
/// derive hover/pressed accent shades from whichever swatch is selected.
pub fn shade(color: Color, delta: i32) -> Color {
    let shift = |c: u8| (c as i32 + delta).clamp(0, 255) as u8;
    Color::rgb(shift(color.r), shift(color.g), shift(color.b))
}

pub fn accent_foreground(color: Color) -> Color {
    let luminance =
        (color.r as f32 * 0.2126 + color.g as f32 * 0.7152 + color.b as f32 * 0.0722) / 255.;
    if luminance > 0.56 {
        Color::rgb(0x2d, 0x29, 0x2b)
    } else {
        Color::rgb(0xff, 0xff, 0xff)
    }
}

pub fn build_theme(dark: bool, accent: Color) -> Theme {
    let mut theme = if dark { Theme::dark() } else { Theme::light() };
    theme.colors.accent = accent;
    theme.colors.accent_hover = shade(accent, if dark { 20 } else { -12 });
    theme.colors.accent_pressed = shade(accent, -26);
    theme.colors.selection_background = accent;
    theme.colors.selection_text = accent_foreground(accent);
    theme
}

pub fn label_style() -> Style {
    Style {
        size: creamui_core::layout::Size {
            width: Dimension::Percent(1.0),
            // Auto so a wrapped second line grows the box instead of overflowing it.
            height: Dimension::Auto,
        },
        ..Default::default()
    }
}

/// Vertical rhythm between the cards inside a panel — wider than the
/// theme's own `spacing_large` so each topic reads as a separate block
/// instead of one continuous, undifferentiated column.
pub fn section_gap() -> f32 {
    let theme = use_theme();
    theme.spacing_large * 1.75
}

/// A full-width block styled to sit visibly inset against the panel's
/// elevated background (see `SurfaceRole::Inset`), so one topic's controls
/// are framed as a single modular unit rather than running directly into
/// the next. `gap` spaces the children stacked inside it.
pub fn card(gap: f32) -> Surface {
    let theme = use_theme();
    Surface::new(
        SurfaceRole::Inset,
        padding(
            Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Auto,
                },
                ..column(gap)
            },
            theme.spacing_large,
        ),
    )
}

/// A muted, left-aligned caption used to label a group of controls within a
/// card (e.g. "Mode", "Accent color").
pub fn field_label(text: impl Into<String>) -> BoxedWidget {
    Box::new(
        Text::secondary(text)
            .align(TextAlign::Start)
            .style(label_style()),
    )
}

/// A caption stacked over one control, without its own card — used to pack
/// several related fields into one bigger card (see [`card`]).
pub fn stacked_field(label: &str, control: BoxedWidget) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        RawView::new(column(theme.spacing_small))
            .child(field_label(label))
            .child(control),
    )
}

/// The most common card shape on this page: one caption over one control.
pub fn field_card(label: &str, control: BoxedWidget) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        card(theme.spacing_small)
            .child(field_label(label))
            .child(control),
    )
}

/// Lays out same-height cards side by side with a comfortable gutter
/// between them.
pub fn card_row(children: Vec<BoxedWidget>) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        RawView::new(Style {
            align_items: Some(AlignItems::Stretch),
            ..row(theme.spacing_large)
        })
        .with_children(children),
    )
}

/// A section heading: a bold title plus a muted one-line description.
#[component]
pub fn SectionHeader(title: String, subtitle: String) -> BoxedWidget {
    let theme = use_theme();
    // Auto height so a wrapped title doesn't overflow into the subtitle.
    let heading_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Auto,
        },
        ..Default::default()
    };
    Box::new(jsx! {
        <RawView style={column(4.0)}>
            <Heading size={TextSize::Xl} style={heading_style}>{title}</Heading>
            <Text color={theme.text_secondary} align={TextAlign::Start} style={label_style()}>{subtitle}</Text>
        </RawView>
    })
}
