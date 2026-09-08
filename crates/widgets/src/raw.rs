//! Headless widgets: fully unstyled building blocks with no opinion on
//! color, radius, or spacing. Themed widgets (see [`crate::themed`]) wrap
//! these and fill in appearance from a [`creamui_theme::Theme`]; apps that
//! want a completely custom look can use these directly instead.

use creamui_core::layout::Style;
use creamui_core::{
    BoxedWidget, CursorIcon, Key, KeyInput, Painter, Point, Rect, TextAlign, Widget,
};
use creamui_theme::Color;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    // On X11/Wayland the clipboard owner must remain alive after the write;
    // creating and dropping `arboard::Clipboard` inside a key callback makes
    // clipboard managers lose the contents immediately.
    static SYSTEM_CLIPBOARD: RefCell<Option<arboard::Clipboard>> = const { RefCell::new(None) };
}

fn clipboard_write(text: String) {
    SYSTEM_CLIPBOARD.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = arboard::Clipboard::new().ok();
        }
        if let Some(clipboard) = slot.as_mut() {
            let _ = clipboard.set_text(text);
        }
    });
}

fn clipboard_read() -> Option<String> {
    SYSTEM_CLIPBOARD.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = arboard::Clipboard::new().ok();
        }
        slot.as_mut()
            .and_then(|clipboard| clipboard.get_text().ok())
    })
}

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
    pub text: String,
    pub color: Color,
    pub font_size: f32,
    pub align: TextAlign,
    pub style: Style,
}

impl RawText {
    pub fn new(text: impl Into<String>, color: Color, font_size: f32) -> Self {
        RawText {
            text: text.into(),
            color,
            font_size,
            align: TextAlign::Center,
            style: Style::default(),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
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
        painter.fill_text(rect, &self.text, self.color, self.font_size, self.align);
    }

    fn measure(&self) -> Option<creamui_core::MeasureFn> {
        let text = self.text.clone();
        let font_size = self.font_size;
        Some(Box::new(move |known_dimensions, available_space| {
            let max_width = match (known_dimensions.width, available_space.width) {
                (Some(w), _) => w,
                (None, creamui_core::layout::AvailableSpace::Definite(w)) => w,
                (None, _) => crate::text_metrics::unbounded_width(),
            };
            let (natural_width, natural_height) =
                crate::text_metrics::measure(&text, font_size, max_width);
            creamui_core::layout::Size {
                width: known_dimensions.width.unwrap_or(natural_width),
                height: known_dimensions.height.unwrap_or(natural_height),
            }
        }))
    }
}

/// An unstyled clickable region. Paints only its `background`/`border` if
/// set; combine with [`RawText`] as a child for a labeled button.
pub struct RawButton {
    pub style: Style,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub children: Vec<BoxedWidget>,
    pub on_click: Rc<dyn Fn()>,
    pub disabled: bool,
}

impl RawButton {
    pub fn new(style: Style, on_click: impl Fn() + 'static) -> Self {
        RawButton {
            style,
            background: None,
            corner_radius: 0.0,
            children: Vec::new(),
            on_click: Rc::new(on_click),
            disabled: false,
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

    /// While `true`, the button reports no click handler (so it truly can't
    /// be activated, not just visually dimmed) and shows a "not allowed"
    /// cursor instead of the usual pointer.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Widget for RawButton {
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

    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        if self.disabled {
            None
        } else {
            Some(self.on_click.clone())
        }
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(if self.disabled {
            CursorIcon::NotAllowed
        } else {
            CursorIcon::Pointer
        })
    }
}

/// An unstyled single-line text input. The caller owns the current text
/// (typically a `String` `Signal`) and updates it from `on_change`, fired
/// on every keystroke — same "no internal state" pattern as every other
/// widget. Supports appending characters and backspace; cursor
/// positioning/selection is not implemented yet (see ROADMAP.md).
pub struct RawTextInput {
    pub style: Style,
    pub value: String,
    pub placeholder: String,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub background: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub font_size: f32,
    pub cursor: usize,
    pub selection: TextSelection,
    pub selection_background: Option<Color>,
    pub selection_text_color: Option<Color>,
    pub on_change: Rc<dyn Fn(String)>,
    pub on_cursor_change: Rc<dyn Fn(usize)>,
    pub on_selection_change: Rc<dyn Fn(TextSelection)>,
    pub clipboard_enabled: bool,
    keyboard_selection: Rc<Cell<TextSelection>>,
    drag_anchor: Rc<Cell<usize>>,
}

/// An unstyled multi-line text editor. Like [`RawTextInput`], its value is
/// owned by the caller; this deliberately keeps editing state compatible
/// with CreamUI's reactive, rebuild-on-change model.
pub struct RawTextArea {
    pub style: Style,
    pub value: String,
    pub placeholder: String,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub background: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub font_size: f32,
    pub cursor: usize,
    pub alternating_line_background: Option<Color>,
    pub active_line_background: Option<Color>,
    pub selection: TextSelection,
    pub selection_background: Option<Color>,
    pub selection_text_color: Option<Color>,
    pub on_change: Rc<dyn Fn(String)>,
    pub on_cursor_change: Rc<dyn Fn(usize)>,
    pub on_selection_change: Rc<dyn Fn(TextSelection)>,
    pub on_ctrl_o: Rc<dyn Fn()>,
    pub clipboard_enabled: bool,
    pub wrap: bool,
    // Pointer interaction can outlive a reactive frame when renders are
    // coalesced. This tiny ephemeral cell keeps drag selection anchored
    // without requiring every mouse move to rebuild the widget tree.
    drag_anchor: Rc<Cell<usize>>,
    drag_focus: Rc<Cell<usize>>,
    keyboard_selection: Rc<Cell<TextSelection>>,
}

impl RawTextArea {
    pub fn new(
        style: Style,
        value: impl Into<String>,
        font_size: f32,
        text_color: Color,
        on_change: impl Fn(String) + 'static,
    ) -> Self {
        let value = value.into();
        let cursor = value.len();
        Self {
            style,
            value,
            placeholder: String::new(),
            text_color,
            placeholder_color: text_color,
            background: None,
            border_color: None,
            border_width: 1.0,
            corner_radius: 0.0,
            font_size,
            cursor,
            alternating_line_background: None,
            active_line_background: None,
            selection: TextSelection {
                anchor: cursor,
                focus: cursor,
            },
            selection_background: None,
            selection_text_color: None,
            on_change: Rc::new(on_change),
            on_cursor_change: Rc::new(|_| {}),
            on_selection_change: Rc::new(|_| {}),
            on_ctrl_o: Rc::new(|| {}),
            clipboard_enabled: true,
            wrap: false,
            drag_anchor: Rc::new(Cell::new(cursor)),
            drag_focus: Rc::new(Cell::new(cursor)),
            keyboard_selection: Rc::new(Cell::new(TextSelection {
                anchor: cursor,
                focus: cursor,
            })),
        }
    }

