//! CSS-like layout primitives built on `taffy`.
//!
//! [`Flex`] is the semantic, unstyled equivalent of a `<div
//! style="display: flex">`: use it when the node is a layout container.
//! [`StyleExt`] provides the same vocabulary for an existing widget's
//! [`Style`]. The low-level Taffy types remain available through
//! `creamui_core::layout` for cases that need them.
//!
//! Every container here defaults to `min-size: 0` on both axes instead of
//! Taffy/CSS's own default (`min-size: auto`, i.e. "never shrink below your
//! content's intrinsic size"). That default is the classic flexbox/grid
//! trap: a child that's supposed to shrink to fit a tight parent instead
//! refuses to, and overflows past it instead. [`Block::min_size`],
//! [`Flex::min_size`], [`Grid::min_size`] and friends opt back into the
//! CSS default on a case-by-case basis when a container genuinely should
//! never shrink below its content.

use creamui_core::layout::{
    AlignContent, AlignItems, Dimension, Display, FlexDirection, FlexWrap, GridAutoFlow,
    GridPlacement, GridTrackRepetition, JustifyContent, LengthPercentage, LengthPercentageAuto,
    NonRepeatedTrackSizingFunction, Style, TaffyGridLine, TrackSizingFunction,
};
use creamui_core::{BoxedWidget, Painter, Rect, Widget};
use creamui_theme::Color;

/// Alignment on a flex container's cross axis (`align-items`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

impl From<Align> for AlignItems {
    fn from(value: Align) -> Self {
        match value {
            Align::Start => Self::Start,
            Align::Center => Self::Center,
            Align::End => Self::End,
            Align::Stretch => Self::Stretch,
            Align::Baseline => Self::Baseline,
        }
    }
}

/// Distribution on a flex container's main axis (`justify-content`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Justify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

impl From<Justify> for JustifyContent {
    fn from(value: Justify) -> Self {
        match value {
            Justify::Start => Self::Start,
            Justify::Center => Self::Center,
            Justify::End => Self::End,
            Justify::Between => Self::SpaceBetween,
            Justify::Around => Self::SpaceAround,
            Justify::Evenly => Self::SpaceEvenly,
        }
    }
}

/// Whether flex items can flow onto another line (`flex-wrap`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Wrap {
    #[default]
    NoWrap,
    Wrap,
    Reverse,
}

impl From<Wrap> for FlexWrap {
    fn from(value: Wrap) -> Self {
        match value {
            Wrap::NoWrap => Self::NoWrap,
            Wrap::Wrap => Self::Wrap,
            Wrap::Reverse => Self::WrapReverse,
        }
    }
}

/// Fills in `min-size: 0` on whichever axes are still at Taffy's `Auto`
/// default, without touching an axis the caller already set explicitly.
///
/// This is what makes [`Block`], [`Flex`] and [`Grid`] shrinkable by
/// default: without it, a flex/grid child never shrinks below its content's
/// intrinsic size (CSS's "min-width/height: auto" floor), so a tight parent
/// just overflows instead of the child wrapping, eliding, or scrolling.
pub(crate) fn shrinkable(mut style: Style) -> Style {
    if style.min_size.width == Dimension::Auto {
        style.min_size.width = Dimension::Length(0.0);
    }
    if style.min_size.height == Dimension::Auto {
        style.min_size.height = Dimension::Length(0.0);
    }
    style
}

