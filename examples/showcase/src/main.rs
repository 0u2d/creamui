//! A component showcase: a sidebar switches between a live theme editor
//! ("Appearance") and a gallery view for every themed control CreamUI ships
//! with — `Input`, `Button`, `Slider`, `Checkbox`, `Sidebar`, and `Tabs`.
//!
//! The whole window is driven by a handful of small signals — `dark_mode`,
//! `accent_index`, `active_section`, plus one signal per interactive control
//! — so picking a new accent color or toggling dark/light mode re-renders
//! every panel with the new `Theme` immediately, the same reactive path any
//! other `Signal` change takes.

use creamui_core::layout::{AlignItems, Dimension, JustifyContent, Style};
use creamui_core::{BoxedWidget, Size, TextAlign};
use creamui_macros::{component, jsx};
use creamui_reactive::Signal;
use creamui_render::{run, WindowOptions};
use creamui_theme::{Color, SelectionStyle, Theme};
use creamui_widgets::layout::{column, fixed, padding, row};
use creamui_widgets::{
    tab_styles, Button, ButtonSize, ButtonState, Heading, RawText, RawView, Sidebar, SidebarItem,
    Switch, Tab, TabColors, TabController, TabSizing, Tabs, Text, TextController, TextInput,
    TextSize, View,
};
use creamui_widgets::{Choice, Icon, NavigationItem, Surface, SurfaceRole, Symbol};

