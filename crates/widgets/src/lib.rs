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

pub use controller::{
    ColorPickerController, DateTimeController, ScrollController, SelectController, TabController,
    TextController, TreeController,
};
pub use raw::{
    DateTime, RawButton, RawCheckbox, RawColorPicker, RawDateTimePicker, RawFilePicker,
    RawListView, RawScrollView, RawScrollbar, RawSidebar, RawSlider, RawSpinner, RawSwitch, RawTab,
    RawTable, RawTabs, RawText, RawTextArea, RawTextInput, RawView, TabIndicatorSide, TableColumn,
    TextSelection,
};
pub use themed::{
    tab_styles, AlertDialog, Button, ButtonSize, ButtonState, ButtonVariant, Checkbox, ColorPicker,
    ComboBox, DateInput, DateTimePicker, Dialog, FilePicker, Heading, ListBox, ListView, MenuBar,
    MenuColors, MenuItem, MenuPopup, Overlay, Popover, ProgressBar, ProgressRing, Radio,
    RadioGroup, ScrollView, SegmentedControl, Select, Sidebar, SidebarItem, SidebarSeparator,
    Slider, Spinner, Switch, Tab, TabColors, TabSizing, Table, Tabs, Text, TextArea, TextInput,
    TextSize, TimeInput, TreeNode, TreeView, View,
};
