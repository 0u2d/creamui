//! Convenience constructors for common HTML/CSS-like layout patterns, built
//! directly on `taffy`'s flex and grid styles (re-exported through
//! `creamui_core::layout`) rather than a custom layout engine.

use creamui_core::layout::{
    AlignItems, Dimension, FlexDirection, GridPlacement, JustifyContent, LengthPercentage,
    LengthPercentageAuto, NonRepeatedTrackSizingFunction, Style, TaffyGridLine,
    TrackSizingFunction,
};

/// A flex row: children laid out left-to-right with a fixed pixel `gap`.
pub fn row(gap: f32) -> Style {
    Style {
        display: creamui_core::layout::Display::Flex,
        flex_direction: FlexDirection::Row,
        gap: creamui_core::layout::Size {
            width: LengthPercentage::Length(gap),
            height: LengthPercentage::Length(gap),
        },
        align_items: Some(AlignItems::Center),
        ..Default::default()
    }
}

/// A flex column: children laid out top-to-bottom with a fixed pixel `gap`.
pub fn column(gap: f32) -> Style {
    Style {
        display: creamui_core::layout::Display::Flex,
        flex_direction: FlexDirection::Column,
        gap: creamui_core::layout::Size {
            width: LengthPercentage::Length(gap),
            height: LengthPercentage::Length(gap),
        },
        ..Default::default()
    }
}

/// A CSS-grid-like layout with `columns` equal-width tracks and a fixed pixel `gap`.
pub fn grid(columns: usize, gap: f32) -> Style {
    let track: NonRepeatedTrackSizingFunction = creamui_core::layout::fr(1.0f32);
    Style {
        display: creamui_core::layout::Display::Grid,
        grid_template_columns: vec![TrackSizingFunction::Single(track); columns],
        gap: creamui_core::layout::Size {
            width: LengthPercentage::Length(gap),
            height: LengthPercentage::Length(gap),
        },
        ..Default::default()
    }
}

/// Places a grid item at an explicit 1-indexed `(column, row)` cell.
pub fn grid_cell(column: i16, row: i16) -> Style {
    Style {
        grid_column: creamui_core::layout::Line {
            start: GridPlacement::from_line_index(column),
            end: GridPlacement::Auto,
        },
        grid_row: creamui_core::layout::Line {
            start: GridPlacement::from_line_index(row),
            end: GridPlacement::Auto,
        },
        ..Default::default()
    }
}

/// A fixed pixel size for `Style::size`.
pub fn fixed(width: f32, height: f32) -> creamui_core::layout::Size<Dimension> {
    creamui_core::layout::Size {
        width: Dimension::Length(width),
        height: Dimension::Length(height),
    }
}

/// Makes a style fill the available space in its parent.
pub fn fill(mut style: Style) -> Style {
    style.size = creamui_core::layout::Size {
        width: Dimension::Percent(1.0),
        height: Dimension::Percent(1.0),
    };
    style
}

/// Stretches a style to its parent's full width while leaving height alone —
/// e.g. a search field or button that should span a sidebar or toolbar
/// instead of keeping a fixed-width default like [`crate::TextInput`]'s
/// hardcoded 200px. [`fill`] stretches both axes, which isn't what you want
/// when only the cross axis should grow.
pub fn full_width(mut style: Style) -> Style {
    style.size.width = Dimension::Percent(1.0);
    style
}

/// Centers children on both flex axes while preserving the rest of `style`.
pub fn centered(mut style: Style) -> Style {
    style.justify_content = Some(JustifyContent::Center);
    style.align_items = Some(AlignItems::Center);
    style
}

/// Applies equal inner spacing, equivalent to CSS `padding: value`.
pub fn padding(mut style: Style, value: f32) -> Style {
    style.padding = edges(value, value, value, value);
    style
}

/// Applies horizontal and vertical inner spacing, equivalent to CSS
/// `padding: vertical horizontal`.
pub fn padding_xy(mut style: Style, horizontal: f32, vertical: f32) -> Style {
    style.padding = edges(horizontal, horizontal, vertical, vertical);
    style
}

/// Applies equal outer spacing, equivalent to CSS `margin: value`.
pub fn margin(mut style: Style, value: f32) -> Style {
    style.margin = auto_edges(value, value, value, value);
    style
}

/// Applies horizontal and vertical outer spacing, equivalent to CSS
/// `margin: vertical horizontal`.
pub fn margin_xy(mut style: Style, horizontal: f32, vertical: f32) -> Style {
    style.margin = auto_edges(horizontal, horizontal, vertical, vertical);
    style
}

fn edges(
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> creamui_core::layout::Rect<LengthPercentage> {
    creamui_core::layout::Rect {
        left: LengthPercentage::Length(left),
        right: LengthPercentage::Length(right),
        top: LengthPercentage::Length(top),
        bottom: LengthPercentage::Length(bottom),
    }
}

fn auto_edges(
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> creamui_core::layout::Rect<LengthPercentageAuto> {
    creamui_core::layout::Rect {
        left: LengthPercentageAuto::Length(left),
        right: LengthPercentageAuto::Length(right),
        top: LengthPercentageAuto::Length(top),
        bottom: LengthPercentageAuto::Length(bottom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creamui_core::layout::AvailableSpace;

    #[test]
    fn row_lays_out_children_left_to_right() {
        let mut tree = creamui_core::layout::TaffyTree::<()>::new();
        let child_style = Style {
            size: fixed(10.0, 10.0),
            ..Default::default()
        };
        let a = tree.new_leaf(child_style.clone()).unwrap();
        let b = tree.new_leaf(child_style).unwrap();
        let root = tree.new_with_children(row(5.0), &[a, b]).unwrap();
        tree.compute_layout(
            root,
            creamui_core::layout::Size {
                width: AvailableSpace::Definite(200.0),
                height: AvailableSpace::Definite(200.0),
            },
        )
        .unwrap();

        let a_layout = tree.layout(a).unwrap();
        let b_layout = tree.layout(b).unwrap();
        assert_eq!(a_layout.location.x, 0.0);
        assert_eq!(
            b_layout.location.x, 15.0,
            "second child should start after first child (10px) + gap (5px)"
        );
    }

    #[test]
    fn grid_places_children_in_columns() {
        let mut tree = creamui_core::layout::TaffyTree::<()>::new();
        let child_style = Style {
            size: fixed(50.0, 50.0),
            ..Default::default()
        };
        let a = tree.new_leaf(child_style.clone()).unwrap();
        let b = tree.new_leaf(child_style).unwrap();
        let root_style = Style {
            size: fixed(200.0, 200.0),
            ..grid(2, 0.0)
        };
        let root = tree.new_with_children(root_style, &[a, b]).unwrap();
        tree.compute_layout(
            root,
            creamui_core::layout::Size {
                width: AvailableSpace::Definite(200.0),
                height: AvailableSpace::Definite(200.0),
            },
        )
        .unwrap();

        let a_layout = tree.layout(a).unwrap();
        let b_layout = tree.layout(b).unwrap();
        assert_eq!(a_layout.location.x, 0.0);
        assert_eq!(
            b_layout.location.x, 100.0,
            "second column should start at half the 200px grid width"
        );
    }
}
