use super::*;
/// A controlled text selection represented as byte offsets into a UTF-8
/// document. `anchor` stays at the point where selection began while `focus`
/// follows the caret or pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextSelection {
    pub anchor: usize,
    pub focus: usize,
}

impl TextSelection {
    pub fn range(self) -> std::ops::Range<usize> {
        self.anchor.min(self.focus)..self.anchor.max(self.focus)
    }

    pub fn is_empty(self) -> bool {
        self.anchor == self.focus
    }
}

/// An unstyled rectangular container that lays out its children.
pub struct RawView {
    pub style: Style,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub children: Vec<BoxedWidget>,
}

impl RawView {
    pub fn new(style: Style) -> Self {
        RawView {
            style,
            background: None,
            corner_radius: 0.0,
            children: Vec::new(),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Replaces the layout style. Useful when a base style is refined by a
    /// reusable component before it is returned.
    pub fn layout_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn child(mut self, widget: BoxedWidget) -> Self {
        self.children.push(widget);
        self
    }

    pub fn with_children(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.children = widgets;
        self
    }
}

impl Widget for RawView {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        std::mem::take(&mut self.children)
    }
}

/// Unstyled text with no color or size opinion beyond what's passed in.
pub struct RawText {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub text: String,
    pub color: Color,
    pub font_size: f32,
    pub align: TextAlign,
    pub style: Style,
    /// CSS-style family stack (e.g. `"Inter, sans-serif"`) resolved against
    /// the font registry. `None` uses the bundled default.
    pub family: Option<String>,
}

impl RawText {
    pub fn new(text: impl Into<String>, color: Color, font_size: f32) -> Self {
        RawText {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            text: text.into(),
            color,
            font_size,
            align: TextAlign::Center,
            style: Style::default(),
            family: None,
        }
    }

    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    /// Synthesized by shearing the glyph raster (no italic face is
    /// bundled), so it combines freely with [`RawText::bold`].
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.strikethrough = strikethrough;
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn layout_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for RawText {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        painter.fill_text_font(
            rect,
            &self.text,
            self.color,
            self.font_size,
            self.align,
            self.family.as_deref(),
            self.bold,
            self.italic,
        );
        super::draw_text_decorations(
            painter,
            rect,
            &self.text,
            self.font_size,
            self.bold,
            self.align,
            self.color,
            self.underline,
            self.strikethrough,
        );
    }

    fn measure(&self) -> Option<creamui_core::MeasureFn> {
        let text = self.text.clone();
        let font_size = self.font_size;
        let bold = self.bold;
        let family = self.family.clone();
        Some(Box::new(move |known_dimensions, available_space| {
            let max_width = match (known_dimensions.width, available_space.width) {
                (Some(w), _) => w,
                (None, creamui_core::layout::AvailableSpace::Definite(w)) => w,
                (None, _) => crate::text_metrics::unbounded_width(),
            };
            let (natural_width, natural_height) = crate::text_metrics::measure_family(
                &text,
                font_size,
                max_width,
                family.as_deref(),
                bold,
            );
            creamui_core::layout::Size {
                width: known_dimensions.width.unwrap_or(natural_width),
                height: known_dimensions.height.unwrap_or(natural_height),
            }
        }))
    }
}
