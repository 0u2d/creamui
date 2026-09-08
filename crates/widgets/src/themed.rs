//! Themed widgets: opinionated, styled wrappers around the headless widgets
//! in [`crate::raw`]. Each one reads its appearance from a
//! [`creamui_theme::Theme`] passed in at construction time — either a fixed
//! value, or `creamui_theme::ThemeProvider::get()`'s result each render, to
//! support runtime theme switching.
//!
//! These are meant to be copied and adapted: a themed `Button` is nothing
//! more than a [`crate::raw::RawButton`] with theme-derived style baked in,
//! so writing a derived component (e.g. a `DangerButton`) is just writing a
//! new constructor function in the same shape.

use crate::raw::{
    RawButton, RawCheckbox, RawListView, RawScrollView, RawSidebar, RawSlider, RawSpinner,
    RawSwitch, RawTab, RawTable, RawTabs, RawText, RawTextArea, RawTextInput, RawView,
    TabIndicatorSide, TableColumn,
};
use creamui_core::layout::{
    AlignItems, Dimension, JustifyContent, LengthPercentage, Rect as LayoutRect, Style,
};
use creamui_core::{BoxedWidget, CursorIcon, KeyInput, Painter, Point, Rect, TextAlign, Widget};
use creamui_theme::{Color, SelectionStyle, Theme};
use std::rc::Rc;

fn centered_box_style(padding: f32) -> Style {
    Style {
        padding: LayoutRect {
            left: LengthPercentage::Length(padding),
            right: LengthPercentage::Length(padding),
            top: LengthPercentage::Length(padding * 0.6),
            bottom: LengthPercentage::Length(padding * 0.6),
        },
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
        ..Default::default()
    }
}

mod button;
mod controls;
mod dataview;
mod feedback;
mod inputs;
mod navigation;
mod scroll;
mod selection;
mod surfaces;
mod text;

pub use button::*;
pub use controls::*;
pub use dataview::*;
pub use feedback::*;
pub use inputs::*;
pub use navigation::*;
pub use scroll::*;
pub use selection::*;
pub use surfaces::*;
pub use text::*;
