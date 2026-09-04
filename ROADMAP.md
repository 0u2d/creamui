# CreamUI Roadmap

Legend: `[x]` done and tested, `[ ]` not started, `[~]` partial.

## Iteration 1 — MVP engine (current)

- [x] Reactive core: `Signal<T>`, `create_effect`, subscription tracking,
      effect teardown on drop (`creamui-reactive`, 5 unit tests)
- [x] Renderable component model: `Widget` trait (`style`/`paint`/`children`/`on_click`),
      backend-agnostic `Painter` trait (`creamui-core`)
- [x] HTML/CSS-like layout via `taffy` (flex rows/columns, CSS grid,
      gap, padding, alignment) — re-exported, not reimplemented
      (`creamui_core::layout`, widget layout tests)
- [x] Scene graph: widget tree → `taffy` layout → paint → click
      hit-testing, rebuilt each reactive render (`creamui_core::render_frame`)
- [x] Semantic theme tokens (surface/accent/text/border/danger/warning/success,
      radius\*, spacing\*) with bundled `dark()` and `light()` themes
      (`creamui-theme`)
- [x] Headless widgets: `RawView`, `RawText`, `RawButton` (no styling opinion)
- [x] Themed widgets built on the headless ones: `View`, `Text`, `Button`
      (`creamui-widgets::themed`) — pattern is copy-and-adapt for custom
      derived components
- [x] Windowing + GPU presentation on Linux: winit window, CPU rasterization
      (`tiny-skia` + `fontdue`) uploaded to a `wgpu` texture and blitted to
      the surface (`creamui-render`)
- [x] Reactive render loop: signal change → effect reruns → repaint →
      redraw request; click hit-testing dispatches to widget handlers,
      which mutate signals and trigger the next reactive re-render
- [x] `CREAMUI_DEBUG=1` verbose logging; `CREAMUI_DUMP_FRAME=<path>` frame
      dump for headless visual verification
- [x] ABI-stable C interface (`creamui-ffi`, builds as `cdylib`): opaque
      widget handles, `#[repr(C)]` options/colors, `creamui_run` with a
      C callback rebuilding the tree — verified end-to-end via a
      `libloading` test that `dlopen`s the built library
- [x] `examples/hello_world`: themed counter exercising all of the above
      (verified by running it and inspecting a dumped frame)
- [x] Test coverage: unit tests per crate + one cross-crate integration
      test (`creamui-widgets/tests/integration.rs`) driving a real
      widget tree through layout, paint, and a simulated click

## Iteration 2 — up next: foundations for scale

The important blocker to clear before a declarative/JSX layer is worth
building: right now every signal change rebuilds the *entire* widget tree
and repaints the *entire* window (see `creamui_render::window::run`'s
effect closure). Fine for a small counter or a desktop-shell widget; not
fine for anything with a non-trivial tree. Retained-tree diffing below is
the prerequisite — building `jsx!` on top of a full-rebuild engine would
just bake the perf ceiling into every app that uses it.

- [ ] Retained-tree diffing instead of full rebuild-per-render (perf) —
      **do this before Iteration 3**
- [ ] Runtime `ThemeProvider` (swap/override themes at runtime instead of
      passing a `Theme` value into every constructor)
- [ ] Proper text shaping/measurement via `taffy`'s measure/context API
      instead of the current heuristic width estimate in
      `creamui-widgets::text_metrics`
- [ ] Bundle a default font instead of probing system font paths
- [ ] More headless/themed widgets: checkbox, text input, slider, scroll view
- [ ] Keyboard input and focus handling
- [ ] DPI/scale-factor awareness (currently assumes scale factor 1.0)
- [ ] Expand the C ABI: full layout style control, more widget kinds,
      window-level operations (resize, move, always-on-top) for desktop
      shell use cases

## Iteration 3 — declarative layer: `jsx!`

Only start this once Iteration 2's diffing item is done — see above.
Reactive state stays `Signal`/`create_effect` as-is (Solid-style
fine-grained reactivity, not React hooks/fiber semantics); `jsx!` is a
syntax layer over the existing `Widget` builder pattern, not a new
reactivity model.

- [ ] `jsx!{ ... }` proc-macro: parses JSX-like syntax
      (`<View style={...}><Text>...</Text></View>`) and expands to the
      existing `creamui_widgets` builder calls — pure syntax sugar, no
      engine changes required
- [ ] `examples/jsx_hello_world` (or extend the existing hello-world):
      the same counter app rewritten with `jsx!` instead of hand-written
      builders, to prove the macro output matches hand-written trees
- [ ] Document the mapping from JSX attributes/props to `Style`/theme
      tokens (e.g. `style={{ bg: "#ff0000" }}`) so the macro's prop
      surface is predictable
- [ ] (Exploratory, not required for the above) a React-hooks-semantics
      compatibility shim on top of `Signal`, only if a future need for
      literal `useState`/`useEffect` dependency-array semantics comes up —
      not needed for `jsx!` itself

## Iteration 4 — Windows support

- [ ] `creamui-render` backend validation on Windows (winit + wgpu should
      mostly carry over; font probing needs a Windows path list)
- [ ] CI matrix covering Linux + Windows
