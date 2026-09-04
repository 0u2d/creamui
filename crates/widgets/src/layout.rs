//! Convenience constructors for common HTML/CSS-like layout patterns, built
//! directly on `taffy`'s flex and grid styles (re-exported through
//! `creamui_core::layout`) rather than a custom layout engine.

use creamui_core::layout::{
    AlignItems, Dimension, FlexDirection, GridPlacement, LengthPercentage,
    NonRepeatedTrackSizingFunction, Style, TaffyGridLine, TrackSizingFunction,
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
        assert_eq!(b_layout.location.x, 15.0, "second child should start after first child (10px) + gap (5px)");
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
        assert_eq!(b_layout.location.x, 100.0, "second column should start at half the 200px grid width");
    }
}