/// Generates the builder methods shared by every container that wraps a
/// `RawView` in an `inner` field ([`Block`], [`Flex`], [`Grid`]) — kept as a
/// macro rather than a shared base type because these are consuming
/// (`self -> Self`) builders returning the concrete container type, which a
/// trait or inheritance hierarchy can't express without losing the type
/// (`Block::new().size(..)` needs to stay a `Block`, not a boxed trait
/// object). Each container still owns its layout-specific methods
/// (`Flex::gap`, `Grid::columns`, ...) directly.
macro_rules! layout_container_methods {
    () => {
        /// Sets a fixed pixel size.
        pub fn size(mut self, width: f32, height: f32) -> Self {
            self.inner.style = self.inner.style.size(width, height);
            self
        }

        /// Applies equal inner spacing.
        pub fn padding(mut self, value: f32) -> Self {
            self.inner.style = self.inner.style.padding_all(value);
            self
        }

        /// Applies CSS-like `padding: vertical horizontal` spacing.
        pub fn padding_xy(mut self, horizontal: f32, vertical: f32) -> Self {
            self.inner.style = self.inner.style.padding_xy(horizontal, vertical);
            self
        }

        /// Applies equal outer spacing.
        pub fn margin(mut self, value: f32) -> Self {
            self.inner.style = self.inner.style.margin_all(value);
            self
        }

        /// Applies CSS-like `margin: vertical horizontal` spacing.
        pub fn margin_xy(mut self, horizontal: f32, vertical: f32) -> Self {
            self.inner.style = self.inner.style.margin_xy(horizontal, vertical);
            self
        }

        /// Sets a fixed pixel width, leaving height alone — e.g. combine
        /// with `full_width` for the opposite axis, or with a parent that
        /// stretches this container's height for you.
        pub fn width(mut self, value: f32) -> Self {
            self.inner.style.size.width = Dimension::Length(value);
            self
        }

        /// Sets a fixed pixel height, leaving width alone; see `width`.
        pub fn height(mut self, value: f32) -> Self {
            self.inner.style.size.height = Dimension::Length(value);
            self
        }

        /// Stretches the container across its parent's width.
        pub fn full_width(mut self) -> Self {
            self.inner.style = full_width(self.inner.style);
            self
        }

        /// Stretches the container across its parent's height.
        pub fn full_height(mut self) -> Self {
            self.inner.style.size.height = Dimension::Percent(1.0);
            self
        }

        /// Stretches the container to the available size on both axes.
        pub fn fill(mut self) -> Self {
            self.inner.style = fill(self.inner.style);
            self
        }

        /// Makes this container take remaining space in a flex parent.
        pub fn grow(mut self, factor: f32) -> Self {
            self.inner.style = self.inner.style.grow(factor);
            self
        }

        /// Sets how readily this container shrinks in a flex parent.
        pub fn shrink(mut self, factor: f32) -> Self {
            self.inner.style = self.inner.style.shrink(factor);
            self
        }

        /// Sets this container's initial main-axis size in a flex parent.
        pub fn basis(mut self, value: f32) -> Self {
            self.inner.style = self.inner.style.basis(value);
            self
        }

        /// Overrides the parent's `align-items` for this container.
        pub fn align_self(mut self, value: Align) -> Self {
            self.inner.style = self.inner.style.align_self(value);
            self
        }

        /// Sets a minimum pixel size, opting back into the "never shrink
        /// below this size" behavior on both axes — this container
        /// otherwise shrinks to `0` by default (see the module docs).
        pub fn min_size(mut self, width: f32, height: f32) -> Self {
            self.inner.style.min_size = fixed(width, height);
            self
        }

        /// Sets a minimum pixel width; see `min_size`.
        pub fn min_width(mut self, value: f32) -> Self {
            self.inner.style.min_size.width = Dimension::Length(value);
            self
        }

        /// Sets a minimum pixel height; see `min_size`.
        pub fn min_height(mut self, value: f32) -> Self {
            self.inner.style.min_size.height = Dimension::Length(value);
            self
        }

        /// Gives the container a background without introducing a themed
        /// surface.
        pub fn background(mut self, color: Color) -> Self {
            self.inner = self.inner.background(color);
            self
        }

        /// Rounds the optional background's corners.
        pub fn corner_radius(mut self, radius: f32) -> Self {
            self.inner = self.inner.corner_radius(radius);
            self
        }

        /// Adds a child widget.
        pub fn child(mut self, child: BoxedWidget) -> Self {
            self.inner = self.inner.child(child);
            self
        }

        /// Adds all child widgets at once.
        pub fn with_children(mut self, children: Vec<BoxedWidget>) -> Self {
            self.inner = self.inner.with_children(children);
            self
        }
    };
}

/// An unstyled block container, equivalent to a semantic HTML `<div>`.
///
/// `Block` is intentionally the base container. Use [`Flex`] only when its
/// children need flexbox behavior; the name `View` is kept free for a future
/// navigation/view abstraction.
pub struct Block {
    inner: crate::raw::RawView,
}

impl Block {
    /// Creates an empty block-layout container.
    pub fn new() -> Self {
        Self::with_style(Style::default())
    }

    /// Creates a block container from a Taffy style. `display` is always
    /// normalized to `Block` so the component keeps its semantic contract.
    pub fn with_style(mut style: Style) -> Self {
        style.display = Display::Block;
        Self {
            inner: crate::raw::RawView::new(shrinkable(style)),
        }
    }

    layout_container_methods!();
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Block {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }
}

