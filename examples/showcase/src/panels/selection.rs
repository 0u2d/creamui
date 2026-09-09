use crate::prelude::*;

/// Select, radio, and segmented selection share a page. `SegmentedControl`
/// owns a row of the lower-level `Choice` items, so the demo does not repeat
/// the same interaction as two competing controls.
#[component]
pub fn SelectionPanel(
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
