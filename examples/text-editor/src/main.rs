//! A calm desktop editor built entirely with static CreamUI JSX.
//!
//! `TextArea` is also available through `abi_jsx!` / `creamui_dynamic`, so
//! this screen is a useful native reference for applications shipped over
//! the dynamic ABI.

use creamui_core::layout::{AlignItems, Dimension, FlexDirection, JustifyContent, LengthPercentageAuto, Position, Style};
use creamui_core::{BoxedWidget, Size, TextAlign};
use creamui_macros::{component, jsx};
use creamui_reactive::Signal;
use creamui_render::{run, WindowOptions};
use creamui_theme::Color;
use creamui_widgets::layout::{fixed, row};

const WINDOW: Color = Color::rgb(0x1a, 0x1b, 0x1e);
const CONTENT: Color = Color::rgb(0x24, 0x25, 0x2a);
const ACTIVE_LINE: Color = Color::rgb(0x2c, 0x2d, 0x35);

fn menu_colors() -> creamui_widgets::MenuColors {
    creamui_widgets::MenuColors::dark(&creamui_theme::Theme::dark())
}
const MUTED: Color = Color::rgb(0xa4, 0xa5, 0xad);
const BLUE: Color = Color::rgb(0x0a, 0x84, 0xff);

fn size(width: f32, height: f32) -> Style {
    Style {
        size: fixed(width, height),
        ..Default::default()
    }
}

#[component]
fn ToolbarMenu(label: String, id: i32, active: Signal<i32>) -> BoxedWidget {
    let is_active = active.get() == id;
    let click_active = active.clone();
    let text_color = if is_active { BLUE } else { MUTED };
    // Top-level menus are deliberately text-only. A menu bar is navigation,
    // not a row of contained buttons; the popup supplies the active affordance.
    Box::new(
        creamui_widgets::RawButton::new(size(42.0, 24.0), move || {
            click_active.set(if click_active.get() == id { 0 } else { id });
        })
        .child(Box::new(
            creamui_widgets::RawText::new(label, text_color, 13.0)
                .align(TextAlign::Start)
                .layout_style(size(42.0, 24.0)),
        )),
    )
}

#[component]
fn EditorToolbar(menus: Vec<String>, title: String, active: Signal<i32>, children: Vec<BoxedWidget>) -> BoxedWidget {
    let toolbar = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Length(28.0),
        },
        padding: creamui_core::layout::Rect {
            left: creamui_core::layout::LengthPercentage::Length(8.0),
            right: creamui_core::layout::LengthPercentage::Length(8.0),
            top: creamui_core::layout::LengthPercentage::Length(0.0),
            bottom: creamui_core::layout::LengthPercentage::Length(0.0),
        },
        ..row(8.0)
    };
    let mut view = creamui_widgets::MenuBar::new(menu_colors(), toolbar);
    for (index, label) in menus.into_iter().enumerate() {
        view = view.child(ToolbarMenu(ToolbarMenuProps { label, id: index as i32 + 1, active: active.clone() }));
    }
    let title_style = Style {
        size: creamui_core::layout::Size { width: Dimension::Auto, height: Dimension::Length(24.0) },
        flex_grow: 1.0,
        ..Default::default()
    };
    view = view.child(Box::new(
        creamui_widgets::RawText::new(title, menu_colors().muted_text, 12.0)
            .layout_style(title_style),
    ));
    for child in children { view = view.child(child); }
    Box::new(view)
}

fn command(label: &str, action: impl Fn() + 'static) -> BoxedWidget {
    Box::new(creamui_widgets::MenuItem::new(menu_colors(), size(134.0, 25.0), label, false, action))
}

#[component]
fn MenuPanel(active: Signal<i32>, document: Signal<String>, saved: Signal<bool>, status: Signal<String>) -> BoxedWidget {
    let open = active.get();
    // Do not paint a zero-height popup: some raster backends turn a
    // zero-height rounded rect into a one-pixel hairline below the menu bar.
    if open == 0 {
        return Box::new(creamui_widgets::RawView::new(Style::default()));
    }
    let item_count = if open == 1 { 3 } else { 0 };
    let left = 8.0 + (open.saturating_sub(1) as f32 * 52.0);
    let panel_style = Style {
        position: Position::Absolute,
        inset: creamui_core::layout::Rect { left: LengthPercentageAuto::Length(left), right: LengthPercentageAuto::Auto, top: LengthPercentageAuto::Length(28.0), bottom: LengthPercentageAuto::Auto },
        size: creamui_core::layout::Size { width: Dimension::Length(136.0), height: Dimension::Length(if item_count == 0 { 0.0 } else { item_count as f32 * 25.0 + 2.0 }) },
        flex_direction: FlexDirection::Column,
        padding: creamui_core::layout::Rect { left: creamui_core::layout::LengthPercentage::Length(1.0), right: creamui_core::layout::LengthPercentage::Length(1.0), top: creamui_core::layout::LengthPercentage::Length(1.0), bottom: creamui_core::layout::LengthPercentage::Length(1.0) },
        ..Default::default()
    };
    let close = active.clone();
    let mut panel = creamui_widgets::MenuPopup::new(menu_colors(), panel_style);
    match open {
        1 => {
            let doc = document.clone(); let state = status.clone(); let close_new = close.clone();
            panel = panel.child(command("New", move || { doc.set(String::new()); state.set("New document".into()); close_new.set(0); }));
            let doc = document.clone(); let state = status.clone(); let close_open = close.clone();
            panel = panel.child(command("Open File…", move || {
                if let Some(path) = rfd::FileDialog::new().add_filter("Text", &["txt", "md", "rs"]).pick_file() {
                    match std::fs::read_to_string(&path) { Ok(contents) => { doc.set(contents); state.set(format!("Opened {}", path.display())); }, Err(error) => state.set(format!("Could not open file: {error}")) }
                }
                close_open.set(0);
            }));
            let doc = document.clone(); let saved = saved.clone(); let state = status.clone(); let close_save = close.clone();
            panel = panel.child(command("Save File…", move || {
                if let Some(path) = rfd::FileDialog::new().add_filter("Text", &["txt", "md"]).set_file_name("Untitled.md").save_file() {
                    match std::fs::write(&path, doc.get()) { Ok(()) => { saved.set(true); state.set(format!("Saved {}", path.display())); }, Err(error) => state.set(format!("Could not save file: {error}")) }
                }
                close_save.set(0);
            }));
        }
        _ => {}
    }
    Box::new(panel)
}

