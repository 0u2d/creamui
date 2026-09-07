//! Runtime component protocol used by CreamUI JSX.

use creamui_core::{BoxedWidget, Widget};

/// Converts native widgets and component-function results into a child node.
pub trait IntoWidget {
    fn into_widget(self) -> BoxedWidget;
}

impl<T: Widget + 'static> IntoWidget for T {
    fn into_widget(self) -> BoxedWidget {
        Box::new(self)
    }
}

impl IntoWidget for BoxedWidget {
    fn into_widget(self) -> BoxedWidget {
        self
    }
}
