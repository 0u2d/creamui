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

## Iteration 2 — foundations for scale (nearly done)

The important blocker to clear before a declarative/JSX layer is worth
building: right now every signal change rebuilds the *entire* widget tree
and repaints the *entire* window (see `creamui_render::window::run`'s
effect closure). Fine for a small counter or a desktop-shell widget; not
fine for anything with a non-trivial tree. Retained-tree diffing below is
the prerequisite — building `jsx!` on top of a full-rebuild engine would
just bake the perf ceiling into every app that uses it.

- [x] Retained-tree diffing instead of full rebuild-per-render (perf):
      `creamui_core::Renderer` keeps a persistent `taffy` tree across
      frames and reconciles structurally (by position, not by widget
      identity/keys — see the doc comment on `scene::reconcile`), reusing
      node ids and only touching styles/children that actually changed
      instead of rebuilding the whole tree every render
      (`creamui-core`, 3 reconciliation unit tests)
- [x] Runtime `ThemeProvider` (swap/override themes at runtime instead of
      passing a fixed `Theme` value into every constructor): wraps a
      `Signal<Theme>`, so `.set()` triggers the same reactive re-render
      path as any other signal — no special-casing needed in widgets
      (`creamui-theme`, 2 unit tests; demoed by a theme-toggle button in
      `examples/hello_world`)
- [x] Proper text shaping/measurement via `taffy`'s real measure/context API
      (`Widget::measure`, `TaffyTree<MeasureFn>`, `compute_layout_with_measure`)
      instead of baking a heuristic width into `Style` — width now comes
      from `fontdue`'s own line-width calculation (`max_width - line.padding`),
      so it matches exactly what the renderer does at paint time instead of
      an approximation with a fudge factor (`creamui-widgets::text_metrics`,
      2 unit tests)
- [x] Bundle a default font instead of probing system font paths: DejaVu
      Sans is embedded via `include_bytes!` (`assets/fonts/`, Bitstream
      Vera license — see `assets/fonts/DejaVuSans-LICENSE.txt`) in both
      `creamui-render` and `creamui-widgets`, so text rendering no longer
      depends on what's installed on the target machine
- [x] More headless/themed widgets: `Checkbox`/`RawCheckbox`,
      `TextInput`/`RawTextInput`, `Slider`/`RawSlider`, and
      `ScrollView`/`RawScrollView` — same "caller owns the state via a
      `Signal`, widget only exposes a change callback" pattern as `Button`
      throughout (5 integration tests: toggle, focus+typing+backspace,
      proportional drag, and scroll-clipped hit-testing)
- [x] Real clipping in the render pipeline (what made `ScrollView` possible):
      `Widget::clips_children`/`scroll_offset`/`on_scroll` (`creamui-core`),
      `Painter::push_clip`/`pop_clip` (default no-op; `SkiaPainter`
      implements it with a stack of `tiny_skia::Mask`s, each already
      intersected with its parent). `Scene` intersects every widget's own
      hit/focus/drag/scroll rect against the ambient clip before
      registering it, so content scrolled out of view is provably
      unclickable, not just invisible — see the `Rect::intersect` unit
      tests and the `scroll_view_clips_hit_testing_*` integration test
- [x] Keyboard input and focus handling: `Widget::focusable`/`on_key`/
      `on_drag` (`creamui-core`), with `Scene` collecting focusable and
      draggable hit-regions alongside click hits during paint. A click
      inside a focusable widget's rect gives it focus (elsewhere clears
      it); `creamui-render` translates winit's `KeyEvent` into a
      backend-agnostic `Key` enum and dispatches it to whichever widget is
      currently focused. Indices are positional (stable only while the
      tree's shape doesn't change — no keyed focus tracking yet, same
      caveat as the retained-tree diffing above)
- [x] DPI/scale-factor awareness: widgets are laid out and painted in
      logical pixels; `creamui-render` tracks the window's `scale_factor`
      as its own `Signal` (reacting to `WindowEvent::ScaleFactorChanged`),
      converts pointer coordinates from physical to logical for hit-testing,
      and `SkiaPainter` scales every paint call so the backing pixmap/GPU
      texture stay sized in physical pixels for crisp HiDPI output
- [x] Expose reactive state across the C ABI (`creamui_signal_i32_*`): a
      dynamically-linked app has no Rust-side `Signal`, so without this a
      C click handler had no way to trigger a re-render at all — verified
      by `examples/hello_world_dynamic`, a working counter driven entirely
      through the ABI
