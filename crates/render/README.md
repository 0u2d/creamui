# creamui-render

Native window hosting and frame presentation for CreamUI.

`run` creates a desktop window, builds a widget tree reactively, lays it out, rasterizes it, and presents the resulting frame. GPU presentation is the default; a CPU-only `softbuffer` backend is available through `WindowOptions` when that is a better fit for the host environment.

```rust
creamui_render::run(options, theme.surface, |_| {}, build_ui);
```

For a complete example, see the [CreamUI repository](https://github.com/sammwyy/creamui).