/// An unstyled flex container, equivalent to `<div style="display: flex">`.
///
/// ```
/// use creamui_widgets::layout::{Align, Flex, Justify};
///
/// let toolbar = Flex::row()
///     .gap(12.0)
///     .align(Align::Center)
///     .justify(Justify::Between);
/// ```
pub struct Flex {
    inner: crate::raw::RawView,
}

impl Flex {
    /// Creates a left-to-right flex container. Its defaults match CSS:
    /// `align-items` and `justify-content` are unset, and wrapping is off.
    pub fn row() -> Self {
        Self::new(FlexDirection::Row)
    }

    /// Creates a top-to-bottom flex container.
    pub fn column() -> Self {
        Self::new(FlexDirection::Column)
    }

    /// Creates a flex container with an explicit direction.
    pub fn new(direction: FlexDirection) -> Self {
        Self {
            inner: crate::raw::RawView::new(shrinkable(Style {
                display: Display::Flex,
                flex_direction: direction,
                ..Default::default()
            })),
        }
    }

    /// Changes the flex direction, including reverse directions when needed.
    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.inner.style.flex_direction = direction;
        self
    }

    /// Sets equal row and column gaps, equivalent to CSS `gap`.
    pub fn gap(mut self, value: f32) -> Self {
        self.inner.style = self.inner.style.gap(value);
        self
    }

    /// Sets the horizontal gap (`column-gap`).
    pub fn gap_x(mut self, value: f32) -> Self {
        self.inner.style = self.inner.style.gap_x(value);
        self
    }

    /// Sets the vertical gap (`row-gap`).
    pub fn gap_y(mut self, value: f32) -> Self {
        self.inner.style = self.inner.style.gap_y(value);
        self
    }

    /// Sets `align-items` on the cross axis.
    pub fn align(mut self, value: Align) -> Self {
        self.inner.style = self.inner.style.align(value);
        self
    }

    /// Sets `justify-content` on the main axis.
    pub fn justify(mut self, value: Justify) -> Self {
        self.inner.style = self.inner.style.justify(value);
        self
    }

    /// Sets `align-content`, used when wrapped lines have extra cross-axis space.
    pub fn align_content(mut self, value: Justify) -> Self {
        self.inner.style = self.inner.style.align_content(value);
        self
    }

    /// Sets `flex-wrap`.
    pub fn wrap(mut self, value: Wrap) -> Self {
        self.inner.style = self.inner.style.wrap(value);
        self
    }

    layout_container_methods!();
}

impl Widget for Flex {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }
}

/// A CSS grid track. Use [`Track::fr`] for proportional space, [`Track::px`]
/// for a fixed track, and [`Track::minmax`] for a track that's at least
/// `min` px wide but shares any remaining space like an `fr` track — the
/// building block of a responsive `repeat(auto-fit, minmax(..))` gallery
/// (see [`Grid::auto_fit_columns`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Track {
    Auto,
    Px(f32),
    Fr(f32),
    MinMax { min: f32, max_fr: f32 },
}

impl Track {
    pub const fn auto() -> Self {
        Self::Auto
    }

    pub const fn px(value: f32) -> Self {
        Self::Px(value)
    }

    pub const fn fr(value: f32) -> Self {
        Self::Fr(value)
    }

    /// A track that never shrinks below `min` px, then shares remaining
    /// space as `max_fr` fractional units — CSS's `minmax(min, max_fr fr)`.
    pub const fn minmax(min: f32, max_fr: f32) -> Self {
        Self::MinMax { min, max_fr }
    }

    fn sizing(self) -> NonRepeatedTrackSizingFunction {
        match self {
            Track::Auto => creamui_core::layout::auto(),
            Track::Px(value) => creamui_core::layout::length(value),
            Track::Fr(value) => creamui_core::layout::fr(value),
            Track::MinMax { min, max_fr } => creamui_core::layout::minmax(
                creamui_core::layout::length(min),
                creamui_core::layout::fr(max_fr),
            ),
        }
    }
}

/// Automatic placement order for a [`Grid`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

impl From<GridFlow> for GridAutoFlow {
    fn from(value: GridFlow) -> Self {
        match value {
            GridFlow::Row => Self::Row,
            GridFlow::Column => Self::Column,
            GridFlow::RowDense => Self::RowDense,
            GridFlow::ColumnDense => Self::ColumnDense,
        }
    }
}