    /// Supplies a controlled byte-index cursor and receives updates from
    /// keyboard navigation or pointer placement.
    pub fn cursor(mut self, cursor: usize, on_change: impl Fn(usize) + 'static) -> Self {
        self.cursor = cursor.min(self.value.len());
        self.drag_anchor.set(self.cursor);
        self.drag_focus.set(self.cursor);
        self.keyboard_selection.set(TextSelection {
            anchor: self.cursor,
            focus: self.cursor,
        });
        self.on_cursor_change = Rc::new(on_change);
        self
    }

    /// Supplies a controlled selection. The application owns it just like it
    /// owns `value` and `cursor`, allowing selection appearance/state to be
    /// coordinated across native, JSX and ABI-built UIs.
    pub fn selection(
        mut self,
        selection: TextSelection,
        on_change: impl Fn(TextSelection) + 'static,
    ) -> Self {
        self.selection = TextSelection {
            anchor: selection.anchor.min(self.value.len()),
            focus: selection.focus.min(self.value.len()),
        };
        self.drag_anchor.set(self.selection.anchor);
        self.drag_focus.set(self.selection.focus);
        self.keyboard_selection.set(self.selection);
        self.on_selection_change = Rc::new(on_change);
        self
    }

    /// Visual token for selected text. The text itself remains in the
    /// editor's normal color until rich text spans land in the renderer.
    pub fn selection_background(mut self, color: Color) -> Self {
        self.selection_background = Some(color);
        self
    }

    /// Foreground token used for the selected text. Pair it with
    /// [`Self::selection_background`] to make an editor's selection fully
    /// match its design system.
    pub fn selection_text_color(mut self, color: Color) -> Self {
        self.selection_text_color = Some(color);
        self
    }

    /// Invoked by the conventional Ctrl/Cmd+O command while this editor has
    /// focus. The app decides what opening a document means.
    pub fn on_ctrl_o(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_ctrl_o = Rc::new(callback);
        self
    }

    /// Enables the platform clipboard shortcuts (Ctrl/Cmd+A, C and V).
    /// Enabled by default; disable it for sensitive or deliberately isolated
    /// editors without changing their keyboard-editing behavior.
    pub fn clipboard_enabled(mut self, enabled: bool) -> Self {
        self.clipboard_enabled = enabled;
        self
    }

    /// When `true`, long lines break onto a new visual row at the editor's
    /// width instead of overflowing it — the caret and click-to-position
    /// follow the wrapped rows too. Off by default (a line just scrolls
    /// horizontally, as `RawTextInput` does).
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
    pub fn placeholder(mut self, text: impl Into<String>, color: Color) -> Self {
        self.placeholder = text.into();
        self.placeholder_color = color;
        self
    }

    /// Paints every second source line with a subtle reading-guide color.
    pub fn alternating_line_background(mut self, color: Color) -> Self {
        self.alternating_line_background = Some(color);
        self
    }

    /// Highlights the source line containing the caret.
    pub fn active_line_background(mut self, color: Color) -> Self {
        self.active_line_background = Some(color);
        self
    }

    /// How far to shift every line left so the caret stays inside
    /// `visible_width` instead of running off the unwrapped line's edge.
    fn horizontal_scroll(&self, visible_width: f32) -> f32 {
        let cursor = self.cursor.min(self.value.len());
        let line_start = self.value[..cursor].rfind('\n').map_or(0, |i| i + 1);
        let (cursor_x, _) = crate::text_metrics::measure(
            &self.value[line_start..cursor],
            self.font_size,
            crate::text_metrics::unbounded_width(),
        );
        (cursor_x - visible_width + 4.0).max(0.0)
    }

    /// One row per source line, unbounded width, scrolled horizontally so
    /// the caret stays visible — no wrapping.
    fn paint_unwrapped(
        &self,
        painter: &mut dyn Painter,
        text_rect: Rect,
        text: &str,
        color: Color,
    ) {
        let line_height = self.font_size * 1.4;
        let active_line = self.value[..self.cursor.min(self.value.len())]
            .matches('\n')
            .count();
        let scroll_x = self.horizontal_scroll(text_rect.width);
        let selected = self.selection.range();
        let mut source_offset = 0;
        for (index, line) in text.split('\n').enumerate() {
            let line_rect = Rect {
                y: text_rect.y + index as f32 * line_height,
                height: line_height,
                ..text_rect
            };
            let unbounded_line_rect = Rect {
                x: line_rect.x - scroll_x,
                width: crate::text_metrics::unbounded_width(),
                ..line_rect
            };
            if index == active_line {
                if let Some(background) = self.active_line_background {
                    painter.fill_rect(line_rect, background, 0.0);
                }
            } else if index % 2 == 1 {
                if let Some(background) = self.alternating_line_background {
                    painter.fill_rect(line_rect, background, 0.0);
                }
            }
            if !self.value.is_empty() {
                let line_end = source_offset + line.len();
                let start = selected.start.max(source_offset).min(line_end);
                let end = selected.end.max(source_offset).min(line_end);
                if start < end {
                    if let Some(background) = self.selection_background {
                        let prefix = &line[..start - source_offset];
                        let selected_text = &line[start - source_offset..end - source_offset];
                        let (x, _) = crate::text_metrics::measure(
                            prefix,
                            self.font_size,
                            crate::text_metrics::unbounded_width(),
                        );
                        let (width, _) = crate::text_metrics::measure(
                            selected_text,
                            self.font_size,
                            crate::text_metrics::unbounded_width(),
                        );
                        painter.fill_rect(
                            Rect {
                                x: line_rect.x + x - scroll_x,
                                width,
                                ..line_rect
                            },
                            background,
                            2.0,
                        );
                    }
                    painter.fill_text_selected(
                        unbounded_line_rect,
                        line,
                        color,
                        self.selection_text_color.unwrap_or(color),
                        start - source_offset..end - source_offset,
                        self.font_size,
                        TextAlign::Start,
                    );
                    source_offset = line_end + 1;
                    continue;
                }
                source_offset = line_end + 1;
            }
            painter.fill_text(
                unbounded_line_rect,
                line,
                color,
                self.font_size,
                TextAlign::Start,
            );
        }
    }

