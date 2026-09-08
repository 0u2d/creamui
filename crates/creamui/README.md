# creamui

The recommended starting point for CreamUI applications.

By default, `creamui` re-exports the native runtime: widgets, theme tokens, reactive signals, layout primitives, and window rendering. Optional integrations are opt-in so applications only compile what they use.

```toml
[dependencies]
creamui = { version = "0.1", features = ["jsx", "image"] }
```

| Feature | Enables |
|---|---|
| `image` | `creamui-image` with PNG decoding |
| `image-jpeg` / `image-webp` | JPEG or WebP decoding, respectively |
| `jsx` | JSX macros and runtime support |
| `abi` | C-compatible ABI types |
| `dynamic` | Dynamic runtime client and ABI types |
| `ffi` | Shared-library C ABI and ABI types |
| `full` | Every optional integration |

Use modules such as `creamui::widgets`, `creamui::theme`, and `creamui::render`, or import common items directly from `creamui`.

See the [CreamUI getting-started guide](https://github.com/sammwyy/creamui/blob/main/docs/getting-started.md).