/// An unstyled CSS grid container, equivalent to `<div style="display: grid">`.
///
/// ```
/// use creamui_widgets::layout::{Grid, Track};
///
/// let dashboard = Grid::new()
///     .columns(3)
///     .gap(16.0)
///     .template_rows([Track::px(48.0), Track::fr(1.0)]);
///
/// // A responsive card gallery, no manual column-count math required:
/// // as many 180px-or-wider columns as fit, sharing the rest of the width.
/// let gallery = Grid::new().auto_fit_columns(180.0).gap(16.0);
/// ```
pub struct Grid {
    inner: crate::raw::RawView,
}

impl Grid {
    /// Creates an empty grid. Configure columns/rows with [`Grid::columns`],
    /// [`Grid::rows`], or explicit track templates.
    pub fn new() -> Self {
        Self {
            inner: crate::raw::RawView::new(shrinkable(Style {
                display: Display::Grid,
                ..Default::default()
            })),
        }
    }

    /// Creates `count` equal `1fr` columns.
    pub fn columns(mut self, count: usize) -> Self {
        self.inner.style.grid_template_columns = equal_tracks(count);
        self
    }

    /// Creates `count` equal `1fr` rows.
    pub fn rows(mut self, count: usize) -> Self {
        self.inner.style.grid_template_rows = equal_tracks(count);
        self
    }

    /// Sets an explicit CSS-like `grid-template-columns` list.
    pub fn template_columns(mut self, tracks: impl IntoIterator<Item = Track>) -> Self {
        self.inner.style.grid_template_columns = tracks
            .into_iter()
            .map(|track| TrackSizingFunction::Single(track.sizing()))
            .collect();
        self
    }

    /// Sets an explicit CSS-like `grid-template-rows` list.
    pub fn template_rows(mut self, tracks: impl IntoIterator<Item = Track>) -> Self {
        self.inner.style.grid_template_rows = tracks
            .into_iter()
            .map(|track| TrackSizingFunction::Single(track.sizing()))
            .collect();
        self
    }

    /// A responsive column template equivalent to CSS's
    /// `repeat(auto-fit, minmax(min, 1fr))`: Taffy generates as many
    /// `min`-px-or-wider columns as fit the grid's own available width, and
    /// the columns share whatever's left over evenly — collapsing empty
    /// tracks (and their gaps) if the content doesn't fill a whole row.
    ///
    /// This replaces manually computing a column count from a measured
    /// pixel width every frame: the same layout that a resize would have
    /// required recomputing by hand is instead resolved natively during
    /// layout, and only actually recomputed when the available width
    /// changes.
    pub fn auto_fit_columns(mut self, min: f32) -> Self {
        self.inner.style.grid_template_columns =
            auto_repeat_tracks(GridTrackRepetition::AutoFit, min);
        self
    }

    /// Like [`Grid::auto_fit_columns`], but keeps empty tracks (and their
    /// gaps) around instead of collapsing them — CSS grid's `auto-fill` vs
    /// `auto-fit` distinction.
    pub fn auto_fill_columns(mut self, min: f32) -> Self {
        self.inner.style.grid_template_columns =
            auto_repeat_tracks(GridTrackRepetition::AutoFill, min);
        self
    }

    /// Sets equal row and column gaps.
    pub fn gap(mut self, value: f32) -> Self {
        self.inner.style = self.inner.style.gap(value);
        self
    }

    /// Sets horizontal (`column-gap`) spacing.
    pub fn gap_x(mut self, value: f32) -> Self {
        self.inner.style = self.inner.style.gap_x(value);
        self
    }

    /// Sets vertical (`row-gap`) spacing.
    pub fn gap_y(mut self, value: f32) -> Self {
        self.inner.style = self.inner.style.gap_y(value);
        self
    }

    /// Sets `justify-content` on the inline axis.
    pub fn justify(mut self, value: Justify) -> Self {
        self.inner.style = self.inner.style.justify(value);
        self
    }

    /// Sets `align-content` on the block axis.
    pub fn align_content(mut self, value: Justify) -> Self {
        self.inner.style = self.inner.style.align_content(value);
        self
    }

    /// Sets the automatic grid placement order.
    pub fn flow(mut self, value: GridFlow) -> Self {
        self.inner.style.grid_auto_flow = value.into();
        self
    }