    /// Bounded width, letting `fontdue` wrap long lines onto new visual
    /// rows; selection is highlighted per glyph since rows no longer line
    /// up with source lines.
    fn paint_wrapped(&self, painter: &mut dyn Painter, text_rect: Rect, text: &str, color: Color) {
        // `Painter::fill_text` centers a block vertically within the rect
        // it's given. A rect as tall as the whole editor would center a
        // short wrapped block partway down it, desyncing every y this
        // module computes (which all assume the first row starts at the
        // rect's very top). Sizing the rect to the block's own height
        // makes that centering a no-op.
        let block_rect = Rect {
            height: crate::text_metrics::content_height(text, self.font_size, text_rect.width)
                .max(crate::text_metrics::row_height(self.font_size)),
            ..text_rect
        };
        let selected = self.selection.range();
        if !self.value.is_empty() && !selected.is_empty() {
            if let Some(background) = self.selection_background {
                for glyph in crate::text_metrics::layout(text, self.font_size, text_rect.width) {
                    if selected.contains(&glyph.byte_offset) {
                        painter.fill_rect(
                            Rect {
                                x: text_rect.x + glyph.x,
                                y: text_rect.y + glyph.y,
                                width: glyph.advance,
                                height: glyph.row_height,
                            },
                            background,
                            0.0,
                        );
                    }
                }
            }
            painter.fill_text_selected(
                block_rect,
                text,
                color,
                self.selection_text_color.unwrap_or(color),
                selected,
                self.font_size,
                TextAlign::Start,
            );
        } else {
            painter.fill_text(block_rect, text, color, self.font_size, TextAlign::Start);
        }
    }
}

impl Widget for RawTextArea {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
        if let Some(color) = self.border_color.filter(|_| self.border_width > 0.0) {
            painter.stroke_rect(rect, color, self.border_width, self.corner_radius);
        }
        let padding = 12.0;
        let text_rect = Rect {
            x: rect.x + padding,
            y: rect.y + padding,
            width: (rect.width - padding * 2.0).max(0.0),
            height: (rect.height - padding * 2.0).max(0.0),
        };
        let (text, color) = if self.value.is_empty() && !self.placeholder.is_empty() {
            (&self.placeholder, self.placeholder_color)
        } else {
            (&self.value, self.text_color)
        };
        painter.push_clip(text_rect);
        if self.wrap {
            self.paint_wrapped(painter, text_rect, text, color);
        } else {
            self.paint_unwrapped(painter, text_rect, text, color);
        }
        painter.pop_clip();
    }

