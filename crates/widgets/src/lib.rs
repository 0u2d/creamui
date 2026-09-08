//! Headless and themed widgets built on `creamui-core`.
//!
//! - [`raw`] contains fully unstyled ("headless") widgets like [`raw::RawButton`].
//! - [`themed`] contains styled wrappers like [`themed::Button`] that read
//!   their geometry from a [`creamui_theme::Theme`] and colours from its
//!   independently swappable [`creamui_theme::ColorScheme`].
//! - [`layout`] has convenience constructors for flex/grid layout styles.

mod controller;
pub mod layout;
pub mod raw;
mod text_metrics;
pub mod themed;

pub use controller::TextController;
pub use raw::{
    RawButton, RawCheckbox, RawScrollView, RawSidebar, RawSlider, RawTab, RawTabs, RawText,
    RawSpinner, RawSwitch, RawTextArea, RawTextInput, RawView, TabIndicatorSide, TextSelection,
};
pub use themed::{
    Button, ButtonSize, ButtonState, ButtonVariant, Checkbox, Heading, MenuBar, MenuColors, MenuItem, MenuPopup, ScrollView, Sidebar,
    SidebarItem, SidebarSeparator, Slider, Spinner, Switch, Tab, TabColors, Tabs, Text, TextArea, TextInput,
    TextSize, View,
};