    layout_container_methods!();
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Grid {
    fn style(&self) -> Style {
        self.inner.style()
    }

    fn paint(&self, painter: &mut dyn Painter, rect: Rect) {
        self.inner.paint(painter, rect);
    }

    fn children(&mut self) -> Vec<BoxedWidget> {
        self.inner.children()
    }
}

/// A positioned child of a [`Grid`]. Grid lines are 1-indexed, matching CSS.
pub struct GridItem {
    style: Style,
    children: Vec<BoxedWidget>,
}

impl GridItem {
    /// Creates an automatically placed grid item.
    pub fn new() -> Self {
        Self {
            style: shrinkable(Style {
                display: Display::Block,
                ..Default::default()
            }),
            children: vec![],
        }
    }

    /// Places the item at a 1-indexed `(column, row)` cell.
    pub fn at(mut self, column: i16, row: i16) -> Self {
        self.style = self.style.grid_cell(column, row);
        self
    }

    /// Sets the starting 1-indexed grid column.
    pub fn column(mut self, column: i16) -> Self {
        self.style.grid_column.start = GridPlacement::from_line_index(column);
        self
    }

    /// Sets the starting 1-indexed grid row.
    pub fn row(mut self, row: i16) -> Self {
        self.style.grid_row.start = GridPlacement::from_line_index(row);
        self
    }

    /// Makes the item span `count` columns.
    pub fn column_span(mut self, count: u16) -> Self {
        self.style.grid_column.end = GridPlacement::Span(count);
        self
    }

    /// Makes the item span `count` rows.
    pub fn row_span(mut self, count: u16) -> Self {
        self.style.grid_row.end = GridPlacement::Span(count);
        self
    }

    /// Sets a minimum pixel width, opting back into "never shrink below
    /// this width" — grid items already shrink to `0` by default (see the
    /// module docs), so this is only needed to keep one item from
    /// shrinking as readily as its siblings.
    pub fn min_width(mut self, value: f32) -> Self {
        self.style.min_size.width = Dimension::Length(value);
        self
    }

    /// Sets a minimum pixel height; see [`GridItem::min_width`].
    pub fn min_height(mut self, value: f32) -> Self {
        self.style.min_size.height = Dimension::Length(value);
        self
    }

    /// Adds the item's content.
    pub fn child(mut self, child: BoxedWidget) -> Self {
        self.children.push(child);
        self
    }

    /// Adds all item content at once.
    pub fn with_children(mut self, children: Vec<BoxedWidget>) -> Self {
        self.children = children;
        self
    }
}

impl Default for GridItem {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for GridItem {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, _: &mut dyn Painter, _: Rect) {}

