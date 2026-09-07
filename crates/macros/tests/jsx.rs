use creamui_core::layout::Style;
use creamui_core::{render_frame, BoxedWidget, Painter, Rect, Size, TextAlign};
use creamui_macros::{abi_jsx, component, jsx};
use creamui_reactive::Signal;
use creamui_theme::{Color, Theme};

#[derive(Default)]
struct TextPainter(Vec<String>);

#[component]
fn CounterLabel(theme: Theme, value: i32) -> BoxedWidget {
    Box::new(jsx! { <Text theme={&theme}>{format!("Custom: {value}")}</Text> })
}

#[component]
fn Panel(children: Vec<BoxedWidget>) -> BoxedWidget {
    Box::new(creamui_widgets::raw::RawView::new(Style::default()).with_children(children))
}

#[component]
fn AbiLabel(
    ctx: creamui_dynamic::Context,
    theme: creamui_dynamic::Theme,
    label: String,
) -> creamui_dynamic::Widget {
    abi_jsx! { <Text ctx={&ctx} theme={theme}>{label}</Text> }
}

#[allow(dead_code)]
fn build_abi_component(
    ctx: &creamui_dynamic::Context,
    theme: creamui_dynamic::Theme,
) -> creamui_dynamic::Widget {
    abi_jsx! { <AbiLabel ctx={ctx.clone()} theme={theme} label={"from ABI".to_string()} /> }
}

impl Painter for TextPainter {
    fn fill_rect(&mut self, _: Rect, _: Color, _: f32) {}
    fn stroke_rect(&mut self, _: Rect, _: Color, _: f32, _: f32) {}
    fn fill_text(&mut self, _: Rect, text: &str, _: Color, _: f32, _: TextAlign) {
        self.0.push(text.into());
    }
}

#[test]
fn jsx_expands_to_the_existing_widget_builders() {
    let theme = Theme::dark();
    let clicks = Signal::new(0);
    let clicks_for_handler = clicks.clone();
    let root: BoxedWidget = Box::new(jsx! {
        <View theme={&theme} style={Style::default()}>
            <Text theme={&theme} font_size={20.0}>{format!("Clicked {} times", clicks.get())}</Text>
            <Button theme={&theme} on_click={move || clicks_for_handler.update(|value| *value += 1)}>"Increment"</Button>
        </View>
    });

    let mut painter = TextPainter::default();
    let scene = render_frame(
        root,
        Size {
            width: 300.0,
            height: 120.0,
        },
        &mut painter,
    );
    assert_eq!(painter.0, ["Clicked 0 times", "Increment"]);
    let click = (0..300)
        .step_by(4)
        .flat_map(|x| (0..120).step_by(4).map(move |y| (x, y)))
        .find_map(|(x, y)| {
            scene.hit_test(creamui_core::Point {
                x: x as f32,
                y: y as f32,
            })
        })
        .expect("button hit");
    click();
    assert_eq!(clicks.get(), 1);
}

#[test]
fn application_components_are_typed_functions_not_macro_registrations() {
    let theme = Theme::dark();
    let root: BoxedWidget = Box::new(jsx! {
        <View theme={&theme} style={Style::default()}>
            <CounterLabel theme={theme} value={7} />
        </View>
    });
    let mut painter = TextPainter::default();
    render_frame(
        root,
        Size {
            width: 200.0,
            height: 80.0,
        },
        &mut painter,
    );
    assert_eq!(painter.0, ["Custom: 7"]);
}

#[test]
fn application_components_can_receive_nested_jsx_children() {
    let theme = Theme::dark();
    let root: BoxedWidget = jsx! {
        <Panel>
            <Text theme={&theme}>"Nested"</Text>
        </Panel>
    };
    let mut painter = TextPainter::default();
    render_frame(
        root,
        Size {
            width: 200.0,
            height: 80.0,
        },
        &mut painter,
    );
    assert_eq!(painter.0, ["Nested"]);
}

#[test]
fn jsx_exposes_headless_text_and_buttons_with_layout_props() {
    let theme = Theme::dark();
    let root: BoxedWidget = Box::new(jsx! {
        <RawView style={Style::default()}>
            <RawButton style={Style::default()} background={Color::rgb(20, 20, 20)} corner_radius={12.0} on_click={|| {}}>
                <RawText color={Color::rgb(255, 200, 0)} font_size={18.0} align={TextAlign::End} style={Style::default()}>"Raw label"</RawText>
            </RawButton>
            <Text theme={&theme} color={Color::rgb(120, 220, 255)} align={TextAlign::Start} style={Style::default()}>"Themed label"</Text>
        </RawView>
    });
    let mut painter = TextPainter::default();
    render_frame(
        root,
        Size {
            width: 200.0,
            height: 80.0,
        },
        &mut painter,
    );
    assert_eq!(painter.0, ["Raw label", "Themed label"]);
}

#[test]
fn raw_view_accepts_a_generated_children_list() {
    let children: Vec<BoxedWidget> = vec![Box::new(creamui_widgets::raw::RawText::new(
        "Generated",
        Color::rgb(255, 255, 255),
        14.0,
    ))];
    let root: BoxedWidget = Box::new(jsx! {
        <RawView style={Style::default()} children={children} />
    });
    let mut painter = TextPainter::default();
    render_frame(
        root,
        Size {
            width: 200.0,
            height: 80.0,
        },
        &mut painter,
    );
    assert_eq!(painter.0, ["Generated"]);
}
