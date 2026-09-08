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
        <View theme={&theme} style={my_layout(viewport)}>
            <Text theme={&theme}>"Welcome"</Text>
        </View>
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

`creamui::widgets::layout` provides compact builders for the common cases:

- `row(gap)` and `column(gap)` for flex layouts.
- `grid(columns, gap)` for equal-width grid tracks.
- `fixed(width, height)` for explicit sizes.
- `padding`, `margin`, `fill`, and `centered` for common adjustments.

For advanced layouts, use the re-exported Taffy types through `creamui::core::layout`.

## Next steps

- Browse [components](components.md) for the component families and their state model.
- Read [theming](theming.md) before creating an application-specific visual language.
- Run `cargo run -p showcase` from a clone of this repository to explore live controls.
