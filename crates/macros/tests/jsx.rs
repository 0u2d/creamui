use creamui_core::layout::Style;
use creamui_core::{render_frame, BoxedWidget, Painter, Rect, Size, TextAlign};
use creamui_macros::jsx;
use creamui_reactive::Signal;
use creamui_theme::{Color, Theme};

#[derive(Default)]
struct TextPainter(Vec<String>);

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
