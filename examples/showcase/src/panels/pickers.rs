use crate::prelude::*;

/// Date/time, color, and file pickers live together because each returns a
/// value chosen from a structured external domain rather than free text.
/// The file picker uses the operating system dialog; the others keep their
/// controlled values in the showcase's regular reactive state.
#[component]
pub fn PickersPanel(
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
