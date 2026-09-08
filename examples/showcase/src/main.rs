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
use creamui_theme::{Color, Theme};
use creamui_widgets::layout::{column, fixed, margin_xy, padding, row};
use creamui_widgets::{
    RawView, Sidebar, SidebarItem, Tab, TabColors, Tabs, TextController, TextSize,
};

/// Sidebar entries, in display order. `SEPARATOR_AFTER` marks which of these
/// get a divider drawn below them.
const NAV_LABELS: [&str; 7] = [
    "Appearance",
    "Input",
    "Button",
    "Slider",
    "Checkbox",
    "Sidebar",
    "Tabs",
];
const SEPARATOR_AFTER: [usize; 2] = [0, 4];

const ACCENTS: [(&str, Color); 5] = [
    ("Violet", Color::rgb(0x7c, 0x5c, 0xff)),
    ("Blue", Color::rgb(0x2f, 0x8a, 0xe6)),
    ("Green", Color::rgb(0x2e, 0xc9, 0x6f)),
    ("Pink", Color::rgb(0xff, 0x5a, 0xd8)),
    ("Orange", Color::rgb(0xe6, 0x8a, 0x2e)),
];

/// Shifts each color channel by `delta`, clamping at the `u8` bounds. Used to
/// derive hover/pressed accent shades from whichever swatch is selected.
fn shade(color: Color, delta: i32) -> Color {
    let shift = |c: u8| (c as i32 + delta).clamp(0, 255) as u8;
    Color::rgb(shift(color.r), shift(color.g), shift(color.b))
}

fn build_theme(dark: bool, accent: Color) -> Theme {
    let mut theme = if dark { Theme::dark() } else { Theme::light() };
    theme.accent = accent;
    theme.accent_hover = shade(accent, if dark { 18 } else { -18 });
    theme.accent_pressed = shade(accent, if dark { -18 } else { 18 });
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

/// A 1px divider spanning the sidebar's width, with a little breathing room
/// above and below.
fn separator(theme: &Theme) -> BoxedWidget {
    let style = margin_xy(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(1.0),
            },
            flex_shrink: 0.0,
            ..Default::default()
        },
        0.0,
        6.0,
    );
    Box::new(RawView::new(style).background(theme.border))
}

/// The left-hand navigation rail: `NAV_LABELS`, with dividers spliced in
/// after the indices in `SEPARATOR_AFTER`.
#[component]
fn Nav(theme: Theme, active: Signal<usize>) -> BoxedWidget {
    let colors = TabColors::dark(&theme);
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
    let sidebar_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(180.0),
                height: Dimension::Percent(1.0),
            },
            flex_shrink: 0.0,
            ..column(2.0)
        },
        theme.spacing_small,
    );
    let mut sidebar = Sidebar::new(colors, sidebar_style);
    for (index, label) in NAV_LABELS.iter().enumerate() {
        let is_active = active.get() == index;
        let select = active.clone();
        sidebar = sidebar.child(Box::new(SidebarItem::new(
            colors,
            item_style.clone(),
            *label,
            is_active,
            move || select.set(index),
        )));
        if SEPARATOR_AFTER.contains(&index) {
            sidebar = sidebar.child(separator(&theme));
        }
    }
    Box::new(sidebar)
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
    let style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Auto,
                height: Dimension::Length(34.0),
            },
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            ..Default::default()
        },
        theme.spacing_medium * 1.5,
    );
    let text_color = if active {
        theme.selection_text
    } else {
        theme.text_primary
    };
    let background = if active { theme.accent } else { theme.surface_elevated };
    Box::new(jsx! {
        <RawButton style={style.clone()} background={background} corner_radius={theme.radius_medium} on_click={on_click}>
            <RawText color={text_color} font_size={14.0} align={TextAlign::Center} style={style}>{label.to_owned()}</RawText>
        </RawButton>
    })
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
    let ring = if active { theme.text_primary } else { theme.surface };
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
        swatches.push(swatch(&theme, *color, index == selected_accent, move || {
            set_accent.set(index)
        }));
    }

    let accent_name = ACCENTS[selected_accent].0;
    let summary = format!(
        "{} mode · {} accent",
        if is_dark { "Dark" } else { "Light" },
        accent_name
    );

    Box::new(jsx! {
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Appearance".to_owned()} subtitle={"Edit the current theme: switch modes or pick an accent color.".to_owned()} />
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"Mode".to_owned()}</Text>
                <RawView style={row(theme.spacing_medium)} children={mode_pills} />
            </RawView>
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"Accent color".to_owned()}</Text>
                <RawView style={row(theme.spacing_medium)} children={swatches} />
            </RawView>
            <Text theme={&theme} align={TextAlign::Start} color={theme.text_disabled} style={label_style()}>{summary}</Text>
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
) -> BoxedWidget {
    Box::new(jsx! {
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Input".to_owned()} subtitle={"TextInput and TextArea, each bound to its own TextController.".to_owned()} />
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"Default".to_owned()}</Text>
                <TextInput theme={&theme} controller={&plain} />
            </RawView>
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"With placeholder".to_owned()}</Text>
                <TextInput theme={&theme} controller={&with_placeholder} placeholder={"Type something…".to_owned()} />
            </RawView>
            <RawView style={column(theme.spacing_small)}>
                <Text theme={&theme} align={TextAlign::Start} color={theme.text_secondary} style={label_style()}>{"Multi-line (TextArea)".to_owned()}</Text>
                <TextArea theme={&theme} controller={&notes} placeholder={"Notes…".to_owned()} />
            </RawView>
        </RawView>
    })
}

