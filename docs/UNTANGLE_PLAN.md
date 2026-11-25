# Untangling `Editor` and `Operandi`

Plan for breaking up the two central objects that keep every area in one
dependency cycle (see [AREA_DEPENDENCIES.md](AREA_DEPENDENCIES.md)). Each
step is small, changes no behaviour, merges on its own, and ends by
rerunning `scripts/area-deps.py` and deleting the baseline lines it freed
(`--check` lists them). Steps inside a track can run one after another in
separate branches; the two tracks don't touch each other's files.

Where things stand (2026-11-25): `Editor` (`operandi-core/src/editor.rs`)
has about 125 fields, roughly 45 of them one tool's preset or state, and
`impl Editor` blocks in about 120 files (28 in `polygons/`). `Operandi`
(`operandi-app/src/main.rs`) has 336 fields and `impl Operandi` blocks in
about 230 files across 33 areas.

## Track E: `Editor` (core)

**E1. One store for tool presets.** Replace the ~45 `*_preset` fields
(`bevel_profile`, `cone_preset`, `boolean_preset` ...) with one
`presets: ToolPresets` keyed by type (`editor.preset::<ConePreset>()`),
in `tools/`. Each tool file names its own preset type; `editor.rs` stops
naming them. One branch per tool family (primitives, polygons, edges,
UV, remesh/boolean/drill) so each stays reviewable.

**E2. History out of `Editor`.** Move `undo`, `redo`, `tool_steps`,
`tool_redo_floor`, `history`, `history_mark`, `history_serial`,
`pending_records`, `undo_memory_budget`, `undo_trim_paused`, `undo_fold`
into a `History` struct in `commands/` (next to the command record),
owned by `Editor`. Callers say `editor.history.…`.

**E3. Tool session state out of `Editor`.** Same for the live tool
session (`live`, `armed`, `tweaks`, `tweak_held`, `last_transform_step`,
`gesture_partners`, `dispatch_depth` ...) into a `ToolSession` in `tools/`,
and per-tool memory (`bevel_states`, `surface_states`, `pen_draft`,
`uv_island_memory` ...) into the E1 store.

**E4. A context smaller than `Editor` for tools.** Add
`EditCtx { scene, history, session, presets }` borrowed from `Editor`.
New tool code takes `&mut EditCtx`; existing `impl Editor` blocks move to
it one area at a time, leaf areas first (`meshblend`, `deformers`,
`render`, `materials`, `sculpting`, then `uv`, `curve_tools`,
`transforms`, `polygons` last). Rule for new code from E4 on: no new
`impl Editor` outside `editor/`.

**E5. Tools register themselves.** A `ToolOps` table (activate,
preview, apply, cancel) filled by each tool file, so
`editor/preview.rs` (`preview_one`), `dispatch.rs`, `activate.rs` and
`finish.rs` stop matching on every `Tool` variant. Move one family per
branch; each deletes its match arms. `tool_speed_guard_tests.rs` and the
command contract tests are the safety net.

**E6. `Command` through the registry.** `enum Command` names every
tool's parameter types. Route tool commands through the existing command
registry handlers instead, area by area, until `Command` holds only
editor-level commands (select, undo, mode).

After E5 the `editor → tool area` links (26 areas today) should be gone;
after E4/E6 the reverse links shrink to `EditCtx`.

## Track A: `Operandi` (app)

**A1. Group the fields by area.** Move `Operandi`'s 336 fields into one
state struct per area (`ViewportUi`, `RenderUi`, `PrefsUi`, `PanelsUi`
...), each defined in its area folder and owned by `Operandi`. One branch
per area or two; purely mechanical (`self.x` becomes `self.render.x`).

**A2. A narrow app context.** Add `AppCtx` (editor, preferences, theme,
input map, document state) borrowed from `Operandi`. Area code becomes
`fn ui(state: &mut RenderUi, ctx: &mut AppCtx, ui: &mut egui::Ui)`
instead of `impl Operandi`. Leaf areas first (`deformers`, `duplication`,
`transforms`, `nodes`, `sculpting`, `edges`), hubs (`viewport`, `view`,
`window_layout`) last.

**A3. Panels register themselves.** `window_layout` calls panels through
a `Panel` table keyed by panel kind instead of naming each panel's code.
That removes `window_layout`'s links back up into `settings`, `items`,
`render` and `anim`.

**A4. Shared widgets at the bottom.** Move the bits `controls` borrows
from above (`window_layout::form` row types, `tools::numeric_tab`
helpers, `settings::units_preferences` formatting) down into `controls`,
so it uses only `theme`.

**A5. `view` under `viewport`.** Cut `view/handle_tips.rs` and
`view/slice_view.rs`'s use of `viewport`, and move the tool-specific
views (`haul_view.rs`'s panel calls, `path_tool_view.rs`,
`duplicate_view.rs`) next to their tools; then split the 3D-view giants
along the new lines.

A4 and A5 are independent of A1-A3 and can go first.

## Done means

- `core/geometry`, `core/subdivision` and `core/selections` are out of
  the core cycle; then CI's area check switches to `--strict`.
- `editor` uses no tool area; `window_layout` and `controls` use no
  feature area.
- Every step lowers the baseline line count; none raises it.