    fn focusable(&self) -> bool {
        true
    }
    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Text)
    }

    fn paint_focused_overlay(&self, painter: &mut dyn Painter, rect: Rect, caret_visible: bool) {
        if !caret_visible {
            return;
        }
        let padding = 12.0;
        let text_rect = Rect {
            x: rect.x + padding,
            y: rect.y + padding,
            width: (rect.width - padding * 2.0).max(0.0),
            height: (rect.height - padding * 2.0).max(0.0),
        };
        let cursor = self.cursor.min(self.value.len());
        // Caret proportions match `RawTextInput`'s: a slim bar sized and
        // vertically centered to the glyphs, not a full-height block.
        let (caret_x, caret_y, row_height) = if self.wrap {
            let glyphs = crate::text_metrics::layout(&self.value, self.font_size, text_rect.width);
            let fallback = crate::text_metrics::row_height(self.font_size);
            let (x, y, row_height) = crate::text_metrics::caret_xy(&glyphs, cursor, fallback);
            (text_rect.x + x, text_rect.y + y, row_height)
        } else {
            let before_cursor = &self.value[..cursor];
            let line = before_cursor.rsplit('\n').next().unwrap_or("");
            let (width, _) = crate::text_metrics::measure(
                line,
                self.font_size,
                crate::text_metrics::unbounded_width(),
            );
            let lines = (before_cursor.matches('\n').count()) as f32;
            let line_height = self.font_size * 1.4;
            let scroll_x = self.horizontal_scroll(text_rect.width);
            (
                text_rect.x + width - scroll_x,
                text_rect.y + lines * line_height,
                line_height,
            )
        };
        let caret_height = (self.font_size * 1.2).min(row_height);
        painter.push_clip(text_rect);
        painter.fill_rect(
            Rect {
                x: caret_x,
                y: caret_y + (row_height - caret_height) / 2.0,
                width: 1.5,
                height: caret_height,
            },
            self.text_color,
            0.0,
        );
        painter.pop_clip();
    }

    fn on_key(&self) -> Option<Rc<dyn Fn(KeyInput)>> {
        let value = self.value.clone();
        let on_change = self.on_change.clone();
        let cursor = self.cursor;
        let on_cursor_change = self.on_cursor_change.clone();
        let keyboard_selection = self.keyboard_selection.clone();
        let on_selection_change = self.on_selection_change.clone();
        let on_ctrl_o = self.on_ctrl_o.clone();
        let clipboard_enabled = self.clipboard_enabled;
        Some(Rc::new(move |input| {
            let selection = keyboard_selection.get();
            if clipboard_enabled && input.modifiers.ctrl {
                match input.key {
                    Key::Char('a') | Key::Char('A') => {
                        let all = TextSelection {
                            anchor: 0,
                            focus: value.len(),
                        };
                        keyboard_selection.set(all);
                        creamui_reactive::batch(|| {
                            on_cursor_change(value.len());
                            on_selection_change(all);
                        });
                        return;
                    }
                    Key::Char('c') | Key::Char('C') if !selection.is_empty() => {
                        clipboard_write(value[selection.range()].to_owned());
                        return;
                    }
                    Key::Char('x') | Key::Char('X') if !selection.is_empty() => {
                        let range = selection.range();
                        clipboard_write(value[range.clone()].to_owned());
                        let mut next = value.clone();
                        next.replace_range(range.clone(), "");
                        let next_cursor = range.start;
                        creamui_reactive::batch(|| {
                            on_change(next);
                            on_cursor_change(next_cursor);
                            on_selection_change(TextSelection {
                                anchor: next_cursor,
                                focus: next_cursor,
                            });
                        });
                        return;
                    }
                    Key::Char('v') | Key::Char('V') => {
                        if let Some(pasted) = clipboard_read() {
                            let mut next = value.clone();
                            let range = selection.range();
                            let next_cursor = if range.is_empty() {
                                next.insert_str(cursor.min(next.len()), &pasted);
                                cursor.min(value.len()) + pasted.len()
                            } else {
                                next.replace_range(range.clone(), &pasted);
                                range.start + pasted.len()
                            };
                            keyboard_selection.set(TextSelection {
                                anchor: next_cursor,
                                focus: next_cursor,
                            });
                            creamui_reactive::batch(|| {
                                on_change(next);
                                on_cursor_change(next_cursor);
                                on_selection_change(TextSelection {
                                    anchor: next_cursor,
                                    focus: next_cursor,
                                });
                            });
                        }
                        return;
                    }
                    _ => {}
                }
            }
            if input.modifiers.ctrl && matches!(input.key, Key::Char('o') | Key::Char('O')) {
                on_ctrl_o();
                return;
            }
            let mut next = value.clone();
            let mut next_cursor = cursor.min(next.len());
            let selected = selection.range();
            let mut edited = false;
            let mut replace_selection = |replacement: &str| {
                if !selected.is_empty() {
                    next.replace_range(selected.clone(), replacement);
                    next_cursor = selected.start + replacement.len();
                    edited = true;
                } else {
                    next.insert_str(next_cursor, replacement);
                    next_cursor += replacement.len();
                    edited = true;
                }
            };
            match input.key {
                Key::Char(c) => {
                    replace_selection(&c.to_string());
                }
                Key::Enter => {
                    replace_selection("\n");
                }
                Key::Backspace => {
                    if !selected.is_empty() {
                        next.replace_range(selected.clone(), "");
                        next_cursor = selected.start;
                        edited = true;
                    } else if let Some(previous) = next[..next_cursor]
                        .char_indices()
                        .last()
                        .map(|(index, _)| index)
                    {
                        next.drain(previous..next_cursor);
                        next_cursor = previous;
                        edited = true;
                    }
                }
                Key::Left => {
                    if let Some(previous) = next[..next_cursor]
                        .char_indices()
                        .last()
                        .map(|(index, _)| index)
                    {
                        next_cursor = previous;
                    }
                }
                Key::Right => {
                    if let Some(character) = next[next_cursor..].chars().next() {
                        next_cursor += character.len_utf8();
                    }
                }
                Key::Home => {
                    next_cursor = next[..next_cursor].rfind('\n').map_or(0, |index| index + 1);
                }
                Key::End => {
                    next_cursor = next[next_cursor..]
                        .find('\n')
                        .map_or(next.len(), |index| next_cursor + index);
                }
                Key::Up | Key::Down => {
                    let line_start = next[..next_cursor].rfind('\n').map_or(0, |index| index + 1);
                    let column = next[line_start..next_cursor].chars().count();
                    let lines: Vec<&str> = next.split('\n').collect();
                    let line = next[..next_cursor].matches('\n').count();
                    let target = if input.key == Key::Up {
                        line.checked_sub(1)
                    } else {
                        (line + 1 < lines.len()).then_some(line + 1)
                    };
                    if let Some(target) = target {
                        let start = lines
                            .iter()
                            .take(target)
                            .map(|line| line.len() + 1)
                            .sum::<usize>();
                        next_cursor = start
                            + lines[target]
                                .char_indices()
                                .nth(column)
                                .map_or(lines[target].len(), |(index, _)| index);
                    }
                }
                _ => return,
            }
            let extend = input.modifiers.shift
                && matches!(
                    input.key,
                    Key::Left | Key::Right | Key::Up | Key::Down | Key::Home | Key::End
                );
            let next_selection = if extend {
                TextSelection {
                    anchor: if selection.is_empty() {
                        cursor
                    } else {
                        selection.anchor
                    },
                    focus: next_cursor,
                }
            } else {
                TextSelection {
                    anchor: next_cursor,
                    focus: next_cursor,
                }
            };
            keyboard_selection.set(next_selection);
            creamui_reactive::batch(|| {
                if edited {
                    on_change(next);
                }
                on_cursor_change(next_cursor);
                on_selection_change(next_selection);
            });
        }))
    }

    fn on_drag_start(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        let value = self.value.clone();
        let font_size = self.font_size;
        let wrap = self.wrap;
        let on_cursor_change = self.on_cursor_change.clone();
        let on_selection_change = self.on_selection_change.clone();
        let drag_anchor = self.drag_anchor.clone();
        let drag_focus = self.drag_focus.clone();
        let keyboard_selection = self.keyboard_selection.clone();
        Some(Rc::new(move |point, rect: Rect| {
            let cursor = cursor_at_point(&value, font_size, point, wrap, rect.width - 24.0);
            drag_anchor.set(cursor);
            drag_focus.set(cursor);
            keyboard_selection.set(TextSelection {
                anchor: cursor,
                focus: cursor,
            });
            creamui_reactive::batch(|| {
                on_cursor_change(cursor);
                on_selection_change(TextSelection {
                    anchor: cursor,
                    focus: cursor,
                });
            });
        }))
    }

    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        let value = self.value.clone();
        let font_size = self.font_size;
        let wrap = self.wrap;
        let on_cursor_change = self.on_cursor_change.clone();
        let on_selection_change = self.on_selection_change.clone();
        let drag_anchor = self.drag_anchor.clone();
        let drag_focus = self.drag_focus.clone();
        let keyboard_selection = self.keyboard_selection.clone();
        Some(Rc::new(move |point, rect: Rect| {
            let cursor = cursor_at_point(&value, font_size, point, wrap, rect.width - 24.0);
            if cursor != drag_focus.get() {
                drag_focus.set(cursor);
                keyboard_selection.set(TextSelection {
                    anchor: drag_anchor.get(),
                    focus: cursor,
                });
                creamui_reactive::batch(|| {
                    on_cursor_change(cursor);
                    on_selection_change(TextSelection {
                        anchor: drag_anchor.get(),
                        focus: cursor,
                    });
                });
            }
        }))
    }
}