    fn children(&mut self) -> Vec<BoxedWidget> {
        std::mem::take(&mut self.children)
    }
}

fn equal_tracks(count: usize) -> Vec<TrackSizingFunction> {
    let track: NonRepeatedTrackSizingFunction = creamui_core::layout::fr(1.0f32);
    vec![TrackSizingFunction::Single(track); count]
}

/// A single `repeat(auto-fit | auto-fill, minmax(min, 1fr))` track group —
/// the whole point of `repeat`: Taffy decides how many copies fit at
/// layout time instead of the caller precomputing a column count.
fn auto_repeat_tracks(repetition: GridTrackRepetition, min: f32) -> Vec<TrackSizingFunction> {
    vec![TrackSizingFunction::Repeat(
        repetition,
        vec![creamui_core::layout::minmax(
            creamui_core::layout::length(min),
            creamui_core::layout::fr(1.0f32),
        )],
    )]
}

/// Chainable CSS-like layout properties for a raw Taffy [`Style`].
///
/// This is useful for a flex *item* or for adding flex behavior to another
/// widget without wrapping it in [`Flex`]. Import the trait to use it:
///
/// ```
/// use creamui_core::layout::Style;
/// use creamui_widgets::layout::{Align, Justify, StyleExt};
///
/// let style = Style::default()
///     .flex_column()
///     .gap(8.0)
///     .align(Align::Stretch)
///     .justify(Justify::Center);
/// ```
pub trait StyleExt: Sized {
    /// Enables block layout, equivalent to CSS `display: block`.
    fn block(self) -> Self;
    /// Enables CSS grid layout.
    fn grid(self) -> Self;
    /// Enables flex layout while keeping the current direction.
    fn flex(self) -> Self;
    /// Enables flex layout in a row.
    fn flex_row(self) -> Self;
    /// Enables flex layout in a column.
    fn flex_column(self) -> Self;
    /// Sets equal row and column gaps.
    fn gap(self, value: f32) -> Self;
    /// Sets horizontal (`column-gap`) spacing.
    fn gap_x(self, value: f32) -> Self;
    /// Sets vertical (`row-gap`) spacing.
    fn gap_y(self, value: f32) -> Self;
    /// Sets `align-items`.
    fn align(self, value: Align) -> Self;
    /// Sets `justify-content`.
    fn justify(self, value: Justify) -> Self;
    /// Sets `align-content` for wrapped flex lines.
    fn align_content(self, value: Justify) -> Self;
    /// Sets `flex-wrap`.
    fn wrap(self, value: Wrap) -> Self;
    /// Sets equal padding.
    fn padding_all(self, value: f32) -> Self;
    /// Sets CSS-like `padding: vertical horizontal` spacing.
    fn padding_xy(self, horizontal: f32, vertical: f32) -> Self;
    /// Sets equal margins.
    fn margin_all(self, value: f32) -> Self;
    /// Sets CSS-like `margin: vertical horizontal` spacing.
    fn margin_xy(self, horizontal: f32, vertical: f32) -> Self;
    /// Sets a fixed pixel size.
    fn size(self, width: f32, height: f32) -> Self;
    /// Sets only a fixed pixel width.
    fn width(self, value: f32) -> Self;
    /// Sets only a fixed pixel height.
    fn height(self, value: f32) -> Self;
    /// Sets a minimum pixel size, opting back into "never shrink below
    /// this size" — Taffy's own default. [`Flex`]/[`Block`]/[`Grid`]
    /// override that default to `0`; a bare [`Style`] still starts from
    /// Taffy's `Auto`, so use this when composing one by hand for a
    /// flex/grid child that needs to actually shrink.
    fn min_size(self, width: f32, height: f32) -> Self;
    /// Sets a minimum pixel width; see [`StyleExt::min_size`].
    fn min_width(self, value: f32) -> Self;
    /// Sets a minimum pixel height; see [`StyleExt::min_size`].
    fn min_height(self, value: f32) -> Self;
    /// Sets `flex-grow`.
    fn grow(self, factor: f32) -> Self;
    /// Sets `flex-shrink`.
    fn shrink(self, factor: f32) -> Self;
    /// Sets a fixed pixel `flex-basis`.
    fn basis(self, value: f32) -> Self;
    /// Sets `align-self` on a flex item.
    fn align_self(self, value: Align) -> Self;
    /// Places a grid item at a 1-indexed `(column, row)` cell.
    fn grid_cell(self, column: i16, row: i16) -> Self;
    /// Makes a grid item span `count` columns.
    fn grid_column_span(self, count: u16) -> Self;
    /// Makes a grid item span `count` rows.
    fn grid_row_span(self, count: u16) -> Self;
}

impl StyleExt for Style {
    fn block(mut self) -> Self {
        self.display = Display::Block;
        self
    }

    fn grid(mut self) -> Self {
        self.display = Display::Grid;
        self
    }

    fn flex(mut self) -> Self {
        self.display = Display::Flex;
        self
    }

    fn flex_row(mut self) -> Self {
        self.display = Display::Flex;
        self.flex_direction = FlexDirection::Row;
        self
    }

    fn flex_column(mut self) -> Self {
        self.display = Display::Flex;
        self.flex_direction = FlexDirection::Column;
        self
    }

    fn gap(mut self, value: f32) -> Self {
        self.gap = gap(value, value);
        self
    }

    fn gap_x(mut self, value: f32) -> Self {
        self.gap.width = LengthPercentage::Length(value);
        self
    }

    fn gap_y(mut self, value: f32) -> Self {
        self.gap.height = LengthPercentage::Length(value);
        self
    }

    fn align(mut self, value: Align) -> Self {
        self.align_items = Some(value.into());
        self
    }

    fn justify(mut self, value: Justify) -> Self {
        self.justify_content = Some(value.into());
        self
    }

    fn align_content(mut self, value: Justify) -> Self {
        self.align_content = Some(align_content(value));
        self
    }

    fn wrap(mut self, value: Wrap) -> Self {
        self.flex_wrap = value.into();
        self
    }

    fn padding_all(mut self, value: f32) -> Self {
        self.padding = edges(value, value, value, value);
        self
    }

    fn padding_xy(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding = edges(horizontal, horizontal, vertical, vertical);
        self
    }

    fn margin_all(mut self, value: f32) -> Self {
        self.margin = auto_edges(value, value, value, value);
        self
    }

    fn margin_xy(mut self, horizontal: f32, vertical: f32) -> Self {
        self.margin = auto_edges(horizontal, horizontal, vertical, vertical);
        self
    }

