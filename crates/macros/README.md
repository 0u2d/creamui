# creamui-macros

JSX syntax and typed component helpers for CreamUI.

```rust
use creamui_macros::jsx;

let screen = jsx! {
    <View theme={&theme} style={style}>
        <Button theme={&theme} on_click={|| save()}>"Save"</Button>
    </View>
};
```

The macro expands to ordinary Rust constructors, so component names, imports, props, and callback types are checked by the compiler.