/// The "Button" panel: the default themed `Button`, a couple of
/// semantically-colored variants built straight from `RawButton`, and a
/// disabled-looking one, plus a click counter to prove the handlers fire.
#[component]
fn ButtonPanel(theme: Theme, clicks: Signal<i32>) -> BoxedWidget {
    let count = clicks.get();
    let default_clicks = clicks.clone();
    let danger_clicks = clicks.clone();
    let success_clicks = clicks.clone();

    fn button_style() -> Style {
        padding(
            Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Auto,
                    height: Dimension::Length(38.0),
                },
                justify_content: Some(JustifyContent::Center),
                align_items: Some(AlignItems::Center),
                ..Default::default()
            },
            16.0,
        )
    }

    Box::new(jsx! {
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Button".to_owned()} subtitle={"The default themed Button next to a few RawButton-built variants.".to_owned()} />
            <RawView style={row(theme.spacing_medium)}>
                <Button theme={&theme} on_click={move || default_clicks.update(|c| *c += 1)}>{"Default".to_owned()}</Button>
                <RawButton style={button_style()} background={theme.danger} corner_radius={theme.radius_medium} on_click={move || danger_clicks.update(|c| *c += 1)}>
                    <RawText color={theme.selection_text} font_size={16.0} align={TextAlign::Center} style={button_style()}>{"Delete".to_owned()}</RawText>
                </RawButton>
                <RawButton style={button_style()} background={theme.success} corner_radius={theme.radius_medium} on_click={move || success_clicks.update(|c| *c += 1)}>
                    <RawText color={theme.selection_text} font_size={16.0} align={TextAlign::Center} style={button_style()}>{"Save".to_owned()}</RawText>
                </RawButton>
                <RawButton style={button_style()} background={theme.surface_hover} corner_radius={theme.radius_medium} on_click={|| {}} disabled={true}>
                    <RawText color={theme.text_disabled} font_size={16.0} align={TextAlign::Center} style={button_style()}>{"Disabled".to_owned()}</RawText>
                </RawButton>
            </RawView>
            <Text theme={&theme} align={TextAlign::Start} color={theme.text_disabled} style={label_style()}>{format!("Clicked {} time(s)", count)}</Text>
        </RawView>
    })
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
            <SectionHeader theme={theme} title={"Slider".to_owned()} subtitle={"Three sliders, each bound to its own Signal<f32>.".to_owned()} />
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
            <SectionHeader theme={theme} title={"Checkbox".to_owned()} subtitle={"Three checkboxes, each toggling its own Signal<bool>.".to_owned()} />
            {checkbox_row(&theme, "Notifications", notifications)}
            {checkbox_row(&theme, "Auto-save", auto_save)}
            {checkbox_row(&theme, "Beta features", beta_features)}
        </RawView>
    })
}