fn cursor_at_point(
    value: &str,
    font_size: f32,
    point: Point,
    wrap: bool,
    visible_width: f32,
) -> usize {
    if wrap {
        return crate::text_metrics::byte_offset_at_point(
            value,
            font_size,
            visible_width,
            point.x - 12.0,
            point.y - 12.0,
        );
    }
    let line = ((point.y - 12.0) / (font_size * 1.4)).floor().max(0.0) as usize;
    let lines: Vec<&str> = value.split('\n').collect();
    let line = line.min(lines.len().saturating_sub(1));
    let start = lines
        .iter()
        .take(line)
        .map(|line| line.len() + 1)
        .sum::<usize>();
    start + crate::text_metrics::byte_offset_at_x(lines[line], font_size, (point.x - 12.0).max(0.0))
}

impl RawTextInput {
    pub fn new(
        style: Style,
        value: impl Into<String>,
        font_size: f32,
        text_color: Color,
        on_change: impl Fn(String) + 'static,
    ) -> Self {
        let value = value.into();
        let cursor = value.len();
        RawTextInput {
            style,
            value,
            placeholder: String::new(),
            text_color,
            placeholder_color: text_color,
            background: None,
            border_color: None,
            border_width: 1.0,
            corner_radius: 0.0,
            font_size,
            cursor,
            selection: TextSelection { anchor: cursor, focus: cursor },
            selection_background: None,
            selection_text_color: None,
            on_change: Rc::new(on_change),
            on_cursor_change: Rc::new(|_| {}),
            on_selection_change: Rc::new(|_| {}),
            clipboard_enabled: true,
            keyboard_selection: Rc::new(Cell::new(TextSelection { anchor: cursor, focus: cursor })),
            drag_anchor: Rc::new(Cell::new(cursor)),
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>, color: Color) -> Self {
        self.placeholder = text.into();
        self.placeholder_color = color;
        self
    }

    /// Enables Ctrl/Cmd+V for this field. Text inputs expose the same opt-out
    /// surface as text areas; it is enabled by default.
    pub fn clipboard_enabled(mut self, enabled: bool) -> Self {
        self.clipboard_enabled = enabled;
        self
    }

    pub fn cursor(mut self, cursor: usize, on_change: impl Fn(usize) + 'static) -> Self {
        self.cursor = cursor.min(self.value.len());
        self.selection = TextSelection { anchor: self.cursor, focus: self.cursor };
        self.keyboard_selection.set(self.selection);
        self.on_cursor_change = Rc::new(on_change);
        self
    }

    pub fn selection(mut self, selection: TextSelection, on_change: impl Fn(TextSelection) + 'static) -> Self {
        self.selection = TextSelection { anchor: selection.anchor.min(self.value.len()), focus: selection.focus.min(self.value.len()) };
        self.keyboard_selection.set(self.selection);
        self.on_selection_change = Rc::new(on_change);
        self
    }

    pub fn selection_background(mut self, color: Color) -> Self { self.selection_background = Some(color); self }
    pub fn selection_text_color(mut self, color: Color) -> Self { self.selection_text_color = Some(color); self }

    /// How far to shift the value left so its end (editing is append-only)
    /// stays inside `visible_width` instead of running off the edge.
    fn horizontal_scroll(&self, visible_width: f32) -> f32 {
        let (text_width, _) = crate::text_metrics::measure(
            &self.value,
            self.font_size,
            crate::text_metrics::unbounded_width(),
        );
        (text_width - visible_width + 4.0).max(0.0)
    }
}

impl Widget for RawTextInput {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
        if let Some(color) = self.border_color {
            painter.stroke_rect(rect, color, self.border_width, self.corner_radius);
        }
        let padding = 8.0;
        let text_rect = Rect {
            x: rect.x + padding,
            y: rect.y,
            width: (rect.width - padding * 2.0).max(0.0),
            height: rect.height,
        };
        // Unwrapped: a bounded width here would let fontdue word-wrap onto a second row.
        let unbounded = Rect {
            x: text_rect.x - self.horizontal_scroll(text_rect.width),
            width: crate::text_metrics::unbounded_width(),
            ..text_rect
        };
        painter.push_clip(text_rect);
        if self.value.is_empty() {
            if !self.placeholder.is_empty() {
                painter.fill_text(
                    unbounded,
                    &self.placeholder,
                    self.placeholder_color,
                    self.font_size,
                    TextAlign::Start,
                );
            }
        } else {
            let selected = self.selection.range();
            if !selected.is_empty() {
                if let Some(background) = self.selection_background {
                    let (before, _) = crate::text_metrics::measure(&self.value[..selected.start], self.font_size, crate::text_metrics::unbounded_width());
                    let (width, _) = crate::text_metrics::measure(&self.value[selected.clone()], self.font_size, crate::text_metrics::unbounded_width());
                    painter.fill_rect(Rect { x: unbounded.x + before, y: rect.y + (rect.height - self.font_size * 1.4) / 2.0, width, height: self.font_size * 1.4 }, background, 2.0);
                }
            }
            painter.fill_text_selected(
                unbounded,
                &self.value,
                self.text_color,
                self.selection_text_color.unwrap_or(self.text_color),
                selected,
                self.font_size,
                TextAlign::Start,
            );
        }
        painter.pop_clip();
    }

