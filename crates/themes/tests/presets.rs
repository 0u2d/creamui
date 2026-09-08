use creamui_theme::SelectionStyle;
use creamui_themes::{Cream, Square};

#[test]
fn presets_expose_the_named_styles() {
    assert_eq!(Cream::theme().name, "Cream");
    assert_eq!(Cream::theme().tab_selection, SelectionStyle::Filled);
    assert_eq!(Square::theme().name, "Square");
    assert_eq!(Square::theme().tab_selection, SelectionStyle::Indicator);
}