/// The "Sidebar" panel: a small, self-contained `Sidebar`/`SidebarItem` demo
/// with its own selection state, next to the panel it controls.
#[component]
fn SidebarPanel(theme: Theme, active: Signal<usize>) -> BoxedWidget {
    const ITEMS: [&str; 3] = ["Inbox", "Drafts", "Sent"];
    let colors = TabColors::dark(&theme);
    let item_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(32.0),
            },
            align_items: Some(AlignItems::Center),
            ..Default::default()
        },
        theme.spacing_small,
    );
    let rail_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(120.0),
            height: Dimension::Length(120.0),
        },
        flex_shrink: 0.0,
        ..column(2.0)
    };
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
                width: Dimension::Length(220.0),
                height: Dimension::Length(120.0),
            },
            ..column(theme.spacing_small)
        },
        theme.spacing_medium,
    );
    let current_label = ITEMS[active.get()].to_owned();
    Box::new(jsx! {
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Sidebar".to_owned()} subtitle={"The Sidebar / SidebarItem pair, embedded here as its own live demo.".to_owned()} />
            <RawView style={{ let mut s = row(theme.spacing_medium); s.align_items = Some(AlignItems::Stretch); s }}>
                {Box::new(rail) as BoxedWidget}
                <View theme={&theme} style={preview_style}>
                    <Text theme={&theme} align={TextAlign::Start} style={label_style()}>{current_label}</Text>
                </View>
            </RawView>
        </RawView>
    })
}

/// The "Tabs" panel: a small, self-contained `Tabs`/`Tab` demo, the
/// horizontal counterpart of [`SidebarPanel`].
#[component]
fn TabsPanel(theme: Theme, active: Signal<usize>) -> BoxedWidget {
    const ITEMS: [&str; 3] = ["Overview", "Activity", "Settings"];
    let colors = TabColors::dark(&theme);
    let tab_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Auto,
                height: Dimension::Length(34.0),
            },
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            flex_grow: 1.0,
            ..Default::default()
        },
        theme.spacing_medium,
    );
    let bar_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(340.0),
            height: Dimension::Auto,
        },
        ..row(theme.spacing_small)
    };
    let mut bar = Tabs::new(colors, bar_style);
    for (index, label) in ITEMS.iter().enumerate() {
        let is_active = active.get() == index;
        let select = active.clone();
        bar = bar.child(Box::new(Tab::new(
            colors,
            tab_style.clone(),
            *label,
            is_active,
            move || select.set(index),
        )));
    }
    let preview_style = padding(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(340.0),
                height: Dimension::Length(80.0),
            },
            ..column(theme.spacing_small)
        },
        theme.spacing_medium,
    );
    let current_label = ITEMS[active.get()].to_owned();
    Box::new(jsx! {
        <RawView style={column(theme.spacing_large)}>
            <SectionHeader theme={theme} title={"Tabs".to_owned()} subtitle={"The Tabs / Tab pair, embedded here as its own live demo.".to_owned()} />
            {Box::new(bar) as BoxedWidget}
            <View theme={&theme} style={preview_style}>
                <Text theme={&theme} align={TextAlign::Start} style={label_style()}>{current_label}</Text>
            </View>
        </RawView>
    })
}

fn main() {
    let dark_mode = Signal::new(true);
    let accent_index = Signal::new(0usize);
    let active_section = Signal::new(0usize);

    let plain = TextController::default();
    let with_placeholder = TextController::default();
    let notes = TextController::new(
        "Every control on this page reads its colors from the current Theme.",
    );

    let clicks = Signal::new(0i32);

    let volume = Signal::new(0.6f32);
    let brightness = Signal::new(0.8f32);
    let zoom = Signal::new(0.3f32);

    let notifications = Signal::new(true);
    let auto_save = Signal::new(false);
    let beta_features = Signal::new(false);

    let sidebar_demo_active = Signal::new(0usize);
    let tabs_demo_active = Signal::new(0usize);

    run(
        WindowOptions {
            title: "CreamUI — Showcase".into(),
            width: 960,
            height: 620,
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

            let content_style = padding(
                Style {
                    flex_grow: 1.0,
                    size: creamui_core::layout::Size {
                        width: Dimension::Auto,
                        height: Dimension::Percent(1.0),
                    },
                    ..column(0.0)
                },
                theme.spacing_large * 1.5,
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
                    active: tabs_demo_active.clone(),
                }),
            ];
            let panel = panels
                .into_iter()
                .nth(active_section.get())
                .expect("active_section is always kept within NAV_LABELS' range by Nav's click handlers");

            Box::new(jsx! {
                <RawView style={root_style} background={theme.surface}>
                    <Nav theme={theme} active={active_section.clone()} />
                    <RawView style={content_style} background={theme.surface}>
                        {panel}
                    </RawView>
                </RawView>
            })
        },
    );
}
