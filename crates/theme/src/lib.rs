//! Theme primitives split deliberately in two: [`ColorScheme`] owns colours,
//! while [`Theme`] owns the shape and behaviour of components.

/// An 8-bit sRGB color with alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub fn to_f32(self) -> [f32; 4] {
        [
            self.r as f32 / 255.,
            self.g as f32 / 255.,
            self.b as f32 / 255.,
            self.a as f32 / 255.,
        ]
    }
}

/// All colour tokens. This can be changed independently from a [`Theme`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_hover: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub selection_background: Color,
    pub selection_text: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub border: Color,
    pub border_strong: Color,
    pub danger: Color,
    pub warning: Color,
    pub success: Color,
}
impl ColorScheme {
    pub const fn dark() -> Self {
        Self {
            surface: Color::rgb(0x1a, 0x1b, 0x1e),
            surface_elevated: Color::rgb(0x24, 0x25, 0x2a),
            surface_hover: Color::rgb(0x2c, 0x2d, 0x33),
            accent: Color::rgb(0x7c, 0x5c, 0xff),
            accent_hover: Color::rgb(0x8d, 0x71, 0xff),
            accent_pressed: Color::rgb(0x6a, 0x4a, 0xe6),
            selection_background: Color::rgb(0x0a, 0x84, 0xff),
            selection_text: Color::rgb(0xff, 0xff, 0xff),
            text_primary: Color::rgb(0xf2, 0xf2, 0xf5),
            text_secondary: Color::rgb(0xa4, 0xa5, 0xad),
            text_disabled: Color::rgb(0x5c, 0x5d, 0x64),
            border: Color::rgb(0x35, 0x36, 0x3d),
            border_strong: Color::rgb(0x4a, 0x4b, 0x54),
            danger: Color::rgb(0xe5, 0x4b, 0x4b),
            warning: Color::rgb(0xe0, 0xa5, 0x2e),
            success: Color::rgb(0x3d, 0xc9, 0x6f),
        }
    }
    pub const fn light() -> Self {
        Self {
            surface: Color::rgb(0xfa, 0xfa, 0xfb),
            surface_elevated: Color::rgb(0xff, 0xff, 0xff),
            surface_hover: Color::rgb(0xef, 0xef, 0xf2),
            accent: Color::rgb(0x6a, 0x4a, 0xe6),
            accent_hover: Color::rgb(0x7c, 0x5c, 0xff),
            accent_pressed: Color::rgb(0x59, 0x3c, 0xcc),
            selection_background: Color::rgb(0x0a, 0x66, 0xcc),
            selection_text: Color::rgb(0xff, 0xff, 0xff),
            text_primary: Color::rgb(0x1a, 0x1b, 0x1e),
            text_secondary: Color::rgb(0x54, 0x55, 0x5c),
            text_disabled: Color::rgb(0xa8, 0xa9, 0xb0),
            border: Color::rgb(0xdf, 0xe0, 0xe3),
            border_strong: Color::rgb(0xc4, 0xc5, 0xca),
            danger: Color::rgb(0xd1, 0x3a, 0x3a),
            warning: Color::rgb(0xb8, 0x7d, 0x0a),
            success: Color::rgb(0x22, 0xa0, 0x55),
        }
    }
}
impl Default for ColorScheme {
    fn default() -> Self {
        Self::dark()
    }
}

/// How a selected tab or sidebar item communicates selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStyle {
    Filled,
    Indicator,
}

