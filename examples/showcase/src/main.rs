//! A component showcase: a sidebar switches between a live theme editor
//! ("Appearance") and a gallery view for every themed control CreamUI ships
//! with — inputs, selection controls, feedback, navigation, and overlays.
//!
//! The whole window is driven by a handful of small signals — `dark_mode`,
//! `accent_index`, `active_section`, plus one signal per interactive control
//! — so picking a new accent color or toggling dark/light mode re-renders
//! every panel with the new `Theme` immediately, the same reactive path any
//! other `Signal` change takes.
//!
//! The derived theme itself flows through `use_theme()`: an effect set up
//! in `on_window_ready` watches `dark_mode`/`accent_index` and pushes the
//! recomputed `Theme` via `WindowHandle::set_theme`, so every panel below
//! reads it with `use_theme()` instead of recomputing it locally.

use creamui_core::layout::{AlignItems, Dimension, JustifyContent, Style};
use creamui_core::{BoxedWidget, Size, TextAlign};
use creamui_image::{Image, ImageData, ImageFit};
use creamui_macros::{component, jsx};
use creamui_reactive::{create_effect, Effect, Signal};
use creamui_render::{run, WindowHandle, WindowOptions};
use creamui_theme::{use_theme, Color, SelectionStyle, Theme};
use creamui_widgets::layout::{column, fixed, padding, row};
use creamui_widgets::{
    tab_styles, AlertDialog, Button, ButtonSize, ButtonState, ColorPicker, ColorPickerController,
    DateTime, DateTimeController, DateTimePicker, FilePicker, Heading, Link, ListBox, Popover,
    Pre, ProgressBar, ProgressRing, Quote, RadioGroup, RawScrollView, RawText, RawView,
    ScrollController, ScrollView, SegmentedControl, Select, SelectController, Sidebar,
    SidebarItem, Switch, Tab, TabColors, TabController, TabSizing, Table, TableColumn, Tabs, Text,
    TextController, TextInput, TextSize, TreeController, TreeNode, TreeView, View,
};
use std::cell::RefCell;
use std::rc::Rc;
use creamui_widgets::{Choice, Icon, NavigationItem, Surface, SurfaceRole, Symbol};

/// Sidebar categories in display order.
const NAV_LABELS: [&str; 15] = [
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
];
const ACCENTS: [(&str, Color); 5] = [
    ("Lilac", Color::rgb(181, 139, 255)),
    ("Sky", Color::rgb(118, 192, 255)),
    ("Mint", Color::rgb(105, 218, 166)),
    ("Berry", Color::rgb(248, 135, 181)),
    ("Apricot", Color::rgb(255, 177, 109)),
];

/// Shifts each color channel by `delta`, clamping at the `u8` bounds. Used to
/// derive hover/pressed accent shades from whichever swatch is selected.
fn shade(color: Color, delta: i32) -> Color {
    let shift = |c: u8| (c as i32 + delta).clamp(0, 255) as u8;
    Color::rgb(shift(color.r), shift(color.g), shift(color.b))
}

fn accent_foreground(color: Color) -> Color {
    let luminance =
        (color.r as f32 * 0.2126 + color.g as f32 * 0.7152 + color.b as f32 * 0.0722) / 255.;
    if luminance > 0.56 {
        Color::rgb(0x2d, 0x29, 0x2b)
    } else {
        Color::rgb(0xff, 0xff, 0xff)
    }
}

fn build_theme(dark: bool, accent: Color) -> Theme {
    let mut theme = if dark { Theme::dark() } else { Theme::light() };
    theme.colors.accent = accent;
    theme.colors.accent_hover = shade(accent, if dark { 20 } else { -12 });
    theme.colors.accent_pressed = shade(accent, -26);
    theme.colors.selection_background = accent;
    theme.colors.selection_text = accent_foreground(accent);
    theme
}

fn label_style() -> Style {
    Style {
        size: creamui_core::layout::Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Length(16.0),
        },
        ..Default::default()
    }
}

/// Vertical rhythm between the cards inside a panel — wider than the
/// theme's own `spacing_large` so each topic reads as a separate block
/// instead of one continuous, undifferentiated column.
fn section_gap(theme: &Theme) -> f32 {
    theme.spacing_large * 1.75
}