    fn size(mut self, width: f32, height: f32) -> Self {
        self.size = fixed(width, height);
        self
    }

    fn width(mut self, value: f32) -> Self {
        self.size.width = Dimension::Length(value);
        self
    }

    fn height(mut self, value: f32) -> Self {
        self.size.height = Dimension::Length(value);
        self
    }

    fn min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = fixed(width, height);
        self
    }

    fn min_width(mut self, value: f32) -> Self {
        self.min_size.width = Dimension::Length(value);
        self
    }

    fn min_height(mut self, value: f32) -> Self {
        self.min_size.height = Dimension::Length(value);
        self
    }

    fn grow(mut self, factor: f32) -> Self {
        self.flex_grow = factor;
        self
    }

    fn shrink(mut self, factor: f32) -> Self {
        self.flex_shrink = factor;
        self
    }

    fn basis(mut self, value: f32) -> Self {
        self.flex_basis = Dimension::Length(value);
        self
    }

    fn align_self(mut self, value: Align) -> Self {
        self.align_self = Some(value.into());
        self
    }

    fn grid_cell(mut self, column: i16, row: i16) -> Self {
        self.grid_column.start = GridPlacement::from_line_index(column);
        self.grid_column.end = GridPlacement::Auto;
        self.grid_row.start = GridPlacement::from_line_index(row);
        self.grid_row.end = GridPlacement::Auto;
        self
    }

    fn grid_column_span(mut self, count: u16) -> Self {
        self.grid_column.end = GridPlacement::Span(count);
        self
    }

    fn grid_row_span(mut self, count: u16) -> Self {
        self.grid_row.end = GridPlacement::Span(count);
        self
    }
}

fn gap(horizontal: f32, vertical: f32) -> creamui_core::layout::Size<LengthPercentage> {
    creamui_core::layout::Size {
        width: LengthPercentage::Length(horizontal),
        height: LengthPercentage::Length(vertical),
    }
}

fn align_content(value: Justify) -> AlignContent {
    match value {
        Justify::Start => AlignContent::Start,
        Justify::Center => AlignContent::Center,
        Justify::End => AlignContent::End,
        Justify::Between => AlignContent::SpaceBetween,
        Justify::Around => AlignContent::SpaceAround,
        Justify::Evenly => AlignContent::SpaceEvenly,
    }
}

/// A flex row style with a fixed pixel `gap`. Shrinkable by default; see the
/// module docs.
pub fn row(gap: f32) -> Style {
    shrinkable(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        gap: creamui_core::layout::Size {
            width: LengthPercentage::Length(gap),
            height: LengthPercentage::Length(gap),
        },
        ..Default::default()
    })
}

/// A flex column style with a fixed pixel `gap`. Shrinkable by default; see
/// the module docs.
pub fn column(gap: f32) -> Style {
    shrinkable(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        gap: creamui_core::layout::Size {
            width: LengthPercentage::Length(gap),
            height: LengthPercentage::Length(gap),
        },
        ..Default::default()
    })
}