    fn focusable(&self) -> bool {
        true
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Text)
    }

    fn paint_focused_overlay(&self, painter: &mut dyn Painter, rect: Rect, caret_visible: bool) {
        if !caret_visible {
            return;
        }
        let padding = 8.0;
        let visible_width = (rect.width - padding * 2.0).max(0.0);
        let cursor = self.cursor.min(self.value.len());
        let (text_width, _) = crate::text_metrics::measure(
            &self.value[..cursor],
            self.font_size,
            crate::text_metrics::unbounded_width(),
        );
        let text_width = if self.value.is_empty() {
            0.0
        } else {
            text_width
        };
        let caret_x = rect.x + padding + text_width - self.horizontal_scroll(visible_width);
        let caret_height = (self.font_size * 1.2).min(rect.height);
        let caret_rect = Rect {
            x: caret_x,
            y: rect.y + (rect.height - caret_height) / 2.0,
            width: 1.5,
            height: caret_height,
        };
        painter.fill_rect(caret_rect, self.text_color, 0.0);
    }

    fn on_key(&self) -> Option<Rc<dyn Fn(KeyInput)>> {
        let value = self.value.clone(); let on_change = self.on_change.clone();
        let cursor = self.cursor; let on_cursor_change = self.on_cursor_change.clone();
        let selection = self.keyboard_selection.clone(); let on_selection_change = self.on_selection_change.clone();
        let clipboard_enabled = self.clipboard_enabled;
        Some(Rc::new(move |input: KeyInput| {
            let selected = selection.get();
            if clipboard_enabled && input.modifiers.ctrl {
                match input.key {
                    Key::Char('a') | Key::Char('A') => { let all = TextSelection { anchor: 0, focus: value.len() }; selection.set(all); creamui_reactive::batch(|| { on_cursor_change(value.len()); on_selection_change(all); }); return; }
                    Key::Char('c') | Key::Char('C') if !selected.is_empty() => { clipboard_write(value[selected.range()].to_owned()); return; }
                    Key::Char('x') | Key::Char('X') if !selected.is_empty() => { let range = selected.range(); clipboard_write(value[range.clone()].to_owned()); let mut next = value.clone(); next.replace_range(range.clone(), ""); let at = range.start; let collapsed = TextSelection { anchor: at, focus: at }; selection.set(collapsed); creamui_reactive::batch(|| { on_change(next); on_cursor_change(at); on_selection_change(collapsed); }); return; }
                    Key::Char('v') | Key::Char('V') => { if let Some(paste) = clipboard_read() { let range = selected.range(); let mut next = value.clone(); let at = if range.is_empty() { cursor.min(next.len()) } else { range.start }; next.replace_range(if range.is_empty() { at..at } else { range }, &paste); let at = at + paste.len(); let collapsed = TextSelection { anchor: at, focus: at }; selection.set(collapsed); creamui_reactive::batch(|| { on_change(next); on_cursor_change(at); on_selection_change(collapsed); }); } return; }
                    _ => {}
                }
            }
            let mut next = value.clone(); let mut at = cursor.min(next.len()); let range = selected.range(); let mut changed = false;
            match input.key {
                Key::Char(c) => { next.replace_range(if range.is_empty() { at..at } else { range.clone() }, &c.to_string()); at = if range.is_empty() { at + c.len_utf8() } else { range.start + c.len_utf8() }; changed = true; }
                Key::Backspace => { if !range.is_empty() { next.replace_range(range.clone(), ""); at = range.start; changed = true; } else if let Some(previous) = next[..at].char_indices().last().map(|(i, _)| i) { next.replace_range(previous..at, ""); at = previous; changed = true; } }
                Key::Delete => { if !range.is_empty() { next.replace_range(range.clone(), ""); at = range.start; changed = true; } else if let Some(ch) = next[at..].chars().next() { next.replace_range(at..at + ch.len_utf8(), ""); changed = true; } }
                Key::Left => { if let Some(previous) = next[..at].char_indices().last().map(|(i, _)| i) { at = previous; } }
                Key::Right => { if let Some(ch) = next[at..].chars().next() { at += ch.len_utf8(); } }
                Key::Home => at = 0, Key::End => at = next.len(), _ => return,
            }
            let next_selection = if input.modifiers.shift && matches!(input.key, Key::Left | Key::Right | Key::Home | Key::End) { TextSelection { anchor: if selected.is_empty() { cursor } else { selected.anchor }, focus: at } } else { TextSelection { anchor: at, focus: at } };
            selection.set(next_selection); creamui_reactive::batch(|| { if changed { on_change(next); } on_cursor_change(at); on_selection_change(next_selection); });
        }))
    }

    fn on_drag_start(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        let value = self.value.clone(); let font_size = self.font_size;
        let on_cursor_change = self.on_cursor_change.clone(); let on_selection_change = self.on_selection_change.clone();
        let selection = self.keyboard_selection.clone(); let anchor = self.drag_anchor.clone();
        Some(Rc::new(move |point, rect| {
            let visible = (rect.width - 16.0).max(0.0);
            let (width, _) = crate::text_metrics::measure(&value, font_size, crate::text_metrics::unbounded_width());
            let scroll = (width - visible + 4.0).max(0.0);
            let cursor = crate::text_metrics::byte_offset_at_x(&value, font_size, (point.x - 8.0 + scroll).max(0.0));
            anchor.set(cursor); let next = TextSelection { anchor: cursor, focus: cursor }; selection.set(next);
            creamui_reactive::batch(|| { on_cursor_change(cursor); on_selection_change(next); });
        }))
    }

    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        let value = self.value.clone(); let font_size = self.font_size;
        let on_cursor_change = self.on_cursor_change.clone(); let on_selection_change = self.on_selection_change.clone();
        let selection = self.keyboard_selection.clone(); let anchor = self.drag_anchor.clone();
        Some(Rc::new(move |point, rect| {
            let visible = (rect.width - 16.0).max(0.0);
            let (width, _) = crate::text_metrics::measure(&value, font_size, crate::text_metrics::unbounded_width());
            let scroll = (width - visible + 4.0).max(0.0);
            let cursor = crate::text_metrics::byte_offset_at_x(&value, font_size, (point.x - 8.0 + scroll).max(0.0));
            let next = TextSelection { anchor: anchor.get(), focus: cursor }; selection.set(next);
            creamui_reactive::batch(|| { on_cursor_change(cursor); on_selection_change(next); });
        }))
    }
}