/// A full-width block styled to sit visibly inset against the panel's
/// elevated background (see `SurfaceRole::Inset`), so one topic's controls
/// are framed as a single modular unit rather than running directly into
/// the next. `gap` spaces the children stacked inside it.
fn card(theme: &Theme, gap: f32) -> Surface {
    Surface::new(
        theme,
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
fn field_label(theme: &Theme, text: impl Into<String>) -> BoxedWidget {
    Box::new(
        Text::secondary(theme, text)
            .align(TextAlign::Start)
            .style(label_style()),
    )
}

/// A caption stacked over one control, without its own card — used to pack
/// several related fields into one bigger card (see [`card`]).
fn stacked_field(theme: &Theme, label: &str, control: BoxedWidget) -> BoxedWidget {
    Box::new(
        RawView::new(column(theme.spacing_small))
            .child(field_label(theme, label))
            .child(control),
    )
}

/// The most common card shape on this page: one caption over one control.
fn field_card(theme: &Theme, label: &str, control: BoxedWidget) -> BoxedWidget {
    Box::new(
        card(theme, theme.spacing_small)
            .child(field_label(theme, label))
            .child(control),
    )
}

/// Lays out same-height cards side by side with a comfortable gutter
/// between them.
fn card_row(theme: &Theme, children: Vec<BoxedWidget>) -> BoxedWidget {
    Box::new(
        RawView::new(Style {
            align_items: Some(AlignItems::Stretch),
            ..row(theme.spacing_large)
        })
        .with_children(children),
    )
}

/// The showcase category rail.
#[component]
fn Nav(
    theme: Theme,
    active: Signal<usize>,
    content_scroll: ScrollController,
    nav_scroll: ScrollController,
) -> BoxedWidget {
    // Wider than before, and padded almost only on the left: the card gap
    // to its right already separates it from the content panel, so giving
    // it a matching right pad on top of that would just waste width.
    let mut nav = RawView::new(Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(232.),
            height: Dimension::Percent(1.),
        },
        flex_shrink: 0.,
        padding: creamui_core::layout::Rect {
            left: creamui_core::layout::LengthPercentage::Length(16.),
            right: creamui_core::layout::LengthPercentage::Length(6.),
            top: creamui_core::layout::LengthPercentage::Length(16.),
            bottom: creamui_core::layout::LengthPercentage::Length(16.),
        },
        ..column(5.)
    });
    nav = nav.child(Box::new(
        RawView::new(padding(row(8.), 8.))
            .child(Box::new(
                Icon::new(Symbol::Appearance, theme.accent).size(24.),
            ))
            .child(Box::new(
                RawText::new("CreamUI", theme.text_primary, 19.)
                    .bold(true)
                    .align(TextAlign::Start),
            )),
    ));
    let symbols = [
        Symbol::Appearance,
        Symbol::Display,
        Symbol::Keyboard,
        Symbol::Controls,
        Symbol::Display,
        Symbol::Controls,
        Symbol::Sliders,
        Symbol::Check,
        Symbol::Controls,
        Symbol::Display,
        Symbol::Folder,
        Symbol::Grid,
        Symbol::Grid,
        Symbol::Folder,
        Symbol::Grid,
    ];
    let mut items: Vec<BoxedWidget> = Vec::new();
    for (i, label) in NAV_LABELS.iter().enumerate() {
        if i == 0 || i == 2 || i == 8 || i == 10 || i == 13 {
            items.push(Box::new(
                RawView::new(padding(column(0.), 8.)).child(Box::new(
                    RawText::new(
                        if i == 0 {
                            "SHOWCASE"
                        } else if i == 2 {
                            "CONTROLS"
                        } else if i == 8 {
                            "SELECTION"
                        } else if i == 13 {
                            "DATA VIEW"
                        } else {
                            "NAVIGATION"
                        },
                        theme.text_secondary,
                        10.,
                    )
                    .align(TextAlign::Start),
                )),
            ));
        }
        let select = active.clone();
        let reset_scroll = content_scroll.clone();
        items.push(Box::new(NavigationItem::new(
            &theme,
            symbols[i],
            *label,
            active.get() == i,
            move || {
                select.set(i);
                reset_scroll.set(0.0);
            },
        )));
    }
    // Grows to fill whatever space is left between the logo and the footer
    // below, same as the plain spacer `RawView` this replaces — the only
    // difference is that once the item list is taller than that space, it
    // scrolls (draggable thumb included) instead of pushing the footer off
    // the bottom of the window.
    let items_style = Style {
        flex_grow: 1.,
        size: creamui_core::layout::Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Percent(1.0),
        },
        ..Default::default()
    };
    nav = nav
        .child(Box::new(
            RawScrollView::controlled(items_style, nav_scroll)
                .content_gap(5.)
                .scrollbar_gap(6.)
                .with_children(items),
        ))
        .child(Box::new(
            RawView::new(padding(column(5.), 8.))
                .child(Box::new(
                    RawText::new("Component library", theme.text_secondary, 11.)
                        .align(TextAlign::Start),
                ))
                .child(Box::new(
                    RawText::new("CreamUI · 0.1", theme.text_disabled, 11.).align(TextAlign::Start),
                )),
        ));
    Box::new(nav)
}

/// A section heading: a bold title plus a muted one-line description.
#[component]
fn SectionHeader(theme: Theme, title: String, subtitle: String) -> BoxedWidget {
    let heading_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Length(34.0),
        },
        ..Default::default()
    };
    Box::new(jsx! {
        <RawView style={column(4.0)}>
            <Heading theme={&theme} size={TextSize::Xl} style={heading_style}>{title}</Heading>
            <Text theme={&theme} color={theme.text_secondary} align={TextAlign::Start} style={label_style()}>{subtitle}</Text>
        </RawView>
    })
}

/// A pill-shaped selectable button, used for the dark/light and accent
/// pickers on the Appearance page. Selection is fully controlled: it carries
/// no state of its own.
fn pill(theme: &Theme, label: &str, active: bool, on_click: impl Fn() + 'static) -> BoxedWidget {
    Box::new(Choice::new(theme, label, active, on_click))
}

/// A single accent color swatch: a rounded square filled with the color
/// itself, with a ring drawn around whichever one is active.
fn swatch(theme: &Theme, color: Color, active: bool, on_click: impl Fn() + 'static) -> BoxedWidget {
    let outer = Style {
        size: fixed(40.0, 40.0),
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
        flex_shrink: 0.0,
        ..Default::default()
    };
    let inner_size = if active { 28.0 } else { 32.0 };
    let inner = Style {
        size: fixed(inner_size, inner_size),
        ..Default::default()
    };
    let ring = if active {
        theme.text_primary
    } else {
        theme.surface
    };
    Box::new(jsx! {
        <RawButton style={outer} background={ring} corner_radius={20.0} on_click={on_click}>
            <RawView style={inner} background={color} corner_radius={16.0} />
        </RawButton>
    })
}

/// The "Appearance" panel: toggles dark/light mode and picks an accent
/// color, both stored in `Signal`s owned by `main`, so changes are visible
/// immediately across every other section.
#[component]
fn AppearancePanel(
    theme: Theme,
    dark_mode: Signal<bool>,
    accent_index: Signal<usize>,
) -> BoxedWidget {
    let is_dark = dark_mode.get();
    let selected_accent = accent_index.get();

    let dark_flag = dark_mode.clone();
    let light_flag = dark_mode.clone();
    let mode_pills: Vec<BoxedWidget> = vec![
        pill(&theme, "Dark", is_dark, move || dark_flag.set(true)),
        pill(&theme, "Light", !is_dark, move || light_flag.set(false)),
    ];

    let mut swatches: Vec<BoxedWidget> = Vec::new();
    for (index, (_, color)) in ACCENTS.iter().enumerate() {
        let set_accent = accent_index.clone();
        swatches.push(swatch(
            &theme,
            *color,
            index == selected_accent,
            move || set_accent.set(index),
        ));
    }

    let accent_name = ACCENTS[selected_accent].0;
    let summary = format!(
        "{} mode · {} accent",
        if is_dark { "Dark" } else { "Light" },
        accent_name
    );

    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Appearance".to_owned()} subtitle={"Explore the same components in a different light.".to_owned()} />
            {Box::new(
                card(&theme, theme.spacing_large)
                    .child(Box::new(
                        card(&theme, theme.spacing_small)
                            .child(field_label(&theme, "Mode"))
                            .child(Box::new(RawView::new(row(theme.spacing_medium)).with_children(mode_pills))),
                    ))
                    .child(Box::new(
                        card(&theme, theme.spacing_small)
                            .child(field_label(&theme, "Accent color"))
                            .child(Box::new(RawView::new(row(theme.spacing_medium)).with_children(swatches))),
                    ))
                    .child(Box::new(Text::new(&theme, summary).align(TextAlign::Start).color(theme.text_disabled).style(label_style())))
            ) as BoxedWidget}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(Box::new(Heading::new(&theme, "Component preview")))
                    .child(Box::new(Text::secondary(&theme, "Open a category to explore sizes, states, and interactions.").align(TextAlign::Start)))
                    .child(card_row(&theme, vec![
                        Box::new(Button::new(&theme, "Primary", { let mode = dark_mode.clone(); move || mode.update(|v| *v = !*v) })),
                        Box::new(Button::secondary(&theme, ButtonSize::Md, "Secondary", { let mode = dark_mode.clone(); move || mode.update(|v| *v = !*v) })),
                        Box::new(Button::new(&theme, "Disabled", || {}).disabled(true)),
                    ]))
                    .child(Box::new(Text::secondary(&theme, "These preview buttons switch the color scheme.").align(TextAlign::Start)))
            ) as BoxedWidget}
        </RawView>
    })
}

