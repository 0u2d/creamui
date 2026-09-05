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
| `creamui-render` | winit windowing + `tiny-skia` CPU rasterization + `wgpu`/`softbuffer` presentation |
| `creamui-abi` | Plain `#[repr(C)]` ABI types shared by `creamui-ffi` and `creamui-dynamic`, no engine dependency |
| `creamui-ffi` | `#[no_mangle] extern "C"` ABI, built as a `cdylib`, for dynamic linking |
| `creamui-dynamic` | Safe client that `dlopen`s the `cdylib` and resolves its ABI once, for apps that don't want to hand-write `libloading`/`#[repr(C)]` boilerplate |

## Running the examples

```sh
cargo run -p hello_world           # static: links the Rust crates directly
cargo run -p hello_world_dynamic   # dynamic: dlopens the built cdylib, zero CreamUI crate deps
```

Both are the same themed counter with a runtime theme-toggle button.
`hello_world_dynamic` depends only on `creamui-dynamic` (which itself only
depends on `creamui-abi` + `libloading`) — no engine crate compiled in.
Compare `target/release/hello_world` against `target/release/hello_world_dynamic`
to see the difference linking mode makes to binary size: the dynamic build
carries none of `wgpu`/`winit`/`taffy` itself — that all lives in
`libcreamui.so`.

Set `CUI_DEBUG=1` for verbose logging, or `CUI_DUMP_FRAME=<path.png>`
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

Shape and text rasterization always run on the CPU via `tiny-skia` and
`fontdue`. What happens to that buffer next is chosen by the embedding app
at window-creation time via `WindowOptions::backend`
(`CWindowOptions::backend` over FFI): `RenderBackend::Gpu` (the default)
uploads it to a GPU texture and composites it via a single textured `wgpu`
triangle; `RenderBackend::Cpu` blits it straight to the window surface with
`softbuffer`, skipping GPU init entirely. Either way this keeps the MVP's
rendering code small. A fully GPU-driven vector renderer is on the roadmap.

Whatever the app requests can be force-overridden at launch, without a
rebuild, by setting `CUI_OVERRIDE_RENDER_BACKEND=gpu` or `=cpu` — handy for
testing the CPU path or working around a broken GPU driver.

`creamui_render::run` opens a single window. For several windows sharing one
process and event loop — e.g. a desktop-shell dock where each icon is its
own window — use `AppBuilder` instead: `AppBuilder::new().window(...).window(...).run()`.
Every window keeps fully independent reactive/paint state, and all
GPU-backend windows share a single `wgpu::Instance` rather than each paying
its own driver-init cost. Note this only amortizes the *instance*; each
GPU-backend window still creates its own `wgpu::Device`, which is where most
of the GPU backend's per-window memory actually goes — for a dock with many
small windows, prefer `RenderBackend::Cpu` unless a given window specifically
needs GPU compositing.

The same multi-window capability is exposed across the ABI —
`creamui_app_builder_new`/`creamui_app_builder_add_window`/`creamui_app_builder_run`
in `creamui-ffi` — and as `creamui_dynamic::AppBuilder` on the `dlopen`
side, mirroring the native API one-for-one.

Text layout uses a real `taffy` measure function (`creamui_core::Widget::measure`)
backed by `fontdue`'s own line-width calculation, not a hand-rolled estimate —
see `creamui-widgets::text_metrics`.

## Credits

CreamUI bundles [DejaVu Sans](https://dejavu-fonts.github.io/) as its
default font (`assets/fonts/DejaVuSans.ttf`), licensed under the permissive
Bitstream Vera license — see `assets/fonts/DejaVuSans-LICENSE.txt`.