/// Sidebar categories in display order.
const NAV_LABELS: [&str; 7] = [
    "Appearance",
    "Input",
    "Button",
    "Slider",
    "Checkbox",
    "Sidebar",
    "Tabs",
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

/// The showcase category rail.
#[component]
fn Nav(theme: Theme, active: Signal<usize>) -> BoxedWidget {
    let mut nav = RawView::new(padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(212.),
                height: Dimension::Percent(1.),
            },
            flex_shrink: 0.,
            ..column(5.)
        },
        16.,
    ));
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
        Symbol::Keyboard,
        Symbol::Controls,
        Symbol::Sliders,
        Symbol::Check,
        Symbol::Folder,
        Symbol::Grid,
    ];
    for (i, label) in NAV_LABELS.iter().enumerate() {
        if i == 0 || i == 1 || i == 5 {
            nav = nav.child(Box::new(
                RawView::new(padding(column(0.), 8.)).child(Box::new(
                    RawText::new(
                        if i == 0 {
                            "SHOWCASE"
                        } else if i == 1 {
                            "CONTROLS"
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
        nav = nav.child(Box::new(NavigationItem::new(
            &theme,
            symbols[i],
            *label,
            active.get() == i,
            move || select.set(i),
        )));
    }
    nav = nav
        .child(Box::new(RawView::new(Style {
            flex_grow: 1.,
            ..Default::default()
        })))
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
        <RawView style={column(24.)}>
            <SectionHeader theme={theme} title={"Appearance".to_owned()} subtitle={"Explore the same components in a different light.".to_owned()} />
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"Mode".to_owned()}</Text>
                <RawView style={row(theme.spacing_medium)} children={mode_pills} />
            </RawView>
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"Accent color".to_owned()}</Text>
                <RawView style={row(theme.spacing_medium)} children={swatches} />
            </RawView>
            <Text theme={&theme} align={TextAlign::Start} color={theme.text_disabled} style={label_style()}>{summary}</Text>
            <View theme={&theme} style={padding(column(18.),20.)}>
                <Heading theme={&theme} size={TextSize::Md}>"Component preview"</Heading>
                <Text theme={&theme} align={TextAlign::Start}>"Open a category to explore sizes, states, and interactions."</Text>
                <RawView style={row(10.)}>
                    {Box::new(Button::new(&theme,"Primary",{let mode=dark_mode.clone();move || mode.update(|v| *v=!*v)})) as BoxedWidget}
                    {Box::new(Button::secondary(&theme,ButtonSize::Md,"Secondary",{let mode=dark_mode.clone();move || mode.update(|v| *v=!*v)})) as BoxedWidget}
                    {Box::new(Button::new(&theme,"Disabled",||{}).disabled(true)) as BoxedWidget}
                </RawView>
                <Text theme={&theme} align={TextAlign::Start} secondary={true}>"These preview buttons switch the color scheme."</Text>
            </View>
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
    Box::new(jsx! {
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Input".to_owned()} subtitle={"Write, select, and edit. Each field keeps its own content.".to_owned()} />
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"Default".to_owned()}</Text>
                <TextInput theme={&theme} controller={&plain} />
            </RawView>
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"With placeholder".to_owned()}</Text>
                <TextInput theme={&theme} controller={&with_placeholder} placeholder={"Type something…".to_owned()} />
            </RawView>
            <RawView style={row(theme.spacing_large)}>
                <RawView style={column(theme.spacing_small)}>
                    {Box::new(TextInput::new(&theme, "", |_| {}).border(theme.danger)) as BoxedWidget}
                    <Text theme={&theme} align={TextAlign::Start} color={theme.danger} style={label_style()}>{"Error: this field is required".to_owned()}</Text>
                </RawView>
                <RawView style={column(theme.spacing_small)}>
                    {Box::new(TextInput::new(&theme, "", |_| {}).border(theme.warning)) as BoxedWidget}
                    <Text theme={&theme} align={TextAlign::Start} color={theme.warning} style={label_style()}>{"Warning: verify this value".to_owned()}</Text>
                </RawView>
            </RawView>
            <RawView style={row(theme.spacing_large)}>
                <RawView style={column(theme.spacing_small)}>
                    <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"TextArea · horizontal scrolling".to_owned()}</Text>
                    <TextArea theme={&theme} controller={&notes} style={textarea_style()} placeholder={"Notes…".to_owned()} />
                </RawView>
                <RawView style={column(theme.spacing_small)}>
                    <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"TextArea · wrap to fit".to_owned()}</Text>
                    <TextArea theme={&theme} controller={&notes_wrapped} style={textarea_style()} wrap={true} placeholder={"Notes…".to_owned()} />
                </RawView>
            </RawView>
        </RawView>
    })
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
        RawView::new(column(24.))
            .child(SectionHeader(SectionHeaderProps {
                theme,
                title: "Buttons".into(),
                subtitle: "A clear hierarchy, from everyday actions to important decisions.".into(),
            }))
            .child(Box::new(actions))
            .child(Box::new(
                RawText::new("One family, four sizes", theme.text_secondary, 13.)
                    .align(TextAlign::Start),
            ))
            .child(Box::new(sizes))
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
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Slider".to_owned()} subtitle={"Fine adjustments with immediate feedback.".to_owned()} />
            {slider_row(&theme, "Volume", volume, |v| format!("{:.0}%", v * 100.0))}
            {slider_row(&theme, "Brightness", brightness, |v| format!("{:.0}%", v * 100.0))}
            {slider_row(&theme, "Zoom", zoom, |v| format!("{:.2}x", 0.5 + v * 1.5))}
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
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Checkbox".to_owned()} subtitle={"Small preferences, clearly expressed.".to_owned()} />
            {checkbox_row(&theme, "Notifications", notifications)}
            {checkbox_row(&theme, "Auto-save", auto_save.clone())}
            {checkbox_row(&theme, "Beta features", beta_features)}
            <RawView style={row(theme.spacing_medium)}>
                {Box::new(Switch::new(&theme, auto_save.get(), { let set = auto_save.clone(); move || set.update(|value| *value = !*value) })) as BoxedWidget}
                <Text theme={&theme} align={TextAlign::Start} style={label_style()}>{"Switch presentation".to_owned()}</Text>
            </RawView>
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
        <RawView style={column(theme.spacing_large)}>
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
    controller: TabController,
    colors: TabColors,
    sizing: TabSizing,
    height: f32,
    padding: f32,
) -> BoxedWidget {
    let bar_style = Style {
        size: creamui_core::layout::Size {
            width: if sizing == TabSizing::Fill {
                Dimension::Percent(1.0)
            } else {
                Dimension::Auto
            },
            height: Dimension::Auto,
        },
        align_self: if sizing == TabSizing::Fill {
            None
        } else {
            Some(creamui_core::layout::AlignSelf::Start)
        },
        ..row(colors.gap)
    };
    let styles = tab_styles(&TAB_LABELS, sizing, height, padding);
    let mut bar = Tabs::new(colors, bar_style);
    for (index, label) in TAB_LABELS.iter().enumerate() {
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
    let filled_colors = TabColors::dark(&theme);

    let mut pill_colors = filled_colors;
    pill_colors.inactive_background = Some(theme.surface_hover);
    pill_colors.active_background = theme.surface_elevated;
    pill_colors.active_text = theme.text_primary;
    pill_colors.radius = 16.0;
    pill_colors.container_radius = 20.0;

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
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Tabs".to_owned()} subtitle={"Three visual styles, followed by a tab bar connected to its content.".to_owned()} />
            {tab_example(&theme, "Filled tabs · content width", tab_bar(filled, filled_colors, TabSizing::Content, 38.0, theme.spacing_medium))}
            {tab_example(&theme, "Pill tabs · equal width", tab_bar(pill, pill_colors, TabSizing::Equal, 34.0, theme.spacing_medium))}
            {tab_example(&theme, "Indicator tabs · content width", tab_bar(indicator, indicator_colors, TabSizing::Content, 34.0, theme.spacing_medium))}
            {Box::new(Surface::new(&theme, SurfaceRole::Inset, padding(Style {
                size: creamui_core::layout::Size { width: Dimension::Length(488.0), height: Dimension::Length(184.0) },
                ..column(theme.spacing_large)
            }, theme.spacing_medium))
                .child(tab_bar(content, filled_colors, TabSizing::Equal, 36.0, theme.spacing_medium))
                .child(Box::new(View::new(&theme, preview_style)
                    .child(Box::new(Heading::new(&theme, title)))
                    .child(Box::new(Text::secondary(&theme, detail)))
                    .child(Box::new(RawText::new(current_label, theme.accent, 12.).bold(true).align(TextAlign::Start)))))
            ) as BoxedWidget}
        </RawView>
    })
}