/// A fixed-width, secondary-colored row caption — like [`field_label`] but
/// sized to sit beside its control in a row instead of stacked above it
/// full-width.
fn row_caption(theme: &Theme, text: &str, width: f32) -> BoxedWidget {
    Box::new(
        Text::secondary(theme, text)
            .align(TextAlign::Start)
            .style(Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Length(width),
                    height: Dimension::Length(18.0),
                },
                flex_shrink: 0.0,
                ..Default::default()
            }),
    )
}

/// The "Typography" panel: the heading scale (h1-h5) plus one heading per
/// semantic theme color, then every inline text treatment — weight, slant,
/// underline, strikethrough, a blockquote, a preformatted code block, and
/// clickable links.
#[component]
fn TypographyPanel(theme: Theme, link_clicks: Signal<i32>) -> BoxedWidget {
    let row_style = Style {
        align_items: Some(AlignItems::Center),
        ..row(theme.spacing_medium)
    };

    let sizes = [
        (TextSize::Xl, "Xl · h1"),
        (TextSize::Lg, "Lg · h2"),
        (TextSize::Md, "Md · h3"),
        (TextSize::Sm, "Sm · h4"),
        (TextSize::Xs, "Xs · h5"),
    ];
    let mut scale_children: Vec<BoxedWidget> = vec![field_label(&theme, "Heading scale")];
    for (size, label) in sizes {
        scale_children.push(Box::new(
            RawView::new(row_style.clone())
                .child(row_caption(&theme, label, 60.0))
                .child(Box::new(Heading::sized(&theme, size, "Heading"))),
        ));
    }

    let semantic_colors: [(&str, Color); 7] = [
        ("Primary", theme.text_primary),
        ("Secondary", theme.text_secondary),
        ("Disabled", theme.text_disabled),
        ("Accent", theme.accent),
        ("Danger", theme.danger),
        ("Warning", theme.warning),
        ("Success", theme.success),
    ];
    let mut color_children: Vec<BoxedWidget> =
        vec![field_label(&theme, "Heading · one per semantic color")];
    for (label, color) in semantic_colors {
        color_children.push(Box::new(
            RawView::new(row_style.clone())
                .child(row_caption(&theme, label, 78.0))
                .child(Box::new(
                    Heading::new(&theme, "The quick brown fox").color(color),
                )),
        ));
    }

    const SAMPLE: &str = "The quick brown fox jumps over the lazy dog.";
    let mut text_children: Vec<BoxedWidget> =
        vec![field_label(&theme, "Text · weight and decoration")];
    for (label, text) in [
        ("Normal", Text::new(&theme, SAMPLE).align(TextAlign::Start)),
        (
            "Bold",
            Text::new(&theme, SAMPLE).align(TextAlign::Start).bold(true),
        ),
        (
            "Italic",
            Text::new(&theme, SAMPLE)
                .align(TextAlign::Start)
                .italic(true),
        ),
        (
            "Underline",
            Text::new(&theme, SAMPLE)
                .align(TextAlign::Start)
                .underline(true),
        ),
        (
            "Strikethrough",
            Text::new(&theme, SAMPLE)
                .align(TextAlign::Start)
                .strikethrough(true),
        ),
    ] {
        text_children.push(Box::new(
            RawView::new(row_style.clone())
                .child(row_caption(&theme, label, 100.0))
                .child(Box::new(text)),
        ));
    }

    const CODE: &str = "fn main() {\n    println!(\"Hello, CreamUI!\");\n}";
    let clicks = link_clicks.get();
    let link_a = link_clicks.clone();
    let link_b = link_clicks.clone();

    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Typography".to_owned()} subtitle={"Every heading size and semantic color, plus every inline text treatment.".to_owned()} />
            {Box::new(card(&theme, theme.spacing_medium).with_children(scale_children)) as BoxedWidget}
            {Box::new(card(&theme, theme.spacing_medium).with_children(color_children)) as BoxedWidget}
            {Box::new(card(&theme, theme.spacing_medium).with_children(text_children)) as BoxedWidget}
            {field_card(&theme, "Quote", Box::new(Quote::new(&theme, "Design is not just what it looks like and feels like. Design is how it works.")))}
            {field_card(&theme, "Pre / code", Box::new(Pre::new(&theme, CODE)))}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Links"))
                    .child(Box::new(
                        RawView::new(row(theme.spacing_large))
                            .child(Box::new(Link::new(&theme, "Documentation", move || link_a.update(|v| *v += 1))))
                            .child(Box::new(Link::new(&theme, "Source on GitHub", move || link_b.update(|v| *v += 1)))),
                    ))
                    .child(Box::new(Text::secondary(&theme, format!("{clicks} link clicks")).align(TextAlign::Start)))
            ) as BoxedWidget}
        </RawView>
    })
}

