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
    RawButton, RawCheckbox, RawScrollView, RawSidebar, RawSlider, RawTab, RawTabs, RawText,
    RawTextArea, RawTextInput, RawView, TabIndicatorSide, TextSelection,
};
pub use themed::{
    Button, Checkbox, Heading, MenuBar, MenuColors, MenuItem, MenuPopup, ScrollView, Sidebar,
    SidebarItem, Slider, Tab, TabColors, Tabs, Text, TextArea, TextInput, TextSize, View,
};