/// An unstyled checkbox: a fixed-size clickable box, filled when `checked`
/// and outlined otherwise. The caller owns the checked state (typically a
/// `bool` `Signal`) and toggles it from `on_click` — this widget has no
/// state of its own, same as every other widget (see `creamui_core::Widget`).
pub struct RawCheckbox {
    pub box_size: f32,
    pub checked: bool,
    pub fill_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
    pub on_click: Rc<dyn Fn()>,
}

impl RawCheckbox {
    pub fn new(
        box_size: f32,
        checked: bool,
        fill_color: Color,
        border_color: Color,
        on_click: impl Fn() + 'static,
    ) -> Self {
        RawCheckbox {
            box_size,
            checked,
            fill_color,
            border_color,
            border_width: 1.5,
            corner_radius: 0.0,
            on_click: Rc::new(on_click),
        }
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

impl Widget for RawCheckbox {
    fn style(&self) -> Style {
        Style {
            size: creamui_core::layout::Size {
                width: creamui_core::layout::Dimension::Length(self.box_size),
                height: creamui_core::layout::Dimension::Length(self.box_size),
            },
            ..Default::default()
        }
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if self.checked {
            painter.fill_rect(rect, self.fill_color, self.corner_radius);
        } else {
            painter.stroke_rect(
                rect,
                self.border_color,
                self.border_width,
                self.corner_radius,
            );
        }
    }

    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        Some(self.on_click.clone())
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }
}

/// An unstyled horizontal slider: drag (or click) anywhere along its track
/// to set a `0.0..=1.0` value. The caller owns the value (typically an
/// `f32` `Signal`) and updates it from `on_change` — same pattern as every
/// other interactive widget here.
pub struct RawSlider {
    pub style: Style,
    pub value: f32,
    pub track_color: Color,
    pub fill_color: Color,
    pub handle_color: Color,
    pub track_height: f32,
    pub handle_size: f32,
    pub on_change: Rc<dyn Fn(f32)>,
}

impl RawSlider {
    pub fn new(
        style: Style,
        value: f32,
        track_color: Color,
        fill_color: Color,
        handle_color: Color,
        on_change: impl Fn(f32) + 'static,
    ) -> Self {
        RawSlider {
            style,
            value: value.clamp(0.0, 1.0),
            track_color,
            fill_color,
            handle_color,
            track_height: 4.0,
            handle_size: 16.0,
            on_change: Rc::new(on_change),
        }
    }
}

impl Widget for RawSlider {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        let track_y = rect.y + (rect.height - self.track_height) / 2.0;
        let usable_width = (rect.width - self.handle_size).max(0.0);
        let handle_center_x = rect.x + self.handle_size / 2.0 + usable_width * self.value;

        painter.fill_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: rect.width,
                height: self.track_height,
            },
            self.track_color,
            self.track_height / 2.0,
        );
        painter.fill_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: (handle_center_x - rect.x).max(0.0),
                height: self.track_height,
            },
            self.fill_color,
            self.track_height / 2.0,
        );
        painter.fill_rect(
            Rect {
                x: handle_center_x - self.handle_size / 2.0,
                y: rect.y + (rect.height - self.handle_size) / 2.0,
                width: self.handle_size,
                height: self.handle_size,
            },
            self.handle_color,
            self.handle_size / 2.0,
        );
    }

    fn on_drag(&self) -> Option<Rc<dyn Fn(Point, Rect)>> {
        let on_change = self.on_change.clone();
        let handle_size = self.handle_size;
        Some(Rc::new(move |local: Point, rect: Rect| {
            let usable_width = (rect.width - handle_size).max(1.0);
            let fraction = ((local.x - handle_size / 2.0) / usable_width).clamp(0.0, 1.0);
            on_change(fraction);
        }))
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }
}

/// Which edge of a [`RawTab`] its active indicator bar is drawn on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabIndicatorSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// An unstyled horizontal tab bar: always lays out [`RawTab`] children
/// left-to-right. Like every other raw widget it owns only layout and
/// background — which tab is active and what a click does live in the
/// [`RawTab`] children and whatever `Signal` the caller wires them to. For a
/// vertical stack of nav entries, use [`RawSidebar`] instead.
pub struct RawTabs {
    pub style: Style,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub children: Vec<BoxedWidget>,
}