/// The "Input" panel: every `TextInput`/`TextArea` variation side by side —
/// plain, with a placeholder, and a multi-line editor. Each is bound to its
/// own [`TextController`] rather than a hand-wired `value`/`on_change` (and,
/// for the `TextArea`, `cursor`/`selection`) pair — the controller owns that
/// state and the widget just reads and writes through it.
#[component]
fn InputPanel(
    theme: Theme,
    plain: TextController,
    with_placeholder: TextController,
    notes: TextController,
    notes_wrapped: TextController,
) -> BoxedWidget {
    fn textarea_style() -> Style {
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(280.0),
                height: Dimension::Length(180.0),
            },
            ..Default::default()
        }
    }
    let error_field: BoxedWidget = Box::new(
        RawView::new(column(theme.spacing_small))
            .child(Box::new(
                TextInput::new(&theme, "", |_| {}).border(theme.danger),
            ))
            .child(Box::new(
                Text::new(&theme, "Error: this field is required")
                    .align(TextAlign::Start)
                    .color(theme.danger)
                    .style(label_style()),
            )),
    );
    let warning_field: BoxedWidget = Box::new(
        RawView::new(column(theme.spacing_small))
            .child(Box::new(
                TextInput::new(&theme, "", |_| {}).border(theme.warning),
            ))
            .child(Box::new(
                Text::new(&theme, "Warning: verify this value")
                    .align(TextAlign::Start)
                    .color(theme.warning)
                    .style(label_style()),
            )),
    );

    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Input".to_owned()} subtitle={"Write, select, and edit. Each field keeps its own content.".to_owned()} />
            {Box::new(
                card(&theme, theme.spacing_large)
                    .child(stacked_field(&theme, "Default", Box::new(jsx!{<TextInput theme={&theme} controller={&plain} />})))
                    .child(stacked_field(&theme, "With placeholder", Box::new(jsx!{<TextInput theme={&theme} controller={&with_placeholder} placeholder={"Type something…".to_owned()} />})))
            ) as BoxedWidget}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Validation states"))
                    .child(card_row(&theme, vec![error_field, warning_field]))
            ) as BoxedWidget}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Text area"))
                    .child(card_row(&theme, vec![
                        stacked_field(&theme, "Horizontal scrolling", Box::new(jsx!{<TextArea theme={&theme} controller={&notes} style={textarea_style()} placeholder={"Notes…".to_owned()} />})),
                        stacked_field(&theme, "Wrap to fit", Box::new(jsx!{<TextArea theme={&theme} controller={&notes_wrapped} style={textarea_style()} wrap={true} placeholder={"Notes…".to_owned()} />})),
                    ]))
            ) as BoxedWidget}
        </RawView>
    })
}

/// Date/time, color, and file pickers live together because each returns a
/// value chosen from a structured external domain rather than free text.
/// The file picker uses the operating system dialog; the others keep their
/// controlled values in the showcase's regular reactive state.
#[component]
fn PickersPanel(
    theme: Theme,
    date_time: DateTimeController,
    color: Signal<Color>,
    color_picker: ColorPickerController,
    file: Signal<String>,
) -> BoxedWidget {
    let selected_color = color.get();
    let file_label = file.get();
    let set_color = color.clone();
    let set_file = file.clone();
    let color_label = format!(
        "#{:02X}{:02X}{:02X}",
        selected_color.r, selected_color.g, selected_color.b
    );
    let file_caption = if file_label.is_empty() {
        "No file selected yet".to_owned()
    } else {
        file_label.clone()
    };
    Box::new(
        RawView::new(column(section_gap(&theme)))
            .child(SectionHeader(SectionHeaderProps {
                theme,
                title: "Pickers".into(),
                subtitle: "Structured values, controlled by the application and styled from the active theme.".into(),
            }))
            .child(field_card(
                &theme,
                "Date & time · arrows or upper/lower portions adjust it",
                Box::new(DateTimePicker::controlled(&theme, &date_time)),
            ))
            .child(card_row(
                &theme,
                vec![
                    field_card(
                        &theme,
                        &format!("Color · {color_label}"),
                        Box::new(ColorPicker::controlled(&theme, selected_color, &color_picker, move |next| {
                            set_color.set(next)
                        })),
                    ),
                    field_card(
                        &theme,
                        "File · native system dialog",
                        Box::new(
                            RawView::new(column(theme.spacing_small))
                                .child(Box::new(
                                    FilePicker::new(&theme, file_label, move |path| {
                                        set_file.set(path.display().to_string())
                                    })
                                    .title("Choose an asset")
                                    .filter("Images", ["png", "jpg", "jpeg", "webp"]),
                                ))
                                .child(Box::new(
                                    Text::secondary(&theme, file_caption).align(TextAlign::Start),
                                )),
                        ),
                    ),
                ],
            )),
    )
}

#[component]
fn ImagesPanel(theme: Theme, png: ImageData, jpeg: ImageData, webp: ImageData) -> BoxedWidget {
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
        RawView::new(column(section_gap(&theme)))
            .child(SectionHeader(SectionHeaderProps {
                theme,
                title: "Images".into(),
                subtitle:
                    "Local PNG, JPEG, and WebP assets, each cropped with a different fit and shape."
                        .into(),
            }))
            .child(card_row(
                &theme,
                vec![
                    field_card(
                        &theme,
                        "PNG · square",
                        Box::new(Image::with_style(png, square.clone()).fit(ImageFit::Cover)),
                    ),
                    field_card(
                        &theme,
                        "JPEG · rounded corners",
                        Box::new(
                            Image::with_style(jpeg, landscape)
                                .fit(ImageFit::Cover)
                                .corner_radius(theme.card_radius),
                        ),
                    ),
                    field_card(
                        &theme,
                        "WebP · full circle",
                        Box::new(
                            Image::with_style(webp, square)
                                .fit(ImageFit::Cover)
                                .corner_radius(84.),
                        ),
                    ),
                ],
            )),
    )
}