#[component]
fn LineNumbers(value: String) -> BoxedWidget {
    let lines = value.matches('\n').count() + 1;
    let labels = (1..=lines)
        .map(|number| {
            Box::new(
                creamui_widgets::RawText::new(number.to_string(), MUTED, 14.0)
                    .align(TextAlign::End)
                    .layout_style(size(42.0, 20.0)),
            ) as BoxedWidget
        })
        .collect();
    Box::new(
        jsx! { <RawView style={Style { size: creamui_core::layout::Size { width: Dimension::Length(58.0), height: Dimension::Percent(1.0) }, flex_shrink: 0.0, flex_direction: FlexDirection::Column, padding: creamui_core::layout::Rect { left: creamui_core::layout::LengthPercentage::Length(0.0), right: creamui_core::layout::LengthPercentage::Length(10.0), top: creamui_core::layout::LengthPercentage::Length(14.0), bottom: creamui_core::layout::LengthPercentage::Length(0.0) }, ..Default::default() }} background={WINDOW} children={labels} /> },
    )
}

fn main() {
    let document = Signal::new("# A small thought\n\nCreamUI makes desktop interfaces feel calm.\n\nStart writing here — this is a real multiline editor.\nThe line count, word count, and character count react to each change.\n\n## Notes\n\n- Press Return for a new line\n- Backspace edits normally\n- The UI tree is declarative JSX".to_owned());
    let saved = Signal::new(false);
    let cursor = Signal::new(document.get().len());
    let active_menu = Signal::new(0_i32);
    let status_message = Signal::new("Markdown · UTF-8".to_owned());
    run(
        WindowOptions {
            title: "CreamUI — Text Editor".into(),
            width: 980,
            height: 680,
            ..Default::default()
        },
        WINDOW,
        |_| {},
        move |viewport: Size| -> BoxedWidget {
            let value = document.get();
            let lines = value.matches('\n').count() + 1;
            let words = value.split_whitespace().count();
            let chars = value.chars().count();
            let status_text = status_message.get();
            let on_change = document.clone();
            let saved_for_change = saved.clone();
            let cursor_for_change = cursor.clone();
            let mut editor_theme = creamui_theme::Theme::dark();
            editor_theme.surface = WINDOW;
            editor_theme.surface_elevated = CONTENT;
            let root = Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Length(viewport.width),
                    height: Dimension::Length(viewport.height),
                },
                flex_direction: FlexDirection::Column,
                ..Default::default()
            };
            let editor_row = Style {
                flex_grow: 1.0,
                size: creamui_core::layout::Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Auto,
                },
                ..row(0.0)
            };
            let area = Style {
                flex_grow: 1.0,
                size: creamui_core::layout::Size {
                    width: Dimension::Auto,
                    height: Dimension::Percent(1.0),
                },
                ..Default::default()
            };
            let status = Style {
                size: creamui_core::layout::Size { width: Dimension::Percent(1.0), height: Dimension::Length(30.0) },
                flex_shrink: 0.0,
                padding: creamui_core::layout::Rect {
                    left: creamui_core::layout::LengthPercentage::Length(16.0),
                    right: creamui_core::layout::LengthPercentage::Length(16.0),
                    top: creamui_core::layout::LengthPercentage::Length(0.0),
                    bottom: creamui_core::layout::LengthPercentage::Length(0.0),
                },
                justify_content: Some(JustifyContent::SpaceBetween),
                align_items: Some(AlignItems::Center),
                ..Default::default()
            };
            Box::new(jsx! {
                <RawView style={root} background={WINDOW}>
                    <EditorToolbar menus={vec!["File".into()]} title={"Untitled.md".into()} active={active_menu.clone()} children={Vec::<BoxedWidget>::new()} />
                    <RawView style={editor_row}>
                        <LineNumbers value={value.clone()} />
                        <TextArea theme={&editor_theme} style={area} value={value.clone()} cursor={cursor.get()} on_cursor_change={move |next| cursor_for_change.set(next)} on_change={move |next| { saved_for_change.set(false); on_change.set(next) }} placeholder={"Start writing…"} corner_radius={0.0} border_width={0.0} active_line_background={ACTIVE_LINE} />
                    </RawView>
                    <RawView style={status} background={WINDOW}>
                        <RawText color={BLUE} font_size={12.0} style={Style { flex_grow: 1.0, ..Default::default()}}>{status_text}</RawText>
                        <RawText color={MUTED} font_size={12.0} style={size(250.0, 24.0)} align={TextAlign::End}>{format!("{lines} lines · {words} words · {chars} characters")}</RawText>
                    </RawView>
                    <MenuPanel active={active_menu.clone()} document={document.clone()} saved={saved.clone()} status={status_message.clone()} />
                </RawView>
            })
        },
    );
}
