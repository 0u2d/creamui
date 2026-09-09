# Getting started

CreamUI applications are normal Rust binaries. Build a widget tree in a closure, return it from `run`, and let signals rebuild that tree when state changes.

## Install

For a standard native app, add the facade crate:

```toml
[dependencies]
creamui = { version = "0.1", features = ["jsx"] }
```

Enable image support from the facade when your app displays raster assets:

```toml
creamui = { version = "0.1", features = ["jsx", "image-jpeg", "image-webp"] }
```

PNG is enabled by default. JPEG and WebP stay opt-in to keep application builds focused on the formats they use.

## Build a screen

`creamui::run` creates a window. Its final closure receives the current viewport and returns the root widget for that frame.

```rust
run(options, Theme::default().surface, |_| {}, move |viewport| {
    let theme = Theme::default();
    Box::new(jsx! {
        <Block style={my_layout(viewport)}>
            <Text>"Welcome"</Text>
        </Block>
    })
});
```

Use `Signal<T>` for UI state. Calling `get()` while building subscribes the screen to that value; changing it rebuilds the affected window on the next frame.

```rust
let enabled = Signal::new(false);
let set_enabled = enabled.clone();

jsx! {
    <Checkbox theme={&theme} checked={enabled.get()}
        on_click={move || set_enabled.update(|value| *value = !*value)} />
}
```

## Choose a layout

`creamui::widgets::layout` has a semantic flex API for layout containers:

```rust
use creamui::widgets::layout::{Align, Flex, Justify, Wrap};

let toolbar = Flex::row()
    .gap(12.0)
    .align(Align::Center)
    .justify(Justify::Between);

let cards = Flex::row()
    .gap_x(16.0)
    .gap_y(12.0)
    .wrap(Wrap::Wrap);
```

For two-dimensional layout, use `Grid` and position only the items that need
an explicit cell or span:

```rust
use creamui::widgets::layout::{Grid, GridItem, Track};

let dashboard = Grid::new()
    .template_columns([Track::px(240.0), Track::fr(1.0), Track::fr(1.0)])
    .gap(16.0)
    .child(Box::new(GridItem::new().at(1, 1).column_span(2)));
```

The same container is available in JSX when the `jsx` feature is enabled:

```rust
use creamui::core::layout::FlexDirection;

jsx! {
    <Flex direction={FlexDirection::Column} gap={12.0}
        align={Align::Center} justify={Justify::Center}>
        <Text>"Centered content"</Text>
    </Flex>
}
```

`Flex::row()` and `Flex::column()` are unstyled `div`-like containers. They
support `gap`, axis alignment, `justify`, wrapping, padding, fixed/fill sizes,
and item behavior (`grow`, `shrink`, `basis`, `align_self`). For the same
chainable properties on any `Style`, import `StyleExt` and start with
`Style::default().flex_row()`, `.flex_column()`, or `.grid()`.

For advanced layout properties, use the re-exported Taffy types through
`creamui::core::layout`.

## Next steps

- Browse [components](components.md) for the component families and their state model.
- Read [theming](theming.md) before creating an application-specific visual language.
- Run `cargo run -p showcase` from a clone of this repository to explore live controls.
