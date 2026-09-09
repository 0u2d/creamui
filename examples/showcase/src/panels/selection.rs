use crate::prelude::*;

/// Select, radio, and segmented selection share a page. `SegmentedControl`
/// owns a row of the lower-level `Choice` items, so the demo does not repeat
/// the same interaction as two competing controls.
#[component]
pub fn SelectionPanel(
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
    let list = ListBox::new(list_box_style, list_scroll, list_value, move |index| {
        set_list.set(index)
    })
    .options(&FRUITS);
    Box::new(jsx! {
        <RawView style={column(section_gap())}>
            <SectionHeader title={"Selection".to_owned()} subtitle={"Choose one value with a popup, explanatory radios, compact Choice segments, or a scrollable list.".to_owned()} />
            {field_card("Select / ComboBox", Box::new(Select::controlled(&OPTIONS, select)))}
            {card_row(vec![
                field_card("Radio group", Box::new(
                    RadioGroup::new(radio_value, move |index| set_radio.set(index))
                        .option("Keep files on this device")
                        .option("Sync encrypted copies")
                        .option("Never sync"),
                )),
                field_card("Segmented control", Box::new(
                    SegmentedControl::new(segment_value, move |index| set_segment.set(index))
                        .option("Day").option("Week").option("Month"),
                )),
            ])}
            {field_card(&format!("List box · {}", FRUITS[list_value]), Box::new(list))}
        </RawView>
    })
}