- [x] `examples/hello_world_dynamic`: the same counter as `hello_world`,
      but linked dynamically — zero CreamUI crate dependencies, resolves
      every function via `libloading` at runtime, proving static vs.
      dynamic linking is the app's choice, not baked into the engine
- [x] Expand the C ABI further: theme-token access (`CTheme` +
      `creamui_theme_dark`/`creamui_theme_light`, threaded through
      `creamui_themed_text_new`/`creamui_button_new`/etc. instead of a
      hardcoded `Theme::dark()` — `hello_world_dynamic` now demos the same
      runtime theme toggle the static example does), full layout style
      control (`CStyle`/`CDimension`, `creamui_view_new_styled` and
      style params on the text-input/slider/scroll-view constructors,
      covering flex direction/alignment/size/padding/margin/gap/flex-grow-
      shrink-basis), more widget kinds (`creamui_checkbox_new`,
      `creamui_text_input_new`, `creamui_slider_new`,
      `creamui_scroll_view_new`/`creamui_scroll_view_add_child`),
      window-level operations (`creamui_run`'s new `on_window_ready`
      callback hands back a `CWindowHandle` for `creamui_window_resize`/
      `creamui_window_set_position`/`creamui_window_set_always_on_top`,
      backed by a new `creamui_render::WindowHandle`) — verified by new
      `creamui-ffi` `dlopen` tests and `hello_world_dynamic`'s theme-toggle
      and always-on-top buttons

## Iteration 3 — declarative layer: `jsx!`

Only start this once Iteration 2's diffing item is done — see above.
Reactive state stays `Signal`/`create_effect` as-is (Solid-style
fine-grained reactivity, not React hooks/fiber semantics); `jsx!` is a
syntax layer over the existing `Widget` builder pattern, not a new
reactivity model.

- [x] `jsx!{ ... }` proc-macro (`creamui-macros`): parses JSX-like syntax
      (`<View theme={&theme} style={...}><Text theme={&theme}>"..."</Text></View>`)
      and expands to the existing `creamui_widgets` builder calls — pure
      syntax sugar, no engine changes required; its integration test renders
      the output and dispatches a click through the resulting scene
- [x] Extensible component protocol: application components need no macro
      registry. `#[component] fn MyComponent(...) -> BoxedWidget` generates
      `MyComponentProps`, so an imported `<MyComponent prop={...}/>` becomes
      a statically checked Rust call; `creamui-jsx::IntoWidget` accepts both
      native widgets and component-function results as children
- [x] `examples/jsx_hello_world`:
      the same counter app rewritten with `jsx!` instead of hand-written
      builders, to prove the macro output matches hand-written trees
- [x] Document the mapping from JSX attributes/props to `Style`/theme
      tokens in `README.md`: `style` is an ordinary Rust `Style` expression,
      while themed component props take an explicit `theme={&theme}`; this
      retains the full Taffy API without inventing a second style language
- [ ] (Exploratory, not required for the above) a React-hooks-semantics
      compatibility shim on top of `Signal`, only if a future need for
      literal `useState`/`useEffect` dependency-array semantics comes up —
      not needed for `jsx!` itself

## Iteration 4 — Windows support

- [~] `creamui-render` backend validation on Windows (winit + wgpu should
      mostly carry over; the bundled font removes what used to be a
      Windows-specific font-path gap): reviewed the codebase for
      Windows-portability landmines and found none needing a fix — `winit`/
      `wgpu`/`tiny-skia`/`fontdue` are all cross-platform, the GPU backend
      is picked automatically by `wgpu::Instance::new` (DX12/Vulkan on
      Windows) with no OS-specific code path, the font is embedded via
      `include_bytes!` rather than probed from OS font paths, and the only
      pre-existing `cfg!(target_os = ...)` branches (picking `creamui.dll`
      vs. `libcreamui.so`/`.dylib` in the dynamic-loading examples/tests)
      already handle Windows. What's *not* done: this was validated by
      reading, not by running on Windows — there's no Windows machine in
      this environment, so nothing here has actually built or run there
      yet; the CI matrix below is what closes that gap on the next push
- [x] CI matrix covering Linux + Windows: `.github/workflows/ci.yml` builds
      and tests the whole workspace on `ubuntu-latest` and `windows-latest`
      on every push/PR to `main` — this is the first real Windows build the
      engine will get, so treat its first run as the actual validation for
      the item above, not this checkbox
