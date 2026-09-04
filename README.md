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

## Running the examples

```sh
cargo run -p hello_world           # static: links the Rust crates directly
cargo run -p hello_world_dynamic   # dynamic: dlopens the built cdylib, zero CreamUI crate deps
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
