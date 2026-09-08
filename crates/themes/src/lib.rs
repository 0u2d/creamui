//! Bundled, colour-agnostic CreamUI visual themes.
use creamui_theme::Theme;

/// The default soft, colourful-control-friendly Cream visual style.
pub struct Cream;
impl Cream {
    pub const THEME: Theme = Theme::cream();
    pub const fn theme() -> Theme {
        Self::THEME
    }
}
/// The prior indicator-line visual style, retained as an opt-in preset.
pub struct Square;
impl Square {
    pub const THEME: Theme = Theme::square();
    pub const fn theme() -> Theme {
        Self::THEME
    }
}
pub const CREAM: Theme = Cream::THEME;
pub const SQUARE: Theme = Square::THEME;