fn main() {
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

    let clicks = Signal::new(0i32);

    let volume = Signal::new(0.6f32);
    let brightness = Signal::new(0.8f32);
    let zoom = Signal::new(0.3f32);

    let notifications = Signal::new(true);
    let auto_save = Signal::new(false);
    let beta_features = Signal::new(false);

    let sidebar_demo_active = Signal::new(0usize);
    let tabs_filled = TabController::default();
    let tabs_pill = TabController::new(1);
    let tabs_indicator = TabController::new(2);
    let tabs_content = TabController::default();

    run(
        WindowOptions {
            title: "CreamUI — Showcase".into(),
            width: 1080,
            height: 740,
            ..Default::default()
        },
        Theme::dark().surface,
        |_| {},
        move |viewport: Size| -> BoxedWidget {
            let theme = build_theme(dark_mode.get(), ACCENTS[accent_index.get()].1);

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

            // Every panel is built on every render, regardless of which one
            // is currently visible. `Signal::get()` only subscribes the
            // *currently running* reactive effect, and `build_ui` only ever
            // runs inside that effect on this very first pass (later
            // reactive re-renders come from a redraw driven by whichever
            // signals were subscribed here) — so a panel whose props are
            // never touched on this first pass would never get a chance to
            // subscribe its signals, and its controls would silently stop
            // reacting to clicks/drags/typing. Building all seven up front
            // keeps every signal subscribed from the start, however deep in
            // the sidebar its section sits.
            let panels: [BoxedWidget; 7] = [
                AppearancePanel(AppearancePanelProps {
                    theme,
                    dark_mode: dark_mode.clone(),
                    accent_index: accent_index.clone(),
                }),
                InputPanel(InputPanelProps {
                    theme,
                    plain: plain.clone(),
                    with_placeholder: with_placeholder.clone(),
                    notes: notes.clone(),
                    notes_wrapped: notes_wrapped.clone(),
                }),
                ButtonPanel(ButtonPanelProps {
                    theme,
                    clicks: clicks.clone(),
                }),
                SliderPanel(SliderPanelProps {
                    theme,
                    volume: volume.clone(),
                    brightness: brightness.clone(),
                    zoom: zoom.clone(),
                }),
                CheckboxPanel(CheckboxPanelProps {
                    theme,
                    notifications: notifications.clone(),
                    auto_save: auto_save.clone(),
                    beta_features: beta_features.clone(),
                }),
                SidebarPanel(SidebarPanelProps {
                    theme,
                    active: sidebar_demo_active.clone(),
                }),
                TabsPanel(TabsPanelProps {
                    theme,
                    filled: tabs_filled.clone(),
                    pill: tabs_pill.clone(),
                    indicator: tabs_indicator.clone(),
                    content: tabs_content.clone(),
                }),
            ];
            let panel = panels.into_iter().nth(active_section.get()).expect(
                "active_section is always kept within NAV_LABELS' range by Nav's click handlers",
            );

            Box::new(jsx! {
                <RawView style={root_style} background={theme.surface}>
                    <Nav theme={theme} active={active_section.clone()} />
                    <RawView style={content_outer_style} background={theme.surface}>
                        {Box::new(Surface::new(&theme, SurfaceRole::Panel, content_style).child(panel)) as BoxedWidget}
                    </RawView>
                </RawView>
            })
        },
    );
}
