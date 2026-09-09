use crate::prelude::*;

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
pub fn SliderPanel(
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