/// The "Button" panel: the default themed `Button`, a couple of
/// semantically-colored variants built straight from `RawButton`, and a
/// disabled-looking one, plus a click counter to prove the handlers fire.
#[component]
fn ButtonPanel(theme: Theme, clicks: Signal<i32>) -> BoxedWidget {
    let mut actions = RawView::new(row(10.));
    for (label, variant) in [
        ("Continue", creamui_widgets::ButtonVariant::Primary),
        ("Cancel", creamui_widgets::ButtonVariant::Secondary),
        ("Learn more", creamui_widgets::ButtonVariant::Tertiary),
        ("Delete", creamui_widgets::ButtonVariant::Destructive),
    ] {
        let clicks = clicks.clone();
        actions = actions.child(Box::new(Button::styled(
            &theme,
            variant,
            ButtonSize::Md,
            label,
            ButtonState::Normal,
            move || clicks.update(|c| *c += 1),
        )));
    }
    let mut sizes = RawView::new(row(10.));
    for (label, size) in [
        ("Extra small", ButtonSize::Xs),
        ("Small", ButtonSize::Sm),
        ("Medium", ButtonSize::Md),
        ("Large", ButtonSize::Lg),
    ] {
        let clicks = clicks.clone();
        sizes = sizes.child(Box::new(Button::secondary(
            &theme,
            size,
            label,
            move || clicks.update(|c| *c += 1),
        )));
    }
    Box::new(
        RawView::new(column(section_gap(&theme)))
            .child(SectionHeader(SectionHeaderProps {
                theme,
                title: "Buttons".into(),
                subtitle: "A clear hierarchy, from everyday actions to important decisions.".into(),
            }))
            .child(Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Variants"))
                    .child(Box::new(actions)),
            ))
            .child(Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "One family, four sizes"))
                    .child(Box::new(sizes)),
            ))
            .child(Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "States"))
                    .child(Box::new(
                        RawView::new(row(10.))
                            .child(Box::new(
                                Button::new(&theme, "Unavailable", || {}).disabled(true),
                            ))
                            .child(Box::new(Button::state(
                                &theme,
                                ButtonSize::Md,
                                "Working",
                                ButtonState::Loading,
                                || {},
                            )))
                            .child(Box::new(Button::state(
                                &theme,
                                ButtonSize::Md,
                                "Saved",
                                ButtonState::Success,
                                || {},
                            ))),
                    ))
                    .child(Box::new(
                        RawText::new(
                            format!("{} actions · Try Tab, then Enter or Space", clicks.get()),
                            theme.text_secondary,
                            12.,
                        )
                        .align(TextAlign::Start),
                    )),
            )),
    )
}

fn slider_row(
    theme: &Theme,
    label: &str,
    value: Signal<f32>,
    format: impl Fn(f32) -> String,
) -> BoxedWidget {
    let current = value.get();
    let set = value.clone();
    Box::new(jsx! {
        <RawView style={column(theme.spacing_small)}>
            <Text theme={theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{label.to_owned()}</Text>
            <Slider theme={theme} value={current} on_change={move |v| set.set(v)} />
            <Text theme={theme} align={TextAlign::Start} color={theme.text_disabled} style={label_style()}>{format(current)}</Text>
        </RawView>
    })
}

/// The "Slider" panel: three independent sliders, each with its live value
/// printed underneath.
#[component]
fn SliderPanel(
    theme: Theme,
    volume: Signal<f32>,
    brightness: Signal<f32>,
    zoom: Signal<f32>,
) -> BoxedWidget {
    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Slider".to_owned()} subtitle={"Fine adjustments with immediate feedback.".to_owned()} />
            {Box::new(
                card(&theme, theme.spacing_large)
                    .child(slider_row(&theme, "Volume", volume, |v| format!("{:.0}%", v * 100.0)))
                    .child(slider_row(&theme, "Brightness", brightness, |v| format!("{:.0}%", v * 100.0)))
                    .child(slider_row(&theme, "Zoom", zoom, |v| format!("{:.2}x", 0.5 + v * 1.5)))
            ) as BoxedWidget}
        </RawView>
    })
}

fn checkbox_row(theme: &Theme, label: &str, checked: Signal<bool>) -> BoxedWidget {
    let is_checked = checked.get();
    let toggle = checked.clone();
    let state_text = if is_checked { "On" } else { "Off" };
    let row_style = Style {
        align_items: Some(AlignItems::Center),
        ..row(theme.spacing_medium)
    };
    let text_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Auto,
            height: Dimension::Length(18.0),
        },
        ..Default::default()
    };
    Box::new(jsx! {
        <RawView style={row_style}>
            <Checkbox theme={theme} checked={is_checked} on_click={move || toggle.update(|c| *c = !*c)} />
            <Text theme={theme} align={TextAlign::Start} style={text_style}>{format!("{} — {}", label, state_text)}</Text>
        </RawView>
    })
}

/// The "Checkbox" panel: three checkboxes, each toggling its own boolean
/// `Signal` and reflecting the current state in its label.
#[component]
fn CheckboxPanel(
    theme: Theme,
    notifications: Signal<bool>,
    auto_save: Signal<bool>,
    beta_features: Signal<bool>,
) -> BoxedWidget {
    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Checkbox".to_owned()} subtitle={"Small preferences, clearly expressed.".to_owned()} />
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Preferences"))
                    .child(checkbox_row(&theme, "Notifications", notifications))
                    .child(checkbox_row(&theme, "Auto-save", auto_save.clone()))
                    .child(checkbox_row(&theme, "Beta features", beta_features))
            ) as BoxedWidget}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Switch presentation"))
                    .child(Box::new(
                        RawView::new(Style { align_items: Some(AlignItems::Center), ..row(theme.spacing_medium) })
                            .child(Box::new(Switch::new(&theme, auto_save.get(), { let set = auto_save.clone(); move || set.update(|value| *value = !*value) })))
                            .child(Box::new(Text::secondary(&theme, "The same boolean, shown as a switch instead of a checkbox.").align(TextAlign::Start))),
                    ))
            ) as BoxedWidget}
        </RawView>
    })
}

