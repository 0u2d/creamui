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
}

impl RawButton {
    pub fn new(style: Style, on_click: impl Fn() + 'static) -> Self {
        RawButton {
            style,
            background: None,
            corner_radius: 0.0,
            children: Vec::new(),
            on_click: Rc::new(on_click),
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
        Some(self.on_click.clone())
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
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
    pub on_change: Rc<dyn Fn(String)>,
    pub clipboard_enabled: bool,
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

    /// Highlights the source line containing the caret. CreamUI's current
    /// textarea caret is append-only, so this is the final source line.
    pub fn active_line_background(mut self, color: Color) -> Self {
        self.active_line_background = Some(color);
        self
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
        // `Painter::fill_text` vertically centers a text run. A textarea
        // needs a stable baseline per source line, not one centered block.
        // Draw each line in its own line-height box and clip overflowing
        // document content to the editor's inner padding box.
        let line_height = self.font_size * 1.4;
        let active_line = self.value[..self.cursor.min(self.value.len())]
            .matches('\n')
            .count();
        painter.push_clip(text_rect);
        let selected = self.selection.range();
        let mut source_offset = 0;
        for (index, line) in text.split('\n').enumerate() {
            let line_rect = Rect {
                y: text_rect.y + index as f32 * line_height,
                height: line_height,
                ..text_rect
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
                                x: line_rect.x + x,
                                width,
                                ..line_rect
                            },
                            background,
                            2.0,
                        );
                    }
                    painter.fill_text_selected(
                        line_rect,
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
            painter.fill_text(line_rect, line, color, self.font_size, TextAlign::Start);
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
        let cursor = self.cursor.min(self.value.len());
        let before_cursor = &self.value[..cursor];
        let line = before_cursor.rsplit('\n').next().unwrap_or("");
        let (width, _) = crate::text_metrics::measure(
            line,
            self.font_size,
            crate::text_metrics::unbounded_width(),
        );
        let lines = (before_cursor.matches('\n').count() + 1) as f32;
        let line_height = self.font_size * 1.4;
        painter.fill_rect(
            Rect {
                x: (rect.x + padding + width).min(rect.x + rect.width - 1.0),
                y: rect.y + padding + (lines - 1.0) * line_height,
                width: 1.5,
                height: line_height.min((rect.height - padding * 2.0).max(0.0)),
            },
            self.text_color,
            0.0,
        );
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
        let on_cursor_change = self.on_cursor_change.clone();
        let on_selection_change = self.on_selection_change.clone();
        let drag_anchor = self.drag_anchor.clone();
        let drag_focus = self.drag_focus.clone();
        let keyboard_selection = self.keyboard_selection.clone();
        Some(Rc::new(move |point, _| {
            let cursor = cursor_at_point(&value, font_size, point);
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
        let on_cursor_change = self.on_cursor_change.clone();
        let on_selection_change = self.on_selection_change.clone();
        let drag_anchor = self.drag_anchor.clone();
        let drag_focus = self.drag_focus.clone();
        let keyboard_selection = self.keyboard_selection.clone();
        Some(Rc::new(move |point, _| {
            let cursor = cursor_at_point(&value, font_size, point);
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

fn cursor_at_point(value: &str, font_size: f32, point: Point) -> usize {
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
        RawTextInput {
            style,
            value: value.into(),
            placeholder: String::new(),
            text_color,
            placeholder_color: text_color,
            background: None,
            border_color: None,
            border_width: 1.0,
            corner_radius: 0.0,
            font_size,
            on_change: Rc::new(on_change),
            clipboard_enabled: true,
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
        if self.value.is_empty() {
            if !self.placeholder.is_empty() {
                painter.fill_text(
                    text_rect,
                    &self.placeholder,
                    self.placeholder_color,
                    self.font_size,
                    TextAlign::Start,
                );
            }
        } else {
            painter.fill_text(
                text_rect,
                &self.value,
                self.text_color,
                self.font_size,
                TextAlign::Start,
            );
        }
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
        let (text_width, _) = crate::text_metrics::measure(
            &self.value,
            self.font_size,
            crate::text_metrics::unbounded_width(),
        );
        let text_width = if self.value.is_empty() {
            0.0
        } else {
            text_width
        };
        let caret_x = (rect.x + padding + text_width)
            .min(rect.x + rect.width - 1.0)
            .max(rect.x);
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
        let value = self.value.clone();
        let on_change = self.on_change.clone();
        let clipboard_enabled = self.clipboard_enabled;
        Some(Rc::new(move |input: KeyInput| {
            let mut next = value.clone();
            if clipboard_enabled
                && input.modifiers.ctrl
                && matches!(input.key, Key::Char('v') | Key::Char('V'))
            {
                if let Some(pasted) = clipboard_read() {
                    next.push_str(&pasted);
                    on_change(next);
                }
                return;
            }
            match input.key {
                Key::Char(c) => next.push(c),
                Key::Backspace => {
                    next.pop();
                }
                _ => return,
            }
            on_change(next);
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