/// A CSS-grid-like layout with `columns` equal-width tracks and a fixed
/// pixel `gap`. Shrinkable by default; see the module docs.
pub fn grid(columns: usize, gap: f32) -> Style {
    let track: NonRepeatedTrackSizingFunction = creamui_core::layout::fr(1.0f32);
    shrinkable(Style {
        display: creamui_core::layout::Display::Grid,
        grid_template_columns: vec![TrackSizingFunction::Single(track); columns],
        gap: creamui_core::layout::Size {
            width: LengthPercentage::Length(gap),
            height: LengthPercentage::Length(gap),
        },
        ..Default::default()
    })
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

    #[test]
    fn style_extension_expresses_flex_properties() {
        let style = Style::default()
            .flex_column()
            .gap_x(12.0)
            .gap_y(8.0)
            .align(Align::Stretch)
            .justify(Justify::Between)
            .align_content(Justify::Around)
            .wrap(Wrap::Wrap)
            .padding_xy(16.0, 10.0)
            .grow(1.0)
            .basis(200.0);

        assert_eq!(style.display, creamui_core::layout::Display::Flex);
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.gap.width, LengthPercentage::Length(12.0));
        assert_eq!(style.gap.height, LengthPercentage::Length(8.0));
        assert_eq!(style.align_items, Some(AlignItems::Stretch));
        assert_eq!(style.justify_content, Some(JustifyContent::SpaceBetween));
        assert_eq!(style.align_content, Some(AlignContent::SpaceAround));
        assert_eq!(style.flex_wrap, FlexWrap::Wrap);
        assert_eq!(style.flex_grow, 1.0);
        assert_eq!(style.flex_basis, Dimension::Length(200.0));
    }

    #[test]
    fn flex_container_uses_css_defaults_until_configured() {
        let style = Flex::row()
            .gap(10.0)
            .align(Align::Center)
            .justify(Justify::End)
            .wrap(Wrap::Reverse)
            .style();

        assert_eq!(style.display, creamui_core::layout::Display::Flex);
        assert_eq!(style.flex_direction, FlexDirection::Row);
        assert_eq!(style.gap.width, LengthPercentage::Length(10.0));
        assert_eq!(style.gap.height, LengthPercentage::Length(10.0));
        assert_eq!(style.align_items, Some(AlignItems::Center));
        assert_eq!(style.justify_content, Some(JustifyContent::End));
        assert_eq!(style.flex_wrap, FlexWrap::WrapReverse);
    }

    #[test]
    fn block_is_always_a_block_layout_container() {
        let style = Block::with_style(Style {
            display: creamui_core::layout::Display::Flex,
            ..Default::default()
        })
        .style();

        assert_eq!(style.display, creamui_core::layout::Display::Block);
    }

    #[test]
    fn grid_expresses_tracks_and_item_placement() {
        let style = Grid::new()
            .columns(2)
            .template_rows([Track::Px(40.0), Track::Fr(1.0)])
            .gap(12.0)
            .style();
        let item = GridItem::new()
            .at(2, 1)
            .column_span(2)
            .row_span(3)
            .min_width(0.0)
            .style();

        assert_eq!(style.display, creamui_core::layout::Display::Grid);
        assert_eq!(style.grid_template_columns.len(), 2);
        assert_eq!(style.grid_template_rows.len(), 2);
        assert_eq!(style.gap.width, LengthPercentage::Length(12.0));
        assert_eq!(item.grid_column.start, GridPlacement::from_line_index(2));
        assert_eq!(item.grid_column.end, GridPlacement::Span(2));
        assert_eq!(item.grid_row.start, GridPlacement::from_line_index(1));
        assert_eq!(item.grid_row.end, GridPlacement::Span(3));
        assert_eq!(item.min_size.width, Dimension::Length(0.0));
    }

    #[test]
    fn containers_shrink_below_content_by_default() {
        // A 400px-wide child inside a 100px-wide flex row: without the
        // `min-size: 0` default this would refuse to shrink and overflow
        // the parent instead of clamping to the available width.
        let mut tree = creamui_core::layout::TaffyTree::<()>::new();
        let child_style = Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(400.0),
                height: Dimension::Length(20.0),
            },
            flex_shrink: 1.0,
            ..Default::default()
        };
        let child = tree.new_leaf(shrinkable(child_style)).unwrap();
        let root = tree
            .new_with_children(row(0.0).size(100.0, 20.0), &[child])
            .unwrap();
        tree.compute_layout(
            root,
            creamui_core::layout::Size {
                width: AvailableSpace::Definite(100.0),
                height: AvailableSpace::Definite(20.0),
            },
        )
        .unwrap();

        assert_eq!(tree.layout(child).unwrap().size.width, 100.0);
    }

    #[test]
    fn auto_fit_columns_resolves_the_column_count_from_available_width() {
        let mut tree = creamui_core::layout::TaffyTree::<()>::new();
        let child_style = shrinkable(Style::default());
        let children: Vec<_> = (0..6)
            .map(|_| tree.new_leaf(child_style.clone()).unwrap())
            .collect();
        let root_style = Style {
            size: fixed(400.0, 200.0),
            ..Grid::new().auto_fit_columns(180.0).gap(20.0).style()
        };
        let root = tree.new_with_children(root_style, &children).unwrap();
        tree.compute_layout(
            root,
            creamui_core::layout::Size {
                width: AvailableSpace::Definite(400.0),
                height: AvailableSpace::Definite(200.0),
            },
        )
        .unwrap();

        // (400 - 20 gap) / (180 + 20) rounds down to 2 columns of ~190px.
        let first = tree.layout(children[0]).unwrap();
        let second = tree.layout(children[1]).unwrap();
        let third = tree.layout(children[2]).unwrap();
        assert_eq!(first.location.x, 0.0);
        assert!(second.location.x > first.location.x);
        assert_eq!(
            third.location.y > first.location.y,
            true,
            "a third column shouldn't fit, so the third card wraps to row 2"
        );
    }
}
