//! Semantic design tokens for CreamUI, in the spirit of CSS custom properties
//! or Tailwind's theme scale: components read tokens like `accent` or
//! `radiusMedium` instead of hardcoded colors or pixel values, so swapping a
//! [`Theme`] restyles every themed component at once.

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
        Color { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Components as normalized `[0.0, 1.0]` floats, in the order the GPU
    /// renderer and `tiny_skia` both expect.
    pub fn to_f32(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

/// A full set of semantic design tokens.
///
/// This is the complete surface themed widgets are allowed to depend on;
/// widgets must never reach for raw colors or magic pixel values, so a theme
/// swap always restyles the whole tree consistently.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_hover: Color,

    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    pub border: Color,
    pub border_strong: Color,

    pub danger: Color,
    pub warning: Color,
    pub success: Color,

    pub radius_small: f32,
    pub radius_medium: f32,
    pub radius_large: f32,

    pub spacing_small: f32,
    pub spacing_medium: f32,
    pub spacing_large: f32,
}

impl Theme {
    /// The bundled default dark theme. CreamUI ships with exactly one
    /// built-in theme for the MVP; a runtime `ThemeProvider` that lets apps
    /// switch or override themes is tracked on the roadmap.
    pub const fn dark() -> Self {
        Theme {
            surface: Color::rgb(0x1a, 0x1b, 0x1e),
            surface_elevated: Color::rgb(0x24, 0x25, 0x2a),
            surface_hover: Color::rgb(0x2c, 0x2d, 0x33),

            accent: Color::rgb(0x7c, 0x5c, 0xff),
            accent_hover: Color::rgb(0x8d, 0x71, 0xff),
            accent_pressed: Color::rgb(0x6a, 0x4a, 0xe6),

            text_primary: Color::rgb(0xf2, 0xf2, 0xf5),
            text_secondary: Color::rgb(0xa4, 0xa5, 0xad),
            text_disabled: Color::rgb(0x5c, 0x5d, 0x64),

            border: Color::rgb(0x35, 0x36, 0x3d),
            border_strong: Color::rgb(0x4a, 0x4b, 0x54),

            danger: Color::rgb(0xe5, 0x4b, 0x4b),
            warning: Color::rgb(0xe0, 0xa5, 0x2e),
            success: Color::rgb(0x3d, 0xc9, 0x6f),

            radius_small: 4.0,
            radius_medium: 8.0,
            radius_large: 16.0,

            spacing_small: 4.0,
            spacing_medium: 8.0,
            spacing_large: 16.0,
        }
    }

    /// The bundled default light theme.
    pub const fn light() -> Self {
        Theme {
            surface: Color::rgb(0xfa, 0xfa, 0xfb),
            surface_elevated: Color::rgb(0xff, 0xff, 0xff),
            surface_hover: Color::rgb(0xef, 0xef, 0xf2),

            accent: Color::rgb(0x6a, 0x4a, 0xe6),
            accent_hover: Color::rgb(0x7c, 0x5c, 0xff),
            accent_pressed: Color::rgb(0x59, 0x3c, 0xcc),

            text_primary: Color::rgb(0x1a, 0x1b, 0x1e),
            text_secondary: Color::rgb(0x54, 0x55, 0x5c),
            text_disabled: Color::rgb(0xa8, 0xa9, 0xb0),

            border: Color::rgb(0xdf, 0xe0, 0xe3),
            border_strong: Color::rgb(0xc4, 0xc5, 0xca),

            danger: Color::rgb(0xd1, 0x3a, 0x3a),
            warning: Color::rgb(0xb8, 0x7d, 0x0a),
            success: Color::rgb(0x22, 0xa0, 0x55),

            radius_small: 4.0,
            radius_medium: 8.0,
            radius_large: 16.0,

            spacing_small: 4.0,
            spacing_medium: 8.0,
            spacing_large: 16.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_to_f32_normalizes() {
        assert_eq!(Color::rgb(255, 0, 128).to_f32(), [1.0, 0.0, 128.0 / 255.0, 1.0]);
    }

    #[test]
    fn dark_and_light_themes_differ() {
        assert_ne!(Theme::dark(), Theme::light());
    }

    #[test]
    fn default_theme_is_dark() {
        assert_eq!(Theme::default(), Theme::dark());
    }
}