/// Component geometry and interaction-style metadata. It intentionally has
/// no colours, so the same style works with every accent and light/dark mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// Kept as a nested value for ergonomic backwards compatibility. New
    /// code should keep and swap a `ColorScheme` independently (or use the
    /// two providers); style tokens below never encode a colour decision.
    pub colors: ColorScheme,
    pub name: &'static str,
    pub radius_small: f32,
    pub radius_medium: f32,
    pub radius_large: f32,
    pub spacing_small: f32,
    pub spacing_medium: f32,
    pub spacing_large: f32,
    pub button_radius: f32,
    pub checkbox_radius: f32,
    pub input_radius: f32,
    pub textarea_radius: f32,
    pub input_border_width: f32,
    pub card_radius: f32,
    pub scroll_radius: f32,
    pub tabs_radius: f32,
    pub tab_radius: f32,
    pub tab_selection: SelectionStyle,
    pub sidebar_radius: f32,
    pub sidebar_item_radius: f32,
    pub sidebar_selection: SelectionStyle,
    pub indicator_thickness: f32,
    pub tab_gap: f32,
    pub sidebar_gap: f32,
    pub sidebar_icon_size: f32,
    pub sidebar_icon_radius: f32,
    pub sidebar_item_gap: f32,
    pub menu_radius: f32,
    pub menu_item_radius: f32,
}
impl Theme {
    /// The soft, rounded Cream style. Built-in presets in `creamui-themes`
    /// expose this as `Cream`; this definition keeps the core crate cycle-free.
    pub const fn cream() -> Self {
        Self {
            colors: ColorScheme::dark(),
            name: "Cream",
            radius_small: 8.,
            radius_medium: 12.,
            radius_large: 20.,
            spacing_small: 4.,
            spacing_medium: 8.,
            spacing_large: 16.,
            button_radius: 12.,
            checkbox_radius: 6.,
            input_radius: 10.,
            textarea_radius: 12.,
            input_border_width: 1.,
            card_radius: 16.,
            scroll_radius: 16.,
            tabs_radius: 14.,
            tab_radius: 10.,
            tab_selection: SelectionStyle::Filled,
            sidebar_radius: 16.,
            sidebar_item_radius: 10.,
            sidebar_selection: SelectionStyle::Filled,
            indicator_thickness: 3.,
            tab_gap: 4.,
            sidebar_gap: 6.,
            sidebar_icon_size: 16.,
            sidebar_icon_radius: 5.,
            sidebar_item_gap: 9.,
            menu_radius: 10.,
            menu_item_radius: 7.,
        }
    }
    /// The original restrained, square-ish indicator treatment.
    pub const fn square() -> Self {
        Self {
            colors: ColorScheme::dark(),
            name: "Square",
            radius_small: 4.,
            radius_medium: 8.,
            radius_large: 16.,
            spacing_small: 4.,
            spacing_medium: 8.,
            spacing_large: 16.,
            button_radius: 8.,
            checkbox_radius: 4.,
            input_radius: 4.,
            textarea_radius: 8.,
            input_border_width: 1.,
            card_radius: 8.,
            scroll_radius: 8.,
            tabs_radius: 0.,
            tab_radius: 0.,
            tab_selection: SelectionStyle::Indicator,
            sidebar_radius: 0.,
            sidebar_item_radius: 0.,
            sidebar_selection: SelectionStyle::Indicator,
            indicator_thickness: 3.,
            tab_gap: 16.,
            sidebar_gap: 2.,
            sidebar_icon_size: 14.,
            sidebar_icon_radius: 2.,
            sidebar_item_gap: 8.,
            menu_radius: 4.,
            menu_item_radius: 3.,
        }
    }
    pub const fn with_colors(mut self, colors: ColorScheme) -> Self {
        self.colors = colors;
        self
    }
    pub const fn dark() -> Self {
        Self::cream().with_colors(ColorScheme::dark())
    }
    pub const fn light() -> Self {
        Self::cream().with_colors(ColorScheme::light())
    }
}
impl std::ops::Deref for Theme {
    type Target = ColorScheme;
    fn deref(&self) -> &Self::Target {
        &self.colors
    }
}
impl Default for Theme {
    fn default() -> Self {
        Self::cream()
    }
}

/// Reactive provider for a style theme.
#[derive(Clone)]
pub struct ThemeProvider {
    theme: creamui_reactive::Signal<Theme>,
}
impl ThemeProvider {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme: creamui_reactive::Signal::new(theme),
        }
    }
    pub fn get(&self) -> Theme {
        self.theme.get()
    }
    pub fn set(&self, theme: Theme) {
        self.theme.set(theme)
    }
}
impl Default for ThemeProvider {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}

/// Reactive provider for an independently configurable colour scheme.
#[derive(Clone)]
pub struct ColorSchemeProvider {
    colors: creamui_reactive::Signal<ColorScheme>,
}
impl ColorSchemeProvider {
    pub fn new(colors: ColorScheme) -> Self {
        Self {
            colors: creamui_reactive::Signal::new(colors),
        }
    }
    pub fn get(&self) -> ColorScheme {
        self.colors.get()
    }
    pub fn set(&self, colors: ColorScheme) {
        self.colors.set(colors)
    }
}
impl Default for ColorSchemeProvider {
    fn default() -> Self {
        Self::new(ColorScheme::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_is_rounded_cream() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Cream");
        assert_eq!(theme.tab_selection, SelectionStyle::Filled);
        assert_eq!(theme.sidebar_selection, SelectionStyle::Filled);
        assert!(theme.input_radius > Theme::square().input_radius);
    }

    #[test]
    fn palette_can_change_without_changing_style() {
        let cream = Theme::cream();
        let light = cream.with_colors(ColorScheme::light());
        assert_eq!(cream.name, light.name);
        assert_eq!(cream.tab_radius, light.tab_radius);
        assert_ne!(cream.colors, light.colors);
    }
}