/// Select, radio, and segmented selection share a page. `SegmentedControl`
/// owns a row of the lower-level `Choice` items, so the demo does not repeat
/// the same interaction as two competing controls.
#[component]
fn SelectionPanel(
    theme: Theme,
    select: SelectController,
    radio: Signal<usize>,
    segment: Signal<usize>,
    list_scroll: ScrollController,
    list_selected: Signal<usize>,
) -> BoxedWidget {
    const OPTIONS: [&str; 3] = ["System", "Light", "Dark"];
    const FRUITS: [&str; 16] = [
        "Apple",
        "Banana",
        "Cherry",
        "Date",
        "Elderberry",
        "Fig",
        "Grape",
        "Honeydew",
        "Kiwi",
        "Lemon",
        "Mango",
        "Nectarine",
        "Orange",
        "Papaya",
        "Quince",
        "Raspberry",
    ];
    let radio_value = radio.get();
    let segment_value = segment.get();
    let set_radio = radio.clone();
    let set_segment = segment.clone();
    let list_value = list_selected.get();
    let set_list = list_selected.clone();
    let list_box_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(220.0),
            height: Dimension::Length(200.0),
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    let list = ListBox::new(
        &theme,
        list_box_style,
        list_scroll,
        list_value,
        move |index| set_list.set(index),
    )
    .options(&FRUITS);
    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Selection".to_owned()} subtitle={"Choose one value with a popup, explanatory radios, compact Choice segments, or a scrollable list.".to_owned()} />
            {field_card(&theme, "Select / ComboBox", Box::new(Select::controlled(&theme, &OPTIONS, select)))}
            {card_row(&theme, vec![
                field_card(&theme, "Radio group", Box::new(
                    RadioGroup::new(&theme, radio_value, move |index| set_radio.set(index))
                        .option("Keep files on this device")
                        .option("Sync encrypted copies")
                        .option("Never sync"),
                )),
                field_card(&theme, "Segmented control", Box::new(
                    SegmentedControl::new(&theme, segment_value, move |index| set_segment.set(index))
                        .option("Day").option("Week").option("Month"),
                )),
            ])}
            {field_card(&theme, &format!("List box · {}", FRUITS[list_value]), Box::new(list))}
        </RawView>
    })
}

/// Determinate/indeterminate feedback plus a regular floating popover and a
/// modal alert. The alert itself is attached at the root below.
#[component]
fn FeedbackPanel(
    theme: Theme,
    progress: Signal<f32>,
    show_popover: Signal<bool>,
    show_alert: Signal<bool>,
) -> BoxedWidget {
    let value = progress.get();
    let set_progress = progress.clone();
    let open_popover = show_popover.clone();
    let open_alert = show_alert.clone();
    let popover = if show_popover.get() {
        Box::new(
            Popover::new(
                &theme,
                padding(column(theme.spacing_small), theme.spacing_medium),
            )
            .child(Box::new(
                RawText::new("Popover", theme.text_primary, 13.).bold(true),
            ))
            .child(Box::new(RawText::new(
                "A floating surface can contain any widget tree.",
                theme.text_secondary,
                12.,
            ))),
        ) as BoxedWidget
    } else {
        Box::new(RawView::new(Style::default())) as BoxedWidget
    };
    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Feedback & overlays".to_owned()} subtitle={"Show work in progress, surface contextual detail, and ask for confirmation without losing context.".to_owned()} />
            {field_card(&theme, &format!("Determinate progress · {:.0}%", value * 100.0), Box::new(
                RawView::new(column(theme.spacing_small))
                    .child(Box::new(ProgressBar::new(&theme, value)))
                    .child(Box::new(jsx!{<Slider theme={&theme} value={value} on_change={move |next| set_progress.set(next)} />})),
            ))}
            {card_row(&theme, vec![
                field_card(&theme, "Progress ring", Box::new(
                    RawView::new(row(theme.spacing_medium))
                        .child(Box::new(ProgressRing::new(&theme, value).size(32.0)))
                        .child(Box::new(ProgressRing::indeterminate(&theme).size(32.0))),
                )),
                field_card(&theme, "Indeterminate bar", Box::new(ProgressBar::indeterminate(&theme))),
            ])}
            {Box::new(
                card(&theme, theme.spacing_medium)
                    .child(field_label(&theme, "Overlays"))
                    .child(Box::new(
                        RawView::new(row(theme.spacing_medium))
                            .child(Box::new(Button::secondary(&theme, ButtonSize::Md, "Toggle popover", move || open_popover.update(|open| *open = !*open))))
                            .child(Box::new(Button::new(&theme, "Open alert dialog", move || open_alert.set(true)))),
                    ))
                    .child(popover)
            ) as BoxedWidget}
        </RawView>
    })
}

/// The "Sidebar" panel: a small, self-contained `Sidebar`/`SidebarItem` demo
/// with its own selection state, next to the panel it controls.
#[component]
fn SidebarPanel(theme: Theme, active: Signal<usize>) -> BoxedWidget {
    const ITEMS: [&str; 3] = ["Inbox", "Drafts", "Sent"];
    let colors = TabColors::sidebar(&theme);
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
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Sidebar".to_owned()} subtitle={"A compact navigation rail with independent selection.".to_owned()} />
            {Box::new(Surface::new(&theme, SurfaceRole::Inset, padding(Style {
                size: creamui_core::layout::Size { width: Dimension::Length(488.0), height: Dimension::Length(192.0) },
                align_items: Some(AlignItems::Stretch),
                ..row(theme.spacing_large)
            }, theme.spacing_medium))
                .child(Box::new(rail))
                .child(Box::new(View::new(&theme, preview_style)
                    .child(Box::new(Heading::new(&theme, current_label.clone())))
                    .child(Box::new(Text::secondary(&theme, "The selected section is shown here.")))))
            ) as BoxedWidget}
        </RawView>
    })
}

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

