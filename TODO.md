# TODO

Prioritized by impact.

3. **No way to test real interaction (clicks, keyboard) without a display.** Verifying render-loop behavior currently means `CUI_DUMP_FRAME` + hand-reconstructed logic in isolated scratch tests (that's how the `theme_sync` bug above was diagnosed). A harness that feeds synthetic `WindowEvent`s into a `WindowState` from a normal `#[test]` would catch this class of bug automatically.

4. **Devtools benchmark overlay has no RAM/CPU sampling on Windows.** Linux (`/proc/self/*`) and macOS (`getrusage`) are implemented; Windows falls back to "n/a". Not blocking, just incomplete.

5. **`with_cached_theme` (in `crates/widgets/src/themed.rs`) is a hand-placed workaround, not a framework guarantee.** Widgets that build children lazily inside `Widget::children`/`paint`/`measure` (which run outside `build_ui`'s context scope) — `Dialog`, `Select`, `RadioGroup`, `SegmentedControl`, `ColorPicker`, `DateTimePicker` — each re-open a context scope by hand with their own cached `Theme` snapshot before calling `use_theme()`-based constructors. It works and is tested, but the pattern is easy to forget when adding a new widget with deferred children; the render loop offering a scope that spans the whole frame (not just `build_ui`) would remove the need for it entirely.
