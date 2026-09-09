//! Single import for every panel/helper module: the widgets, layout, and
//! reactive types the showcase uses, plus the shared helpers in `common`
//! and `nav`. Each file under `panels/` starts with `use crate::prelude::*;`
//! instead of hand-picking imports.

pub use creamui_core::layout::{AlignItems, Dimension, JustifyContent, Style};
pub use creamui_core::{BoxedWidget, Size, TextAlign};
pub use creamui_image::{Image, ImageData, ImageFit};
pub use creamui_macros::{component, jsx};
pub use creamui_reactive::{create_effect, Effect, Signal};
pub use creamui_render::{run, WindowHandle, WindowOptions};
pub use creamui_theme::{use_theme, Color, SelectionStyle, Theme};
pub use creamui_widgets::layout::{column, fixed, padding, row};
pub use creamui_widgets::{
    tab_styles, AlertDialog, Button, ButtonSize, ButtonState, Choice, ColorPicker,
    ColorPickerController, DateTime, DateTimeController, DateTimePicker, FilePicker, Heading, Icon,
    Link, ListBox, NavigationItem, Popover, Pre, ProgressBar, ProgressRing, Quote, RadioGroup,
    RawScrollView, RawText, RawView, ScrollController, ScrollView, SegmentedControl, Select,
    SelectController, Sidebar, SidebarItem, Surface, SurfaceRole, Switch, Symbol, Tab, TabColors,
    TabController, TabSizing, Table, TableColumn, Tabs, Text, TextController, TextInput, TextSize,
    TreeController, TreeNode, TreeView, View,
};
pub use std::cell::RefCell;
pub use std::rc::Rc;

pub use crate::common::*;
pub use crate::nav::*;