fn tab_example(theme: &Theme, label: &str, bar: BoxedWidget) -> BoxedWidget {
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
fn TabsPanel(
    theme: Theme,
    filled: TabController,
    pill: TabController,
    indicator: TabController,
    content: TabController,
) -> BoxedWidget {
    let mut filled_colors = TabColors::dark(&theme);
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
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Tabs".to_owned()} subtitle={"Three visual styles, followed by a tab bar connected to its content.".to_owned()} />
            {Box::new(
                RawView::new(column(theme.spacing_large))
                    .child(tab_example(&theme, "Filled tabs · content width", tab_bar(&TAB_LABELS, filled, filled_colors, TabSizing::Content, 38.0, theme.spacing_medium, theme.spacing_small)))
                    .child(tab_example(&theme, "Pill tabs · equal width", tab_bar(&TAB_LABELS, pill, pill_colors, TabSizing::Equal, 34.0, theme.spacing_medium, theme.spacing_small)))
                    .child(tab_example(&theme, "Indicator tabs · content width", tab_bar(&TAB_LABELS, indicator, indicator_colors, TabSizing::Content, 34.0, theme.spacing_medium, theme.spacing_small)))
            ) as BoxedWidget}
            {Box::new(Surface::new(&theme, SurfaceRole::Inset, padding(Style {
                size: creamui_core::layout::Size { width: Dimension::Length(488.0), height: Dimension::Length(184.0) },
                ..column(theme.spacing_large)
            }, theme.spacing_medium))
                .child(tab_bar(&TAB_LABELS, content, filled_colors, TabSizing::Equal, 36.0, theme.spacing_medium, theme.spacing_small))
                .child(Box::new(View::new(&theme, preview_style)
                    .child(Box::new(Heading::new(&theme, title)))
                    .child(Box::new(Text::secondary(&theme, detail)))
                    .child(Box::new(RawText::new(current_label, theme.accent, 12.).bold(true).align(TextAlign::Start)))))
            ) as BoxedWidget}
        </RawView>
    })
}

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
fn ScrollPanel(
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

/// The "Tree" panel: the "Data View" category's first entry — browsing a
/// hierarchy is a different job from picking one value (see `ListBox` on
/// the "Selection" page instead). A `TreeController` tracks which folders
/// are expanded and which row is selected.
#[component]
fn TreePanel(theme: Theme, tree_scroll: ScrollController, tree: TreeController) -> BoxedWidget {
    let nodes = vec![
        TreeNode::new(1, "src").with_children(vec![
            TreeNode::new(2, "main.rs"),
            TreeNode::new(3, "widgets").with_children(vec![
                TreeNode::new(4, "button.rs"),
                TreeNode::new(5, "scroll.rs"),
            ]),
        ]),
        TreeNode::new(6, "Cargo.toml"),
        TreeNode::new(7, "README.md"),
    ];
    let tree_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(260.0),
            height: Dimension::Length(280.0),
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    let view = TreeView::new(&theme, tree_style, tree_scroll, tree, &nodes);
    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Tree".to_owned()} subtitle={"Click a chevron to expand or collapse a folder, click a row to select it, or use the arrow keys once focused.".to_owned()} />
            {field_card(&theme, "File browser", Box::new(view))}
        </RawView>
    })
}

/// The "Table" panel: the Data View category's column-based entry — a CSV
/// viewer's shape. `Table` only needs `Vec<Vec<String>>`, so these rows are
/// parsed from a plain comma-separated string with `str::split` below, not
/// a CSV crate — however an application gets to that shape is up to it.
#[component]
fn TablePanel(
    theme: Theme,
    table_scroll: ScrollController,
    table_selected: Signal<usize>,
) -> BoxedWidget {
    const CSV: &str = "Ada Lovelace,Mathematician,1815
Grace Hopper,Programmer,1906
Alan Turing,Mathematician,1912
Margaret Hamilton,Engineer,1936
Katherine Johnson,Physicist,1918
Barbara Liskov,Computer Scientist,1939
Radia Perlman,Engineer,1951";
    let rows: Vec<Vec<String>> = CSV
        .lines()
        .map(|line| line.split(',').map(|cell| cell.to_owned()).collect())
        .collect();
    let columns = vec![
        TableColumn::new("Name", 160.0),
        TableColumn::new("Field", 160.0),
        TableColumn::new("Born", 70.0),
    ];
    let selected = table_selected.get();
    let set_selected = table_selected.clone();
    let table_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(390.0),
            height: Dimension::Length(280.0),
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    let table = Table::new(&theme, table_style, table_scroll, columns)
        .rows(rows)
        .on_row_click(Some(selected), move |index| set_selected.set(index));
    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Table".to_owned()} subtitle={"A CSV-shaped grid: fixed columns, a header that stays put, and a scrollable body. Click a row to select it.".to_owned()} />
            {field_card(&theme, "Notable computer scientists", Box::new(table))}
        </RawView>
    })
}

