# CreamUI

A GPU-rendered, reactive UI toolkit for Rust: fine-grained reactivity,
CSS-like flex/grid layout, and a themeable widget system, designed from the
start to also be linked dynamically through a stable C ABI when an
application wants to share one runtime instead of bundling its own copy.

**Status: early MVP, Linux only.** See [`ROADMAP.md`](ROADMAP.md) for what's
done and what's next.

## Design

- **Reactive.** [`creamui-reactive`](crates/reactive) provides `Signal<T>`
  and `create_effect`; widget trees are rebuilt automatically whenever a
  signal they read changes.
- **Renderable components.** [`creamui-core`](crates/core) defines the
  `Widget` trait and a backend-agnostic `Painter` trait.
- **HTML/CSS-like layout.** Layout is `taffy`'s flex and grid engine
  (re-exported through `creamui_core::layout`), not a bespoke engine.
- **Themeable, headless-first widgets.** [`creamui-theme`](crates/theme)
  defines semantic design tokens (`accent`, `surface`, `radiusMedium`, ...).
  [`creamui-widgets`](crates/widgets) ships headless widgets (`RawButton`,
  `RawText`, `RawView`) with no styling opinion at all, and themed wrappers
  (`Button`, `Text`, `View`) built on top of them that read from an
  in-memory `Theme`. Writing your own themed/derived widgets follows the
  same pattern.
- **Stable C ABI, by choice.** [`creamui-ffi`](crates/ffi) exposes a
  `#[no_mangle] extern "C"` interface built as a `cdylib`. Static linking
  by depending on the Rust crates directly is the default and simplest
  path; the ABI exists for cases that specifically want dynamic linking —
  e.g. several apps in a desktop environment sharing one CreamUI runtime,
  or consuming CreamUI from a non-Rust language.

## Crates

| Crate | Purpose |
|---|---|
| `creamui-reactive` | `Signal<T>` / `create_effect` reactive primitives |
| `creamui-core` | `Widget` trait, layout (via `taffy`), `Painter` trait, scene/hit-testing |
| `creamui-theme` | Semantic design tokens (`Theme`, `Color`) |
| `creamui-widgets` | Headless (`raw`) and themed (`themed`) widgets, layout helpers |
| `creamui-render` | winit windowing + `tiny-skia` CPU rasterization + `wgpu` presentation |
| `creamui-ffi` | `#[no_mangle] extern "C"` ABI, built as a `cdylib`, for dynamic linking |
| `creamui-macros` | `jsx!` syntax for the Rust widget builders |

## Running the examples

```sh
cargo run -p hello_world           # static: links the Rust crates directly
cargo run -p hello_world_dynamic   # dynamic: dlopens the built cdylib, zero CreamUI crate deps
cargo run -p jsx_hello_world       # static counter expressed with jsx!
```

Both are the same themed counter with a runtime theme-toggle button (the
static one only — the dynamic ABI doesn't expose theme tokens yet, see
`ROADMAP.md`). Compare `target/release/hello_world` against
`target/release/hello_world_dynamic` to see the difference linking mode
makes to binary size: the dynamic build carries none of `wgpu`/`winit`/
`taffy` itself — that all lives in `libcreamui.so`.

Set `CREAMUI_DEBUG=1` for verbose logging, or `CREAMUI_DUMP_FRAME=<path.png>`
to write every painted frame to a PNG (useful for headless verification with
no compositor attached).

## Testing

```sh
cargo test --workspace
```

This includes an end-to-end test (`creamui-widgets`) that renders a themed
button through the real layout/paint pipeline and asserts the click handler
fires, and a `libloading`-based test (`creamui-ffi`) that `dlopen`s the
built `cdylib` and drives it purely through its C ABI.

## How rendering works (MVP)

Shape and text rasterization run on the CPU via `tiny-skia` and `fontdue`;
the result is uploaded to a GPU texture and composited to the window
surface via a single textured `wgpu` triangle. This keeps the MVP's
rendering code small while still presenting through the GPU. A fully
GPU-driven vector renderer is on the roadmap.

Text layout uses a real `taffy` measure function (`creamui_core::Widget::measure`)
backed by `fontdue`'s own line-width calculation, not a hand-rolled estimate —
see `creamui-widgets::text_metrics`.

## JSX

`creamui-macros::jsx!` expands directly to the existing Rust widget
constructors. It owns no runtime state and has no special rendering path, so
`Signal` reads and callback closures keep exactly their usual behavior.

```rust
use creamui_macros::jsx;

let tree = jsx! {
    <View theme={&theme} style={creamui_widgets::layout::column(8.0)}>
        <Text theme={&theme} font_size={20.0}>"Settings"</Text>
        <Button theme={&theme} on_click={move || save()}>"Save"</Button>
    </View>
};
```

Every prop is a Rust expression inside braces. `style` is therefore a normal
`creamui_core::layout::Style`, including values built with
`creamui_widgets::layout::{row, column, grid, fixed}`. This keeps the JSX
surface aligned with Taffy's complete style API instead of introducing a
second, partial CSS object syntax. `RawView` accepts `style`, `background`,
and `corner_radius`; `View` accepts `theme` and `style`; `Text` accepts
`theme`, `font_size`, and `secondary`; and `Button` accepts `theme` and
`on_click`. `Checkbox`, `TextInput`, `Slider`, and `ScrollView` map one to
one to their themed constructors: their required state/callback props retain
the constructor names (`checked`/`on_click`, `value`/`on_change`, or
`scroll_y`/`on_scroll`). Containers accept nested components; dynamic child
widgets can be supplied as a `{BoxedWidget}` expression.

Text content is either a Rust string expression (`{format!(...)}`) or a Rust
string literal (`"Save"`). This is intentional: it preserves Rust's normal
string and formatting semantics without a separate JSX text lexer.

There is one macro crate, not an `abi_macros` crate. Proc macros expand in a
Rust crate and produce Rust widget builders; the C ABI is a runtime boundary
whose consumers cannot invoke Rust proc macros. If a future Rust-facing ABI
adapter needs a declarative API, it should use this macro and expose a
purpose-built Rust wrapper, rather than duplicate parser and prop rules.

## Credits

CreamUI bundles [DejaVu Sans](https://dejavu-fonts.github.io/) as its
default font (`assets/fonts/DejaVuSans.ttf`), licensed under the permissive
Bitstream Vera license — see `assets/fonts/DejaVuSans-LICENSE.txt`.