impl RawTabs {
    pub fn new(style: Style) -> Self {
        RawTabs {
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

    pub fn child(mut self, widget: BoxedWidget) -> Self {
        self.children.push(widget);
        self
    }

    pub fn with_children(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.children = widgets;
        self
    }
}

impl Widget for RawTabs {
    fn style(&self) -> Style {
        Style {
            display: creamui_core::layout::Display::Flex,
            flex_direction: creamui_core::layout::FlexDirection::Row,
            ..self.style.clone()
        }
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

/// An unstyled vertical nav rail: always lays out [`RawTab`] children
/// top-to-bottom. Same shape as [`RawTabs`] — owns only layout and
/// background — but for the "sidebar switches the visible view" pattern
/// instead of a horizontal tab bar.
pub struct RawSidebar {
    pub style: Style,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub children: Vec<BoxedWidget>,
}

impl RawSidebar {
    pub fn new(style: Style) -> Self {
        RawSidebar {
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

    pub fn child(mut self, widget: BoxedWidget) -> Self {
        self.children.push(widget);
        self
    }

    pub fn with_children(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.children = widgets;
        self
    }
}

impl Widget for RawSidebar {
    fn style(&self) -> Style {
        Style {
            display: creamui_core::layout::Display::Flex,
            flex_direction: creamui_core::layout::FlexDirection::Column,
            ..self.style.clone()
        }
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

/// An unstyled, controlled tab button, shared by [`RawTabs`] and
/// [`RawSidebar`]. `active` is supplied by the caller —
/// typically compared against a `Signal<usize>` holding the selected tab
/// index — so a tab list can be rebuilt reactively with no hidden widget
/// state, the same pattern as [`RawCheckbox`].
///
/// Paints only its `background` and, while `active`, a solid indicator bar
/// along one edge; everything else (label, icon, padding) comes from its
/// children, so a fully custom tab look needs no more than picking colors.
pub struct RawTab {
    pub style: Style,
    pub active: bool,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub indicator: Option<(TabIndicatorSide, Color, f32)>,
    pub children: Vec<BoxedWidget>,
    pub on_click: Rc<dyn Fn()>,
    pub on_hover: Option<Rc<dyn Fn(bool)>>,
}

impl RawTab {
    pub fn new(style: Style, active: bool, on_click: impl Fn() + 'static) -> Self {
        RawTab {
            style,
            active,
            background: None,
            corner_radius: 0.0,
            indicator: None,
            children: Vec::new(),
            on_click: Rc::new(on_click),
            on_hover: None,
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

    /// Draws a solid `thickness`-px bar along `side` while `active` is true.
    pub fn indicator(mut self, side: TabIndicatorSide, color: Color, thickness: f32) -> Self {
        self.indicator = Some((side, color, thickness));
        self
    }

    /// Registers a pointer enter/leave callback. The renderer invokes it
    /// with `true` on entry and `false` on exit.
    pub fn on_hover(mut self, callback: impl Fn(bool) + 'static) -> Self {
        self.on_hover = Some(Rc::new(callback));
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

impl Widget for RawTab {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
        if self.active {
            if let Some((side, color, thickness)) = self.indicator {
                // Capsule shape, inset from the tab's own edges.
                let inset = (thickness * 1.5).min(rect.width.min(rect.height) * 0.25);
                let bar = match side {
                    TabIndicatorSide::Left => Rect {
                        x: rect.x,
                        y: rect.y + inset,
                        width: thickness,
                        height: (rect.height - inset * 2.0).max(0.0),
                    },
                    TabIndicatorSide::Right => Rect {
                        x: rect.x + rect.width - thickness,
                        y: rect.y + inset,
                        width: thickness,
                        height: (rect.height - inset * 2.0).max(0.0),
                    },
                    TabIndicatorSide::Top => Rect {
                        x: rect.x + inset,
                        y: rect.y,
                        width: (rect.width - inset * 2.0).max(0.0),
                        height: thickness,
                    },
                    TabIndicatorSide::Bottom => Rect {
                        x: rect.x + inset,
                        y: rect.y + rect.height - thickness,
                        width: (rect.width - inset * 2.0).max(0.0),
                        height: thickness,
                    },
                };
                painter.fill_rect(bar, color, thickness / 2.0);
            }
        }
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        std::mem::take(&mut self.children)
    }

    fn on_click(&self) -> Option<Rc<dyn Fn()>> {
        Some(self.on_click.clone())
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }

    fn on_hover(&self) -> Option<Rc<dyn Fn(bool)>> {
        self.on_hover.clone()
    }
}

/// An unstyled vertically-scrollable container. The caller owns the scroll
/// offset (typically an `f32` `Signal`, clamped however makes sense for the
/// content) and updates it from `on_scroll` — same pattern as every other
/// interactive widget here. Children are laid out at their natural height
/// (never flex-shrunk to fit the visible viewport, which would defeat the
/// point of scrolling) and clipped + offset to this widget's own rect.
pub struct RawScrollView {
    pub style: Style,
    pub scroll_y: f32,
    pub background: Option<Color>,
    pub corner_radius: f32,
    pub children: Vec<BoxedWidget>,
    pub on_scroll: Rc<dyn Fn(f32)>,
}

impl RawScrollView {
    pub fn new(style: Style, scroll_y: f32, on_scroll: impl Fn(f32) + 'static) -> Self {
        RawScrollView {
            style,
            scroll_y,
            background: None,
            corner_radius: 0.0,
            children: Vec::new(),
            on_scroll: Rc::new(on_scroll),
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

    pub fn child(mut self, widget: BoxedWidget) -> Self {
        self.children.push(widget);
        self
    }

    pub fn with_children(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.children = widgets;
        self
    }
}

impl Widget for RawScrollView {
    fn style(&self) -> Style {
        // Always Column, regardless of what the caller passes: the sole
        // child is the content wrapper (see `children()` below), and it
        // needs Column's cross axis (width) to `align-items: stretch` to
        // the container's width while its main axis (height) stays
        // content-based — the combination that makes "as wide as the
        // viewport, as tall as the content" actually happen. Row direction
        // would stretch the wrapper's *height* instead, defeating scrolling
        // entirely (its children would get flex-shrunk to fit).
        Style {
            display: creamui_core::layout::Display::Flex,
            flex_direction: creamui_core::layout::FlexDirection::Column,
            ..self.style.clone()
        }
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        if let Some(color) = self.background {
            painter.fill_rect(rect, color, self.corner_radius);
        }
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        // Wrap the real children in a non-shrinking content column so they
        // keep their natural (possibly taller-than-viewport) height instead
        // of being flex-shrunk to fit — that overflow is exactly what makes
        // scrolling meaningful in the first place.
        let content_style = Style {
            display: creamui_core::layout::Display::Flex,
            flex_direction: creamui_core::layout::FlexDirection::Column,
            flex_shrink: 0.0,
            ..Default::default()
        };
        let content = RawView::new(content_style).with_children(std::mem::take(&mut self.children));
        vec![Box::new(content) as BoxedWidget]
    }

    fn clips_children(&self) -> bool {
        true
    }

    fn scroll_offset(&self) -> Point {
        Point {
            x: 0.0,
            y: self.scroll_y,
        }
    }

    fn on_scroll(&self) -> Option<Rc<dyn Fn(f32)>> {
        Some(self.on_scroll.clone())
    }
}