/// Starts the native showcase, or the browser canvas when built for WASM.
pub fn launch() {
    let image_png = ImageData::from_bytes(include_bytes!("../assets/images/iridescent.png"))
        .expect("bundled PNG should decode");
    let image_jpeg = ImageData::from_bytes(include_bytes!("../assets/images/still-life.jpg"))
        .expect("bundled JPEG should decode");
    let image_webp = ImageData::from_bytes(include_bytes!("../assets/images/botanical.webp"))
        .expect("bundled WebP should decode");
    let dark_mode = Signal::new(true);
    let accent_index = Signal::new(0usize);
    let active_section = Signal::new(0usize);

    let plain = TextController::default();
    let with_placeholder = TextController::default();
    let notes =
        TextController::new("Every control on this page reads its colors from the current Theme.");
    let notes_wrapped = TextController::new(
        "This one sets wrap={true}: long lines break onto a new row instead of scrolling past the edge.",
    );
    let picker_date_time = DateTimeController::new(DateTime::new(2026, 9, 8, 14, 30));
    let picker_color = Signal::new(Color::rgb(181, 139, 255));
    let picker_color_popup = ColorPickerController::default();
    let picker_file = Signal::new(String::new());

    let clicks = Signal::new(0i32);
    let typography_link_clicks = Signal::new(0i32);

    let volume = Signal::new(0.6f32);
    let brightness = Signal::new(0.8f32);
    let zoom = Signal::new(0.3f32);

    let notifications = Signal::new(true);
    let auto_save = Signal::new(false);
    let beta_features = Signal::new(false);

    let select = SelectController::default();
    let radio = Signal::new(0usize);
    let segment = Signal::new(1usize);
    let progress = Signal::new(0.62f32);
    let show_popover = Signal::new(false);
    let show_alert = Signal::new(false);

    let sidebar_demo_active = Signal::new(0usize);
    let tabs_filled = TabController::default();
    let tabs_pill = TabController::new(1);
    let tabs_indicator = TabController::new(2);
    let tabs_content = TabController::default();
    let content_scroll = ScrollController::default();
    let nav_scroll = ScrollController::default();
    let themed_scroll_demo = ScrollController::default();
    let custom_scroll_demo = ScrollController::default();
    let list_scroll_demo = ScrollController::default();
    let list_selected = Signal::new(0usize);
    let tree_scroll_demo = ScrollController::default();
    let tree_demo = TreeController::default();
    let table_scroll_demo = ScrollController::default();
    let table_selected = Signal::new(0usize);

    // Kept alive for the window's whole lifetime (`launch` doesn't return
    // until `run` does) so the effect it holds keeps reacting; dropping an
    // `Effect` unsubscribes it.
    let theme_sync: Rc<RefCell<Option<Effect>>> = Rc::new(RefCell::new(None));

    run(
        WindowOptions {
            title: "CreamUI — Showcase".into(),
            width: 1080,
            height: 740,
            theme: build_theme(dark_mode.peek(), ACCENTS[accent_index.peek()].1),
            ..Default::default()
        },
        Theme::dark().surface,
        {
            let dark_mode = dark_mode.clone();
            let accent_index = accent_index.clone();
            move |handle: WindowHandle| {
                let dark_mode = dark_mode.clone();
                let accent_index = accent_index.clone();
                *theme_sync.borrow_mut() = Some(create_effect(move || {
                    handle.set_theme(build_theme(dark_mode.get(), ACCENTS[accent_index.get()].1));
                }));
            }
        },
        move |viewport: Size| -> BoxedWidget {
            let theme = use_theme();

            let root_style = Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Length(viewport.width),
                    height: Dimension::Length(viewport.height),
                },
                align_items: Some(AlignItems::Stretch),
                ..row(0.0)
            };

            let content_outer_style = padding(
                Style {
                    flex_grow: 1.0,
                    size: creamui_core::layout::Size {
                        width: Dimension::Auto,
                        height: Dimension::Percent(1.0),
                    },
                    ..column(0.0)
                },
                24.,
            );
            let content_style = padding(
                Style {
                    flex_grow: 0.0,
                    size: creamui_core::layout::Size {
                        width: Dimension::Percent(1.0),
                        height: Dimension::Auto,
                    },
                    ..column(0.0)
                },
                28.,
            );

            // Only build the visible page. The reactive runtime removes
            // dependencies from the prior execution before collecting the
            // active branch's signals, so a hidden editor or progress demo
            // cannot invalidate this window or make us lay it out again.
            let panel = match active_section.get() {
                0 => AppearancePanel(AppearancePanelProps {
                    theme,
                    dark_mode: dark_mode.clone(),
                    accent_index: accent_index.clone(),
                }),
                1 => TypographyPanel(TypographyPanelProps {
                    theme,
                    link_clicks: typography_link_clicks.clone(),
                }),
                2 => InputPanel(InputPanelProps {
                    theme,
                    plain: plain.clone(),
                    with_placeholder: with_placeholder.clone(),
                    notes: notes.clone(),
                    notes_wrapped: notes_wrapped.clone(),
                }),
                3 => PickersPanel(PickersPanelProps {
                    theme,
                    date_time: picker_date_time.clone(),
                    color: picker_color.clone(),
                    color_picker: picker_color_popup.clone(),
                    file: picker_file.clone(),
                }),
                4 => ImagesPanel(ImagesPanelProps {
                    theme,
                    png: image_png.clone(),
                    jpeg: image_jpeg.clone(),
                    webp: image_webp.clone(),
                }),
                5 => ButtonPanel(ButtonPanelProps {
                    theme,
                    clicks: clicks.clone(),
                }),
                6 => SliderPanel(SliderPanelProps {
                    theme,
                    volume: volume.clone(),
                    brightness: brightness.clone(),
                    zoom: zoom.clone(),
                }),
                7 => CheckboxPanel(CheckboxPanelProps {
                    theme,
                    notifications: notifications.clone(),
                    auto_save: auto_save.clone(),
                    beta_features: beta_features.clone(),
                }),
                8 => SelectionPanel(SelectionPanelProps {
                    theme,
                    select: select.clone(),
                    radio: radio.clone(),
                    segment: segment.clone(),
                    list_scroll: list_scroll_demo.clone(),
                    list_selected: list_selected.clone(),
                }),
                9 => FeedbackPanel(FeedbackPanelProps {
                    theme,
                    progress: progress.clone(),
                    show_popover: show_popover.clone(),
                    show_alert: show_alert.clone(),
                }),
                10 => SidebarPanel(SidebarPanelProps {
                    theme,
                    active: sidebar_demo_active.clone(),
                }),
                11 => TabsPanel(TabsPanelProps {
                    theme,
                    filled: tabs_filled.clone(),
                    pill: tabs_pill.clone(),
                    indicator: tabs_indicator.clone(),
                    content: tabs_content.clone(),
                }),
                12 => ScrollPanel(ScrollPanelProps {
                    theme,
                    themed_scroll: themed_scroll_demo.clone(),
                    custom_scroll: custom_scroll_demo.clone(),
                }),
                13 => TreePanel(TreePanelProps {
                    theme,
                    tree_scroll: tree_scroll_demo.clone(),
                    tree: tree_demo.clone(),
                }),
                _ => TablePanel(TablePanelProps {
                    theme,
                    table_scroll: table_scroll_demo.clone(),
                    table_selected: table_selected.clone(),
                }),
            };

            let scroll_style = Style {
                flex_grow: 1.0,
                size: creamui_core::layout::Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Percent(1.0),
                },
                ..Default::default()
            };
            let content = RawScrollView::controlled(scroll_style, content_scroll.clone()).child(
                Box::new(Surface::new(&theme, SurfaceRole::Panel, content_style).child(panel)),
            );
            let dialog: BoxedWidget = if show_alert.get() {
                let dismiss = show_alert.clone();
                Box::new(
                    AlertDialog::new(
                        &theme,
                        "Delete this draft?",
                        "This example uses an Overlay backdrop. Clicking outside or Cancel closes it.",
                        move || dismiss.set(false),
                    )
                    .dismiss_button("Cancel")
                    .confirm("Delete", { let dismiss = show_alert.clone(); move || dismiss.set(false) }),
                )
            } else {
                Box::new(RawView::new(Style::default()))
            };

            Box::new(jsx! {
                <RawView style={root_style} background={theme.surface}>
                    <Nav theme={theme} active={active_section.clone()} content_scroll={content_scroll.clone()} nav_scroll={nav_scroll.clone()} />
                    <RawView style={content_outer_style} background={theme.surface}>
                        {Box::new(content) as BoxedWidget}
                    </RawView>
                    {dialog}
                </RawView>
            })
        },
    );
}

#[allow(dead_code)] // This source file is also compiled as the showcase library module.
fn main() {
    launch();
}
