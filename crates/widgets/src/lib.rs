//! Headless and themed widgets built on `creamui-core`.
//!
//! - [`raw`] contains fully unstyled ("headless") widgets like [`raw::RawButton`].
//! - [`themed`] contains styled wrappers like [`themed::Button`] that read
//!   their appearance from a [`creamui_theme::Theme`].
//! - [`layout`] has convenience constructors for flex/grid layout styles.

pub mod layout;
pub mod raw;
mod text_metrics;
pub mod themed;

pub use raw::{
    RawButton, RawCheckbox, RawScrollView, RawSlider, RawText, RawTextArea, RawTextInput, RawView,
    TextSelection,
};
pub use themed::{
    Button, Checkbox, MenuBar, MenuColors, MenuItem, MenuPopup, ScrollView, Slider, Text, TextArea,
    TextInput, View,
};
