//! Headless and themed widgets built on `creamui-core`.
//!
//! - [`raw`] contains fully unstyled ("headless") widgets like [`raw::RawButton`].
//! - [`themed`] contains styled wrappers like [`themed::Button`] that read
//!   their geometry from a [`creamui_theme::Theme`] and colours from its
//!   independently swappable [`creamui_theme::ColorScheme`].
//! - [`layout`] has convenience constructors for flex/grid layout styles.

mod components;
mod controller;
pub use components::{Choice, Icon, NavigationItem, Surface, SurfaceRole, Symbol};
pub mod layout;
pub mod raw;
mod text_metrics;
pub mod themed;

pub use controller::{ScrollController, TabController, TextController};
pub use raw::{
    RawButton, RawCheckbox, RawScrollView, RawSidebar, RawSlider, RawSpinner, RawSwitch, RawTab,
    RawTabs, RawText, RawTextArea, RawTextInput, RawView, TabIndicatorSide, TextSelection,
};
pub use themed::{
    tab_styles, Button, ButtonSize, ButtonState, ButtonVariant, Checkbox, Heading, MenuBar,
    MenuColors, MenuItem, MenuPopup, ScrollView, Sidebar, SidebarItem, SidebarSeparator, Slider,
    Spinner, Switch, Tab, TabColors, TabSizing, Tabs, Text, TextArea, TextInput, TextSize, View,
};
