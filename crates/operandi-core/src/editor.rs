use crate::{Mode, Result, Scene};
use glam::DVec3;

mod activate;
mod command;
mod context;
mod dispatch;
mod finish;
mod history;
mod parameters;
mod tool;

pub use command::Command;
pub use parameters::Parameters;
pub use tool::Tool;

#[derive(Clone, PartialEq)]
pub struct Live {
    pub identity_base: Option<crate::IdentityScene>,
    pub identity_preview: Option<(crate::EditCorrespondence, crate::MeshIdentity)>,
    pub tool: Tool,
    pub base: Scene,
    pub params: Parameters,
    pub applied: bool,
    pub action_frame: Option<crate::ActionFrame>,
    /// ACS-07: a centre the user placed (a viewport click or a typed
    /// centre, `SetActionCenter`), world space; `None` while the pivot
    /// service places it.
    pub placed_centre: Option<DVec3>,
    pub preview_error: Option<String>,
}
#[derive(Clone, PartialEq)]
pub struct Checkpoint {
    pub identity_before: Option<crate::IdentityScene>,
    pub identity_after: Option<crate::IdentityScene>,
    pub label: String,
    pub before: Scene,
    pub after: Scene,
    /// The gesture this step belongs to (`Editor::undo_fold`).
    pub fold: Option<u64>,
    /// The command lines that made this step (UIC-226).
    pub records: crate::StepRecords,
    /// `Editor::history_mark` right after this step was recorded: which
    /// steps are newer than a mark, however trimming shifted the list.
    pub mark: u64,
}
/// Most memory the undo history takes (`trim_undo_memory`), counting the
/// mesh data each step alone keeps (undo_memory.rs).
pub const UNDO_MEMORY_BUDGET: usize = 4 << 30;
/// Merge Tool's Distance after Reset (recorded, VTO-D0-05).
pub const MERGE_TOOL_DISTANCE: f64 = 0.001;
/// Where the editor asks how much memory is free (bytes). Not editor
/// state: two editors compare equal whatever their sources.
#[derive(Clone, Copy, Debug, Default)]
pub struct FreeMemory(pub Option<fn() -> u64>);
impl PartialEq for FreeMemory {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct Editor {
    /// The undo memory budget in bytes; `None` is `UNDO_MEMORY_BUDGET`.
    pub undo_memory_budget: Option<usize>,
    /// Set while a multi-mesh command records one step per mesh that it
    /// then joins into one (multi_tools.rs): trimming in between would
    /// shift the steps it is about to join (Astra P9-001).
    pub undo_trim_paused: bool,
    /// While set, undo steps recorded under this gesture id fold into one
    /// (a colour drag across several items and frames is one step). The
    /// app sets it around a gesture's commands and clears it after.
    pub undo_fold: Option<u64>,
    /// Meshes a tool or command has just run on as one of several active
    /// meshes (multi_tools.rs): their components were written by that
    /// tool for the new geometry, so they survive the topology check in
    /// `prune_component_selections`. Kept while a live tool is open;
    /// cleared after every other command.
    pub multi_fresh: std::collections::BTreeSet<u64>,
    /// Select > Polygon Tag Type: what tag selection modes select by.
    pub polygon_tag_kind: crate::PolygonTagKind,
    pub pen_draft: Option<crate::PenDraft>,
    /// What the open tool's start inferred from another selection mode
    /// (selection_inference.rs), if anything.
    pub inference: Option<crate::selection_inference::Inference>,
    /// A one-off Symmetry message from the last creation (SYM3-25, e.g.
    /// "Crosses the symmetry plane: not mirrored"); the app shows it in the
    /// status bar and clears it.
    pub symmetry_note: Option<String>,
    /// The Symmetry Tool's points still without a mirror partner, in the
    /// active mesh (the viewport marks them).
    pub symmetry_unpaired: Vec<u32>,
    /// The setting Toggle Symmetry turns back on (the last one before Off).
    pub symmetry_resume: Option<crate::Symmetry>,
    /// Mirror partners added during the open selection gesture (lasso,
    /// rectangle, paint), moved after the swept elements when it ends
    /// (spec v3.1 section 11: swept elements first, then partners).
    pub gesture_partners: Vec<(u64, crate::Component)>,
    /// Interactive Pen options, kept between Pen sessions.
    pub pen_options: crate::PenOptions,
    /// The open Solid Sketch session (solid_sketch_editor.rs).
    pub solid_sketch: Option<crate::SketchSession>,
    /// Solid Sketch options, kept between sessions (spec Q18).
    pub solid_sketch_options: crate::solid_sketch::SketchOptions,
    pub i2: crate::LiveExtrudeHistory,
    pub identities: crate::IdentityRuntime,
    pub bevel_states: std::collections::BTreeMap<String, crate::BevelRecipe>,
    pub bevel_state: Option<String>,
    pub bevel_profile: Option<crate::BevelProfile>,
    /// The active falloff (FAL-M-01); `None` is Falloff None.
    pub spatial_falloff: Option<crate::Falloff>,
    /// Constrain to Background, when on (background_constraint.rs). Tool
    /// context like the falloff: not saved, not an undo step.
    pub background_constraint: Option<crate::background_constraint::BackgroundConstraint>,
    /// Its last options, offered when it is turned on again.
    pub background_constraint_preset: crate::background_constraint::BackgroundConstraint,
    /// The view the last press happened in (Screen Axis searches along it).
    pub background_view: crate::background_constraint::LastView,
    /// The background meshes' surface, kept while they don't change.
    pub background_cache: crate::background_constraint::BackgroundCache,
    /// Topology Pen's options for the session: a new activation starts
    /// from them (TP-10, values stick within the session).
    pub topology_pen_memory: crate::TopologyPenOptions,
    /// Falloffs added on top of the active one, each with its Mix Mode
    /// (VMF-10, VMF-11). Picking another falloff type clears them.
    pub added_falloffs: Vec<(crate::Falloff, crate::MixMode)>,
    /// The active falloff's own Mix Mode (FO-11). Shown on every layer;
    /// on the first layer it has no effect (FO-32, measured). Picking
    /// another type resets it.
    pub spatial_falloff_mix: crate::MixMode,
    /// The Element falloff's pick (FO-45): the element a press on the
    /// model grabbed. Tool context; cleared when the falloff changes.
    pub falloff_element: Option<crate::TweakGrab>,
    /// Screen, Lasso and Airbrush weights per (stack layer, item), worked
    /// out by the app when drawn or painted (FO-Q08). Tool context.
    pub falloff_stamps: std::collections::BTreeMap<(usize, u64), std::sync::Arc<Vec<f64>>>,
    /// The Path falloff's control points (FO-53), in its space. Tool
    /// context.
    pub falloff_path: Vec<glam::DVec3>,
    /// The selected weight map's name (VM-02): the Vertex Map Tools paint
    /// it and the Vertex Map falloff reads it. Tool context, not scene
    /// data.
    pub selected_weight_map: Option<String>,
    /// An open tool preset row (Soft Move, Twist, Flex, Flare ...): its
    /// live tool is the row's tool, and the pipe slots it took go back on
    /// drop (tool_preset.rs).
    pub preset: Option<crate::PresetSession>,
    /// Last Linear and Radial settings, offered on reactivation (FAL-M-03).
    pub linear_falloff_preset: crate::LinearFalloff,
    pub radial_falloff_preset: crate::RadialFalloff,
    pub smooth_shift_preset: crate::SmoothShiftParameters,
    pub chamfer_preset: crate::ChamferParameters,
    pub edge_bevel_preset: crate::EdgeBevelPreset,
    pub edge_chamfer_preset: crate::EdgeBevelPreset,
    pub poly_haul_preset: f64,
    pub sketch_precision_preset: Option<f64>,
    /// Merge Tool's last Distance; `None` is the recorded default 0.001
    /// (VTO-T04).
    pub vertex_merge_preset: Option<f64>,
    /// BN-47: Bend's values from its last use (Angle, Spine, axis, Both
    /// Sides, Deform Space); `None` until it has been dropped once.
    pub bend_preset: Option<crate::BendOptions>,
    /// CS-01 (click_start.rs): honour the tools' Auto-Activate flags, so a
    /// tool flagged off only arms when picked. The app turns it on; off,
    /// every tool starts at once as before (scripts, staged one-shots).
    pub auto_activate_flags: bool,
    /// CS-01: the live tool is armed: picked, values editable, nothing
    /// drawn and the mesh as it was, until the first click or Apply.
    pub armed: Option<Tool>,
    /// N25: Edge Extend was started by a click, so its rows exist even at
    /// zero offset (click_start.rs).
    pub extend_from_click: bool,
    /// How deep inside `dispatch` the editor is: arming and the armed
    /// rules act only on top-level commands (click_start.rs).
    #[doc(hidden)]
    pub dispatch_depth: u32,
    /// 3D painting's saved-image record and last-made image and layer.
    #[cfg(feature = "painting")]
    pub paint: crate::paint::PaintSession,
    pub thicken_preset: Option<crate::SmoothShiftParameters>,
    pub surface_states: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, crate::SmoothShiftParameters>,
    >,
    pub surface_state: std::collections::BTreeMap<String, String>,

    pub bridge: Option<crate::NumericBridge>,
    pub bridge_preset: crate::BridgeParameters,
    pub spike: Option<crate::NumericSpike>,
    pub spike_strength: f64,
    /// MP-T11 Polygon Reduction session; `None` when the tool is off.
    pub reduction: Option<crate::NumericReduction>,
    pub ellipsoid: Option<crate::NumericEllipsoid>,
    pub ellipsoid_preset: crate::EllipsoidParameters,
    /// Torus, Capsule, Arc or Tube numeric session (NPR), and each one's
    /// last settings.
    pub shape: Option<crate::NumericShape>,
    pub shape_presets: crate::ShapePresets,
    pub cone: Option<crate::NumericCone>,
    pub cone_preset: crate::ConeParameters,
    pub scene: Scene,
    pub action_context: crate::ActionContext,
    /// Shared Symmetry setting (SYM-ST-01); tool context, not scene data.
    pub symmetry: crate::Symmetry,
    pub live: Option<Live>,
    /// The document's key time, channel/key selections and playback
    /// (animation.rs; per scene tab).
    pub anim: crate::animation::AnimSession,
    /// RG-11/RG-30 Compensation: rig edits keep items where they are in
    /// world space (off by default; the app keeps it in its preferences).
    pub rig_compensation: bool,
    /// Undo and redo steps, shared: the editor is copied for every command
    /// (identity_editor.rs stages each one), and copying each step's two
    /// scenes made a command in a 100,000-item scene take seconds. Change a
    /// step with `Arc::make_mut` (scene-budget, 2026-11-20).
    pub undo: Vec<std::sync::Arc<Checkpoint>>,
    pub redo: Vec<std::sync::Arc<Checkpoint>>,
    /// Steps recorded since the active tool was activated, for tools that
    /// keep running through Undo/Redo (`mid_tool_undo.rs`). `None` when no
    /// such tool is active or its activation isn't known (re-armed by Undo).
    pub tool_steps: Option<usize>,
    /// The tool Undo just closed, which Redo opens again.
    pub tool_reopen: Option<crate::ToolReopen>,
    /// The active tool's own tweak history (`mid_tool_undo.rs`).
    pub tweaks: crate::ToolTweaks,
    /// The transform drag committed last, for Shift-continue (HM-02) and
    /// middle-drag repeat (HM-04). Not document state.
    pub last_transform_step: Option<Box<crate::transform_repeat::TransformStep>>,
    /// Set by the app while a drag or scrub is held, so its changes are one
    /// tweak; `end_tweak` clears it.
    pub tweak_held: bool,
    /// Counts recorded document steps, to tell a tweak from an Apply.
    pub history_serial: u64,
    /// Counts every change recording makes to the undo history: a new
    /// step, a later edit folded into the newest one, or a selection change
    /// joining a selection group (`recorded_since`). Unlike `undo.len()`, it
    /// still moves once Max Undo Level trims the oldest step.
    pub history_mark: u64,
    /// Redo depth when the active tool started; leaving the tool drops the
    /// steps undone above it.
    pub tool_redo_floor: Option<usize>,
    /// Undo preferences (Max Undo Level, selection grouping); set by the app.
    pub history: crate::HistorySettings,
    /// Command records waiting for the next undo step (UIC-226): the app
    /// adds a line before running it, and a tool session's lines (tool on,
    /// `tool.attr`) wait here until its drop. The next recorded step takes
    /// them all; the app drops them when no step came of them.
    pub pending_records: crate::StepRecords,
    /// Selection changes merged into the newest undo step so far; 0 when
    /// that step is not a selection group.
    pub selection_run: u32,
    pub revision: u64,
    pub next_item_id: u64,
    /// Boxed to keep `Editor` small: the app state is near the stack limit.
    pub selection_start: Option<Box<Scene>>,
    pub extrude_preset: Parameters,
    pub extrude_moved_caps: Vec<u32>,
    /// Extrude on several meshes: each mesh's moving caps and the caps it
    /// selects on Drop (multi_tools.rs). Boxed like `selection_start`.
    pub multi_extrude: Option<Box<crate::multi_tools::MultiExtrude>>,
    pub cube_preset: Parameters,
    /// Last dropped Sides/Segments, used by the next activation and unit
    /// insertion. Not scene data.
    pub sphere_unit_resolution: Option<[u32; 2]>,
    pub cylinder_unit_resolution: Option<[u32; 2]>,
    /// Last dropped Cylinder caps (bottom, top) and axis.
    pub cylinder_shape: Option<([crate::CylinderCap; 2], usize)>,
    /// Last dropped Sphere axis and mode.
    pub sphere_shape: Option<(usize, Option<u32>)>,
    /// Make UVs turned off at the last Drop of Cube, Sphere, Cylinder (in
    /// that order). Stored as "off" so the default is on.
    pub primitive_uvs_off: [bool; 3],
    /// Cut/Copy's app-wide clipboard (clipboard.rs); not scene data and
    /// not undone.
    pub clipboard: Option<Box<crate::Clipboard>>,
    /// The Slice tools' options from their last session.
    pub slice_preset: crate::SliceParameters,
    /// Memory free for the app now, in bytes; Subdivide refuses a result
    /// that needs more (`Editor::subdivide_estimate`). The app sets it;
    /// without it (scripts, tests) nothing is checked.
    pub free_memory: FreeMemory,
    /// Subdivide Polygons' last-used options (SUB-05): the buttons set only
    /// the Method and reuse the rest.
    pub subdivide_preset: crate::SubdivideOptions,
    /// UnSubdivide's last-used options (SUB-40).
    pub unsubdivide_preset: crate::UnsubdivideOptions,
    /// Freeze's last-used options (FRZ-03): what a script's `poly.freeze`
    /// starts from (the app's dialog keeps its own copy too).
    pub freeze_preset: crate::freeze::FreezeOptions,
    /// The Duplicate tools' settings from their last session.
    pub duplicate_preset: Box<crate::DuplicateParameters>,
    /// Detriangulate's last-used options (DT-10, UIC-95).
    pub detriangulate_preset: crate::DetriangulateOptions,
    /// Relax Mesh's last-used options (UIC-95).
    pub relax_mesh_preset: crate::RelaxMeshOptions,
    /// UV Unwrap's and Pack UVs' last-used values (the Pack dialog keeps
    /// them between uses, UV-Q10).
    pub uv_unwrap_preset: crate::uv_tools::UnwrapOptions,
    /// Atlas Unwrap's last settings (uv_atlas.rs).
    pub uv_atlas_preset: crate::uv_atlas::AtlasOptions,
    /// The UV map clipboard: a whole map copied from the map list (UVE-1
    /// section 1).
    pub uv_map_clipboard: Option<crate::uv_maps::UvMapClipboard>,
    /// The E7 dialog values plain clicks use (UVT-08, uv_edit.rs).
    pub uv_island_memory: crate::uv_edit::IslandMemory,
    /// The UV clipboard of Copy / Cut / Paste UVs (UVT-90, uv_ops.rs).
    pub uv_clipboard: Option<crate::uv_ops::UvClipboard>,
    /// The Texel Density panel's values (UVT §7.9), kept between uses.
    pub uv_texel: crate::uv_ops::TexelOptions,
    /// The Cut Map popover's UV Seam: the seam map its buttons use (""
    /// is `uv_seams::DEFAULT_SEAM_MAP`).
    pub uv_seam_map: String,
    /// The UV Symmetry layer (UVT-70): mirrors UV edits in a UV view.
    pub uv_symmetry: crate::uv_symmetry::UvSymmetry,
    pub uv_pack_preset: crate::uv_tools::PackOptions,
    /// Automatic Retopology's, Voxel Remesh's and the octree Remesh's
    /// last-used options (remesh.rs).
    pub retopo_preset: crate::remesh::RetopoOptions,
    pub voxel_remesh_preset: crate::remesh::VoxelOptions,
    pub octree_remesh_preset: crate::remesh::OctreeOptions,
    /// A remesh the app worked out on a worker thread, used by the next
    /// remesh command when it matches (`remesh::RemeshPrepared`).
    pub remesh_prepared: Option<Box<crate::remesh::RemeshPrepared>>,
    /// Boolean's last-used options (BOOL-23).
    pub boolean_preset: crate::BooleanOptions,
    /// What the last successful Boolean wants on the status line.
    pub boolean_note: Option<String>,
    /// Mesh Stack results (mesh_stack.rs): a cache shared by clones, never part
    /// of the scene (SAFE-03).
    pub mesh_stack: crate::mesh_stack::Runtime,
    /// The scene's evaluation graph (eval_state.rs): a cache, never part
    /// of the scene or undo (AR-04, AR-15).
    pub eval: crate::eval_state::EvalState,
    /// The mesh in Edit Base Mesh (MO-27): direct tools edit its Base Mesh.
    pub base_edit: Option<u64>,
    /// The selected row of the Mesh Operations list.
    pub meshop_row: Option<crate::mesh_stack_editor::OpRow>,
    /// Solid Drill's and Axis Drill's last-used options (DRILL-08).
    pub solid_drill_preset: crate::DrillOptions,
    pub axis_drill_preset: crate::DrillOptions,
    /// What the last successful drill wants on the status line.
    pub drill_note: Option<String>,
    /// What the last headless command that writes a file (a bake) wants on
    /// the status line; the app shows and clears it.
    pub command_note: Option<String>,
    /// What the last Reset or Freeze skipped, for the status line
    /// (item_transform_ops.rs, spec section 7).
    pub transform_note: Option<String>,
}
pub const BACKDROP_NO_GEOMETRY: &str =
    "A Backdrop has no geometry to edit; select a mesh, or use Move, Rotate or Scale on it";
pub const LOCATOR_NO_GEOMETRY: &str = "Cameras, lights and locators have no geometry; edit them in Item mode with Move, Rotate or Scale";
fn backdrop_mut(scene: &mut Scene, id: u64) -> Result<&mut crate::backdrop::Backdrop> {
    scene
        .item_mut(id)
        .and_then(|i| i.backdrop.as_mut())
        .ok_or_else(|| "Select a Backdrop".into())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitDestination {
    ExistingMesh,
    NewMesh,
}
impl Editor {
    /// A live Cube, Sphere or Cylinder: like the transform tools it keeps
    /// running through Undo/Redo and restarts on the restored scene.
    fn live_primitive_tool(&self) -> Option<Tool> {
        self.live
            .as_ref()
            .map(|l| l.tool)
            .filter(|t| t.is_primitive())
    }

    /// Throw away a primitive's preview that isn't a step yet, keeping the
    /// tool active. Numeric sessions keep their values; a drawn Cube, Sphere
    /// or Cylinder restarts empty, as after activation. Returns the tool's
    /// name, or None when no such preview shows.
    fn discard_primitive_preview(&mut self, numeric_only: bool) -> Result<Option<String>> {
        let Some(label) = self.primitive_session().and_then(|s| s.pending_preview) else {
            return Ok(None);
        };
        if numeric_only && self.live.is_some() {
            return Ok(None);
        }
        if let Some(s) = &mut self.ellipsoid {
            s.preview = None;
        } else if let Some(s) = &mut self.shape {
            s.preview = None;
        } else if let Some(s) = &mut self.cone {
            s.preview = None;
        } else if let Some(l) = self.live.take() {
            self.scene = l.base;
            if let Some(ids) = l.identity_base {
                self.identities.current = ids;
                self.identities.observe(&self.scene)?;
            }
            self.resume_transform(l.tool);
        }
        Ok(Some(label.to_string()))
    }

    /// After Undo/Redo changed the scene under a numeric primitive session:
    /// keep it on its mesh when that mesh can still take the shape, close
    /// it otherwise (MTU-Q6, provisional). Its values stay; nothing is
    /// previewed until a value changes.
    fn rebase_numeric_primitive(&mut self) {
        if let Some(s) = &self.ellipsoid
            && self.ellipsoid_target_ok(s.target).is_err()
        {
            self.ellipsoid = None;
        }
        if let Some(s) = &self.shape
            && self.shape_target_ok(s.target, s.kind()).is_err()
        {
            self.shape = None;
        }
        if let Some(s) = &self.cone
            && !self
                .scene
                .items
                .iter()
                .any(|i| i.id == s.target && self.scene.shows(i))
        {
            self.cone = None;
        }
        for preview in [
            self.ellipsoid.as_mut().map(|s| &mut s.preview),
            self.shape.as_mut().map(|s| &mut s.preview),
            self.cone.as_mut().map(|s| &mut s.preview),
        ]
        .into_iter()
        .flatten()
        {
            *preview = None;
        }
    }

    /// A live transform tool shows a change not yet recorded as a step
    /// (a typed field value, or a drag still in progress).
    pub fn transform_change_pending(&self) -> bool {
        self.live
            .as_ref()
            .is_some_and(|l| l.tool.is_transform() && l.base != self.scene)
    }

    fn live_transform_tool(&self) -> Option<Tool> {
        self.live
            .as_ref()
            .map(|l| l.tool)
            .filter(|t| t.is_transform())
    }

    /// Record the live transform tool's change since its last step as one
    /// undo step, then keep the tool live from the result with zero offset.
    /// The center follows a move so the handle stays on the moved
    /// selection. Provisional (Jason, 2026-11-03; HND-Q03): the tool's
    /// fields restart from zero after each drag.
    pub(crate) fn commit_transform_step(&mut self) -> Result<()> {
        let live = self
            .live
            .as_ref()
            .filter(|l| l.tool.is_transform())
            .ok_or("No active transform tool")?;
        if let Some(error) = &live.preview_error {
            return Err(format!("Correct the current transform first: {error}"));
        }
        if live.base == self.scene {
            return Ok(());
        }
        self.scene.validate()?;
        self.last_transform_step = Some(Box::new(crate::transform_repeat::TransformStep {
            tool: live.tool,
            base: live.base.clone(),
            params: live.params.clone(),
            action_frame: live.action_frame,
            identity_base: live.identity_base.clone(),
            applied: live.applied,
            after: self.scene.clone(),
        }));
        let mut next = live.clone();
        if matches!(live.tool, Tool::Move | Tool::Transform)
            && let Some(frame) = next.action_frame.as_mut()
        {
            frame.center += frame.basis * live.params.delta;
        }
        next.params.delta = if live.tool == Tool::Scale {
            DVec3::ONE
        } else {
            DVec3::ZERO
        };
        next.params.rotation_delta = DVec3::ZERO;
        next.params.scale_delta = DVec3::ONE;
        // Box Transform (UVT-43): the next drag starts from the moved box;
        // a set pivot moves with it.
        let b = live.params.uv_box;
        next.params.uv_box = crate::uv_box::BoxTransform {
            pivot: b.pivot.map(|p| p + b.offset),
            show_rotation_handle: b.show_rotation_handle,
            snap_angle: b.snap_angle,
            ..Default::default()
        };
        next.params.tweak = None;
        next.params.tweak_plan = None;
        // TK-30: the drag's edit keys when it ends, not only on Drop.
        let base = std::mem::take(&mut next.base);
        self.auto_key_from(&base);
        next.base = self.scene.clone();
        next.applied = false;
        // The drag stays a tweak of the session (Jason, 2026-11-04; this
        // replaces TWK-24's step per Tweak drag); Drop records the session
        // as one step.
        next.identity_base = Some(self.identities.current.clone());
        next.identity_preview = None;
        self.live = Some(next);
        Ok(())
    }

    /// After Undo/Redo with a transform tool live: start it again on the
    /// restored scene (its center comes from the restored selection). If
    /// the restored scene can't host the tool it stays dropped.
    fn resume_transform(&mut self, tool: Tool) {
        if self.live.is_none() {
            let _ = self.handle_activate(tool);
        }
    }

    /// Change the action center and axis.
    fn handle_action_context(&mut self, context: crate::ActionContext) -> Result<()> {
        if self.cone.is_some() {
            return Err("Numeric Cone uses world coordinates; context changes during this session remain uncharacterized".into());
        }
        // Resolve before publishing context so invalid Selection requests
        // preserve the live stage, previous context and history atomically.
        let frame = self.action_context_frame(context)?;
        let previous_context = self.action_context;
        let previous_frame = self.live.as_ref().and_then(|live| live.action_frame);
        let previous_bend_center = self.live.as_ref().map(|live| live.params.bend.base);
        let parameters = self
            .live
            .as_ref()
            .filter(|live| {
                live.applied
                    && matches!(
                        live.tool,
                        Tool::Move
                            | Tool::Rotate
                            | Tool::Scale
                            | Tool::Transform
                            | Tool::EdgeExtend
                    )
            })
            .map(|live| live.params.clone());
        self.action_context = context;
        if let Some(live) = &mut self.live {
            live.action_frame = frame;
            live.placed_centre = None;
        }
        // Bend: its centre is the action centre, so a new centre mode moves
        // it (and re-bends from there once applied).
        let parameters = parameters.or_else(|| {
            let live = self.live.as_mut().filter(|l| l.tool == Tool::Bend)?;
            let center = live.action_frame?.center;
            live.params.bend.base = center;
            live.applied.then(|| live.params.clone())
        });
        if let Some(parameters) = parameters
            && let Err(error) = self.preview(parameters, false)
        {
            // Transform preview validates its local candidate before
            // publishing scene/params. Restore only the frame/context
            // staged above; never clone or commit the undo history.
            self.action_context = previous_context;
            if let Some(live) = &mut self.live {
                live.action_frame = previous_frame;
                if let Some(center) = previous_bend_center {
                    live.params.bend.base = center;
                }
            }
            return Err(error);
        }
        Ok(())
    }

    /// Select the polygons of a named polygon set.
    fn handle_select_polygon_set(&mut self, name: String) -> Result<()> {
        self.require_idle()?;
        let item = self.scene.active().ok_or("Select a mesh")?;
        if !self.scene.shows(item) {
            return Err("Show the mesh before recalling components".into());
        }
        let members = item
            .mesh
            .attributes
            .polygon_sets
            .get(&name)
            .ok_or("Named polygon set not found")?;
        let ids = self
            .identities
            .current
            .meshes
            .get(&item.id)
            .ok_or("Missing identity view")?;
        let faces = ids
            .logical_faces
            .iter()
            .filter_map(|id| ids.faces.iter().position(|x| x == id))
            .filter(|i| members.contains(&(*i as u32)))
            .map(|i| i as u32)
            .collect();
        let before = self.scene.clone();
        self.scene.selection.mode = Mode::Polygon;
        self.scene.selection.faces = faces;
        self.scene.selection.vertices.clear();
        self.scene.selection.edges.clear();
        if before != self.scene {
            self.record_selection("Selection".into(), before);
        }
        Ok(())
    }

    /// A camera, light or Planar locator channel edited from the live
    /// item tool (CL-14, planar_texture.rs). Kept out of `dispatch` so its
    /// scene copies don't grow that recursive function's frame.
    #[inline(never)]
    fn tool_item_channel(
        &mut self,
        item: u64,
        edit: &crate::camera_light::ItemChannelEdit,
    ) -> Result<()> {
        use crate::camera_light::ItemChannelEdit;
        let refusal = || -> String {
            if matches!(edit, ItemChannelEdit::Transform(_)) {
                "Use Move, Rotate, Scale or Transform on the Texture Locator".into()
            } else {
                "Use Move, Rotate or Transform on the camera or light".into()
            }
        };
        let live = self.live.as_ref().ok_or_else(refusal)?;
        let tool_ok = match edit {
            ItemChannelEdit::Transform(_) => live.tool.transforms_items(),
            _ => crate::camera_light::tool_shows_item_channels(live.tool),
        };
        if !tool_ok
            || self.scene.selection.mode != Mode::Item
            || self.scene.selection.item != Some(item)
        {
            return Err(refusal());
        }
        // Both the scene and the tool's base, so the tool's next
        // preview (built from its base) keeps the change.
        let mut scene = Box::new(self.scene.clone());
        scene.apply_item_channel(item, edit)?;
        let mut base = Box::new(live.base.clone());
        match edit {
            ItemChannelEdit::Transform(t) => {
                // PD-Q06: one item only; several selected move about a
                // shared pivot, which this doesn't invert.
                if live.base.item_transform_targets().is_some() {
                    return Err("Select only the Texture Locator to drag its projection".into());
                }
                let b = crate::planar_texture::base_for_tool(t, live.tool, &live.params)?;
                base.apply_item_channel(item, &ItemChannelEdit::Transform(b))?;
            }
            _ => base.apply_item_channel(item, edit)?,
        }
        scene.validate()?;
        self.scene = *scene;
        if let Some(live) = self.live.as_mut() {
            live.base = *base;
        }
        Ok(())
    }

    /// Re-target a live transform tool to a new item selection (see the
    /// route in `dispatch`). Kept out of `dispatch` so the staged copy
    /// doesn't grow every frame of that recursive function.
    #[inline(never)]
    fn retarget_transform(&mut self, tool: Tool, cmd: Command) -> Result<()> {
        let live = self.live.as_ref().ok_or("No live tool")?;
        let negative_scale = live.params.negative_scale;
        let tweak_options = live.params.tweak_options;
        let mut staged = Box::new(self.clone());
        staged.dispatch(Command::Drop)?;
        if staged.has_active_task() || staged.i2_active() {
            return Err("Finish the current tool before changing the item selection".into());
        }
        staged.dispatch(cmd)?;
        if staged.dispatch(Command::Activate(tool)).is_ok()
            && let Some(live) = staged.live.as_mut()
        {
            live.params.negative_scale = negative_scale;
            live.params.tweak_options = tweak_options;
        }
        *self = *staged;
        Ok(())
    }

    /// Select a scene item (or none).
    fn handle_select_item(&mut self, id: Option<u64>) -> Result<()> {
        self.require_idle()?;
        if id.is_some_and(|id| !self.scene.item(id).is_some()) {
            return Err("Missing mesh item".into());
        }
        self.scene.select_item(id);
        Ok(())
    }

    /// Sketch Extrude along an explicit world path in one step.
    fn handle_sketch_extrude_once(&mut self, path: Vec<DVec3>) -> Result<()> {
        if self.live.is_some() {
            return Err("Finish or Cancel the active tool first".into());
        }
        if path.len() < 2 {
            return Err("Sketch Extrude requires a nonzero path".into());
        }
        let mut staged = self.clone();
        staged.dispatch(Command::Activate(Tool::SketchExtrude))?;
        let mut params = staged.live.as_ref().unwrap().params.clone();
        params.sketch_path = path;
        staged.dispatch(Command::Preview(params))?;
        staged.dispatch(Command::Drop)?;
        *self = staged;
        Ok(())
    }

    /// Change the live Extrude stage's numeric parameters.
    fn handle_configure_extrude(&mut self, p: Parameters) -> Result<()> {
        let live = self.live.as_ref().ok_or("Arm Extrude first")?;
        if live.tool != Tool::Extrude {
            return Err("Arm Extrude first".into());
        }
        if p.extrude_uvs != live.params.extrude_uvs
            && (!p.extrude_uv_selected
                || !p.extrude_uv_map.as_ref().is_some_and(|name| {
                    self.scene
                        .active()
                        .is_some_and(|i| i.mesh.uv_maps.contains_key(name))
                }))
        {
            return Err("Select a UV map before changing Make UVs".into());
        }
        if !p.extent.is_finite() || p.extrude_sides == 0 || p.extrude_sides > 1_000_000 {
            return Err("Invalid Extrude numeric configuration".into());
        }
        if live.applied {
            self.preview(p.clone(), false)?;
        } else {
            self.live.as_mut().unwrap().params = p.clone();
        }
        self.extrude_preset = p;
        Ok(())
    }
    /// Stage the drop in a clone. The live task, history and revision remain
    /// owned by this editor until the atomic filesystem write succeeds.
    pub fn save_committed(&mut self, path: &std::path::Path) -> Result<()> {
        if self.live.as_ref().is_some_and(|l| {
            l.params
                .radial
                .is_some_and(|p| p.radius.min_element() == 0.)
        }) {
            return Err("Flat Sphere/Cylinder draft has no committed topology; enter positive Radius values or Cancel before Save".into());
        }
        let mut staged = self.clone();
        staged.dispatch(Command::Drop)?;
        if let Err(e) = crate::persistence::save(path, &staged.scene) {
            return Err(format!("Save {}: {e}", path.display()));
        }
        *self = staged;
        Ok(())
    }
    pub fn require_idle(&self) -> Result<()> {
        if self.bridge.is_some() {
            return Err("Drop or Cancel Bridge first".into());
        }
        if self.reduction.is_some() {
            return Err("Drop or Cancel Polygon Reduction first".into());
        }
        if self.live.is_some()
            || self.cone.is_some()
            || self.ellipsoid.is_some()
            || self.shape.is_some()
            || self.spike.is_some()
        {
            Err("Drop or Cancel the active tool first".into())
        } else {
            Ok(())
        }
    }
    /// PM-UP01..06: stage geometry, destination and validation before publishing
    /// anything. Do not activate/drop a tool or overwrite remembered dimensions.
    fn insert_unit(&mut self, tool: Tool, destination: UnitDestination) -> Result<()> {
        self.require_idle()?;
        if self.selection_start.is_some() {
            return Err("Finish the selection gesture before unit insertion".into());
        }
        if !tool.is_primitive() {
            return Err("Unit insertion requires Cube, Sphere or Cylinder".into());
        }
        self.scene.validate()?;
        let mut scene = self.scene.clone();
        if destination == UnitDestination::NewMesh {
            let next = scene.next_id.max(self.next_item_id);
            if next >= u64::MAX - 1 || scene.items.len() >= crate::MAX_SCENE_ITEMS {
                return Err("Scene item/identity budget exhausted".into());
            }
            scene.next_id = next;
            scene.new_mesh()?;
            // The new item is named after the shape (unit-primitive rule 10).
            scene.active_mut().ok_or("Missing new mesh")?.name = tool.label().into();
        }
        let item = scene
            .active()
            .ok_or("Select an eligible mesh for unit insertion; Shift-click creates a new mesh")?;
        if item.backdrop.is_some() {
            return Err(BACKDROP_NO_GEOMETRY.into());
        }
        if !item.is_mesh() {
            return Err(LOCATOR_NO_GEOMETRY.into());
        }
        if !scene.shows(item) {
            return Err("Unit insertion requires one visible mesh".into());
        }
        let item_id = item.id;
        let make_uvs = !self.primitive_uvs_off[primitive_slot(tool)];
        // Unit primitives ignore the tools' last-used settings and use their
        // reset shapes (unit-primitive-2026-11-05 rules 9 and 11).
        let geometry = if tool == Tool::Box {
            let mut mesh = crate::primitive::segmented_cube(DVec3::ZERO, DVec3::ONE, [1, 1, 1])?;
            if make_uvs {
                crate::add_cube_uvs(&mut mesh, DVec3::ZERO, DVec3::ONE)?;
            }
            mesh
        } else {
            let mut radial = tool
                .initial_radial()
                .ok_or("Missing unit primitive profile")?;
            // The unit Cylinder is 1.5 m tall like the tool's reset size.
            radial.radius = if tool == Tool::Cylinder {
                DVec3::new(0.5, 0.75, 0.5)
            } else {
                DVec3::splat(0.5)
            };
            radial_mesh(
                tool,
                DVec3::ZERO,
                radial,
                &Parameters {
                    make_uvs,
                    ..Default::default()
                },
            )?
        };
        geometry.validate()?;
        let geometry =
            crate::user_defaults::primitive_symmetry(self.active_symmetry()).doubled(geometry)?;
        // Made at the world origin like the drawn tools, then mapped into a
        // moved mesh's local space (provisional, PRIM-MOVED-Q01).
        let geometry = scene.mesh_world_to_local(item_id, geometry)?;
        crate::append_geometry(
            &mut scene.active_mut().ok_or("Select a mesh")?.mesh,
            geometry,
        )?;
        scene.validate()?;
        let before = std::mem::replace(&mut self.scene, scene);
        if destination == UnitDestination::NewMesh {
            self.next_item_id = self.scene.next_id;
        }
        self.record(format!("Unit {}", tool.label()), before);
        Ok(())
    }
    pub(crate) fn preview(&mut self, p: Parameters, apply: bool) -> Result<()> {
        // Bend's centre is the action centre: the frame follows `bend.base`
        // however it changed (panel fields, a click, undo inside the tool).
        let bend_center =
            (self.live.as_ref().is_some_and(|l| l.tool == Tool::Bend)).then_some(p.bend.base);
        let result = self.preview_tool(p, apply);
        if result.is_ok()
            && let Some(center) = bend_center
            && let Some(frame) = self.live.as_mut().and_then(|l| l.action_frame.as_mut())
        {
            frame.center = center;
        }
        // Back to the mode the selection was made in, with that selection
        // (SI-05/06), whether or not the preview worked.
        self.keep_inferred_source();
        result?;
        // Constrain to Background acts after the tool's own change.
        self.constrain_preview()
    }
    fn preview_tool(&mut self, p: Parameters, apply: bool) -> Result<()> {
        let live = self.live.as_ref().ok_or("No active tool")?;
        // Started from a UV view: the pipe runs on the map's UVs (UVT-130,
        // uv_pipe.rs).
        if p.uv.is_some() && crate::uv_pipe::runs_on_uvs(live.tool) {
            return self.preview_uv(p, apply);
        }
        // Several active meshes: the tool runs on each (multi_tools.rs).
        if let Some(targets) = Self::tool_mesh_targets(live.tool, &live.base) {
            return self.preview_on_meshes(&targets, p, apply);
        }
        // Tweak grabs an element of any active mesh and edits that mesh.
        if live.tool == Tool::Tweak
            && let Some(grab) = p.tweak
            && live.base.is_other_active_mesh(grab.item)
        {
            return self.preview_on_meshes(&[grab.item], p, apply);
        }
        self.preview_one(p, apply)
    }
    /// One live tool preview on the primary mesh.
    pub(crate) fn preview_one(&mut self, mut p: Parameters, apply: bool) -> Result<()> {
        let inferred = self.inferred_mode();
        let live = self.live.as_ref().ok_or("No active tool")?;
        let mut scene = live.base.clone();
        // An inferred start: the tool and every layer see its own mode.
        if let Some(mode) = inferred {
            scene.selection.mode = mode;
        }
        // Twist Bevel's note about a group across the symmetry plane.
        let mut twist_note: Option<String> = None;
        match live.tool {
            Tool::Sphere | Tool::Cylinder => {
                let radial = p.radial.ok_or("Missing primitive Radius/resolution")?;
                // The flat first-draw base is a real disc that Drop keeps
                // (drag-creation-2026-11-06 rules 1 and 4; `flat_disc`).
                if !radial.radius.is_finite() || radial.radius.min_element() < 0. {
                    return Err("Radius must be finite and nonnegative".into());
                }
                let mesh = crate::user_defaults::primitive_symmetry(self.symmetry)
                    .doubled(radial_mesh(live.tool, p.position, radial, &p)?)?;
                let item = scene.active().ok_or("Select a mesh")?;
                if !scene.shows(item) {
                    return Err(
                        "The selected mesh is hidden; show it before creating geometry".into(),
                    );
                }
                // Drawn in world space; the mesh is local.
                let mesh = scene.mesh_world_to_local(item.id, mesh)?;
                crate::append_geometry(&mut scene.active_mut().ok_or("Select a mesh")?.mesh, mesh)?;
            }
            Tool::Box => {
                if !apply
                    && !live.applied
                    && p.position == live.params.position
                    && p.size == live.params.size
                    && p.segments == live.params.segments
                    && p.make_uvs == live.params.make_uvs
                {
                    self.live.as_mut().ok_or("No active primitive")?.params = p;
                    return Ok(());
                }
                let mut mesh = crate::primitive::segmented_cube(p.position, p.size, p.segments)?;
                if p.make_uvs && !mesh.faces.is_empty() {
                    crate::add_cube_uvs(&mut mesh, p.position, p.size)?;
                }
                // SYM-PR-01/02.
                let mesh = crate::user_defaults::primitive_symmetry(self.active_symmetry())
                    .doubled(mesh)?;
                let item = scene.active().ok_or("Select an item")?;
                if !scene.shows(item) {
                    return Err(
                        "The selected mesh is hidden; show it before creating geometry".into(),
                    );
                }
                // Drawn in world space (UVs too); the mesh is local.
                let mesh = scene.mesh_world_to_local(item.id, mesh)?;
                let item = scene.active_mut().ok_or("Select an item")?;
                crate::primitive::append_geometry(&mut item.mesh, mesh)?;
            }
            Tool::EdgeExtend
                if self.active_symmetry().axis.is_some()
                    && (p.rotation_delta != DVec3::ZERO || p.scale_delta != DVec3::ONE) =>
            {
                // N27 (Jason, 2026-11-17): with Symmetry, Rotate and Scale
                // turn each half about its own centre, the far half
                // mirrored, instead of both about the point between them.
                let frame = live.action_frame.ok_or("Missing action frame")?;
                let at_zero = self.extend_from_click;
                let symmetry = self.active_symmetry();
                crate::symmetry_sides::two_pass(&mut scene, symmetry, |scene, mirrored| {
                    let edges = scene.selection.edges.clone();
                    let item = scene.active().ok_or("Select a mesh for Extend")?;
                    let world = scene.world_matrix(item.id)?;
                    let mut points: Vec<u32> = edges.iter().flat_map(|&(a, b)| [a, b]).collect();
                    points.sort_unstable();
                    points.dedup();
                    let centre = points
                        .iter()
                        .filter_map(|&v| item.mesh.vertices.get(v as usize))
                        .map(|&q| world.transform_point3(q))
                        .sum::<DVec3>()
                        / points.len().max(1) as f64;
                    let mirror = crate::symmetry_sides::WorldMirror::new(symmetry, world)
                        .filter(|_| mirrored);
                    let (basis, offset) = match &mirror {
                        // A mirrored basis gives the mirror image of the
                        // same rotate and scale (M·B·R·S·Bᵀ·M).
                        Some(m) => (
                            glam::DMat3::from_mat4(m.matrix) * frame.basis,
                            m.vector(frame.basis * p.delta),
                        ),
                        None => (frame.basis, frame.basis * p.delta),
                    };
                    let shape = crate::ExtendShape {
                        offset,
                        inset: p.inset,
                        shift: p.shift,
                        rotate: p.rotation_delta,
                        scale: p.scale_delta,
                        frame: crate::ActionFrame {
                            center: centre,
                            basis,
                        },
                        segments: p.segments[0],
                        make_uvs: p.edge_extend.make_uvs,
                    };
                    let (mesh, outer) = if at_zero {
                        crate::edge_extend_at_zero(&item.mesh, &edges, world, &shape)?
                    } else {
                        crate::edge_extend_shaped(&item.mesh, &edges, world, &shape)?
                    };
                    scene.active_mut().ok_or("Select a mesh")?.mesh = mesh.into();
                    // PF-12: the new edges' points are the point selection too, so
                    // Vertex mode shows them.
                    scene.selection.vertices = crate::topology::edge_points(&outer);
                    scene.selection.edges = outer;
                    Ok(())
                })?;
            }
            Tool::EdgeExtend => {
                let edges = scene.selection.edges.clone();
                let item = scene.active().ok_or("Select a mesh for Extend")?;
                // Offset is along the action axes (as Move's); the shape
                // works in world space and maps back to the mesh.
                let frame = live.action_frame.ok_or("Missing action frame")?;
                let shape = crate::ExtendShape {
                    offset: frame.basis * p.delta,
                    inset: p.inset,
                    shift: p.shift,
                    rotate: p.rotation_delta,
                    scale: p.scale_delta,
                    frame,
                    segments: p.segments[0],
                    make_uvs: p.edge_extend.make_uvs,
                };
                let world = scene.world_matrix(item.id)?;
                let (mut mesh, outer) = if self.extend_from_click {
                    crate::edge_extend_at_zero(&item.mesh, &edges, world, &shape)?
                } else {
                    crate::edge_extend_shaped(&item.mesh, &edges, world, &shape)?
                };
                // SYM3-12: the offset is mirrored for the partner side.
                let symmetry = self.active_symmetry();
                if symmetry.axis.is_some() && shape.offset != DVec3::ZERO {
                    let primary = self.symmetry_primary(&scene);
                    let v = world.inverse().transform_vector3(shape.offset);
                    let moved: Vec<u32> = outer.iter().flat_map(|&(a, b)| [a, b]).collect();
                    let tolerance = crate::symmetry::match_tolerance(&item.mesh);
                    symmetry.mirror_offset(&mut mesh.vertices, &moved, v, primary, tolerance);
                }
                scene.active_mut().unwrap().mesh = mesh.into();
                // PF-12: the new edges' points are the point selection too, so
                // Vertex mode shows them.
                scene.selection.vertices = crate::topology::edge_points(&outer);
                scene.selection.edges = outer;
            }
            Tool::EdgeCrease => {
                let edges = scene.selection.edges.clone();
                let item = scene.active().ok_or("Select a mesh for Edge Weight")?;
                let mesh = crate::edge_crease_geometry(
                    &item.mesh,
                    &edges,
                    p.shift,
                    p.edge_weight_additive,
                )?;
                scene.active_mut().unwrap().mesh = mesh.into();
            }
            Tool::EdgeSlide => {
                let edges = scene.selection.edges.clone();
                let item = scene.active().ok_or("Select a mesh for Slide")?;
                let weights = self.falloff_weights(&scene, item.id)?;
                let (mesh, after) = crate::edge_slide_extra(
                    &item.mesh,
                    &edges,
                    p.shift,
                    p.edge_slide_distance,
                    weights.as_deref(),
                    &p.slide_extras,
                )?;
                let merged = mesh.vertices.len() != item.mesh.vertices.len();
                let mut mesh = mesh;
                if !merged {
                    // SYM3-12: the slide direction is mirrored for the
                    // partner side.
                    let symmetry = self.active_symmetry();
                    let primary = self.symmetry_primary(&scene);
                    symmetry.mirror_moved(&item.mesh, &mut mesh, primary);
                }
                scene.active_mut().unwrap().mesh = mesh.into();
                if merged {
                    // ES-04: merged points renumber the mesh.
                    scene.selection.edges = after;
                    scene.selection.vertices.clear();
                    scene.selection.faces.clear();
                }
            }
            Tool::EdgeExtrude => {
                let edges = scene.selection.edges.clone();
                let item = scene.active().ok_or("Select a mesh for Edge Extrude")?;
                // Use Material tags the ridge quads (extrude-2026-11-05, EE08).
                let material = p.vertex_use_material.then_some(p.vertex_material.as_str());
                let (mesh, raised) =
                    crate::edge_extrude_with(&item.mesh, &edges, p.shift, p.inset, material)?;
                scene.active_mut().unwrap().mesh = mesh.into();
                // The ridge edges and their points are selected after.
                scene.selection.vertices = crate::topology::edge_points(&raised);
                scene.selection.edges = raised;
            }
            Tool::EdgeBevel | Tool::EdgeChamfer => {
                let item = scene.active().ok_or("Select a mesh for Edge Bevel")?;
                // TA-06: nothing selected bevels every edge (Edge Bevel only;
                // activation refuses Edge Chamfer without a selection).
                let edges = if live.tool == Tool::EdgeBevel && scene.selection.edges.is_empty() {
                    item.mesh.edges()
                } else {
                    scene.selection.edges.clone()
                };
                // Widths are world distances; the mesh is local (uniform
                // scale assumed for the basic profile).
                let scale = crate::scene::uniform_scale(scene.world_matrix(item.id)?);
                let weights = self.falloff_weights(&scene, item.id)?;
                let settings = crate::EdgeBevelSettings {
                    width: p.chamfer.offset / scale,
                    rows: p.chamfer.segments,
                    depth: p.chamfer.depth,
                    chamfer: live.tool == Tool::EdgeChamfer,
                    options: p.edge_bevel.clone(),
                };
                let result =
                    crate::edge_bevel_with(&item.mesh, &edges, &settings, weights.as_deref())?;
                scene.active_mut().unwrap().mesh = result.mesh.into();
                // EBC-J-01 / EBC-P-02: Edge Bevel selects the strips' edges,
                // Edge Chamfer nothing (the original edges with Square).
                // EBC-B3 oracles: the points selected are the ends of those
                // edges.
                scene.selection.vertices = crate::topology::edge_points(&result.selected_after);
                scene.selection.edges = result.selected_after;
            }
            Tool::VertexCreate => {
                let item = scene.active().ok_or("Select a mesh for Create")?;
                let local = scene.world_matrix(item.id)?.inverse();
                let mut points: Vec<DVec3> = p
                    .sketch_path
                    .iter()
                    .map(|&q| local.transform_point3(q))
                    .collect();
                // Class F, Weld seam (SYM3-22/23): each placed point gets a
                // mirror point; one on the plane (within the matching
                // tolerance) is made once, exactly on it.
                let symmetry = self.active_symmetry();
                if symmetry.axis.is_some() {
                    let tolerance = crate::symmetry::match_tolerance(&item.mesh).max(
                        crate::symmetry::match_tolerance(&crate::Mesh {
                            vertices: points.clone(),
                            ..Default::default()
                        }),
                    );
                    let mut mirrored = Vec::new();
                    for q in &mut points {
                        if symmetry.side(*q, tolerance) == Some(crate::Side::Plane) {
                            *q = symmetry.onto_plane(*q);
                        } else {
                            mirrored.push(symmetry.mirror(*q));
                        }
                    }
                    points.extend(mirrored);
                }
                let (mesh, ids) = crate::add_points(&item.mesh, &points)?;
                scene.active_mut().unwrap().mesh = mesh.into();
                if !ids.is_empty() {
                    scene.selection.vertices = ids;
                }
            }
            Tool::VertexBevel | Tool::VertexExtrude | Tool::VertexMerge | Tool::VertexTear => {
                let mut points = scene.selection.vertices.clone();
                let item = scene.active().ok_or("Select a mesh")?;
                // Vertex Extrude with nothing selected processes every point
                // (extrude-2026-11-05, VE08).
                if points.is_empty() && live.tool == Tool::VertexExtrude {
                    points = (0..item.mesh.vertices.len() as u32).collect();
                }
                let world = scene.world_matrix(item.id)?;
                // Distances are world lengths; the mesh is local (uniform
                // scale assumed for the basic profile).
                let scale = crate::scene::uniform_scale(world);
                let (mesh, selected) = match live.tool {
                    Tool::VertexBevel => crate::vertex_commands::vertex_bevel_round(
                        &item.mesh,
                        &points,
                        p.inset / scale,
                        p.vertex_round_level,
                        self.falloff_weights(&scene, item.id)?.as_deref(),
                    )?,
                    Tool::VertexExtrude => crate::vertex_commands::vertex_extrude_weighted(
                        &item.mesh,
                        &points,
                        p.shift / scale,
                        p.inset / scale,
                        self.falloff_weights(&scene, item.id)?.as_deref(),
                    )?,
                    Tool::VertexMerge => crate::vertex_merge_with(
                        &item.mesh,
                        &points,
                        p.shift / scale,
                        crate::MergeOptions {
                            keep_small: p.vertex_merge_keep_small,
                            test_morphs: p.vertex_merge_test_morphs,
                            ..Default::default()
                        },
                    )?,
                    _ => crate::vertex_tear_geometry(
                        &item.mesh,
                        &points,
                        &scene.selection.faces,
                        world.inverse().transform_vector3(p.delta),
                    )?,
                };
                let mut mesh = mesh;
                // SYM3-12: Tear's new points move by the drag; those torn
                // from the other side take the mirrored drag, seam points
                // keep only the part along the plane (SYM3-13).
                if live.tool == Tool::VertexTear && mesh.vertices.len() > item.mesh.vertices.len() {
                    let symmetry = self.active_symmetry();
                    let primary = self.symmetry_primary(&scene);
                    let torn: Vec<u32> =
                        (item.mesh.vertices.len() as u32..mesh.vertices.len() as u32).collect();
                    let v = world.inverse().transform_vector3(p.delta);
                    let tolerance = crate::symmetry::match_tolerance(&item.mesh);
                    symmetry.mirror_offset(&mut mesh.vertices, &torn, v, primary, tolerance);
                }
                // VTO-T02/T03 Use Material: tag the new polygons (every
                // polygon of a rounded cap, VX-B07).
                if p.vertex_use_material
                    && matches!(live.tool, Tool::VertexBevel | Tool::VertexExtrude)
                {
                    let new = crate::vertex_commands::new_vertex_tool_polygons(&item.mesh, &mesh);
                    crate::vertex_commands::assign_material(&mut mesh, new, &p.vertex_material);
                }
                let same_polygons = item.mesh.faces == mesh.faces;
                scene.active_mut().unwrap().mesh = mesh.into();
                // Tear keeps polygon order, so its polygon selection stays.
                if !same_polygons && live.tool != Tool::VertexTear {
                    scene.selection.edges.clear();
                    scene.selection.faces.clear();
                }
                scene.selection.vertices = selected;
            }
            Tool::LoopSlice
            | Tool::EdgeSlice
            | Tool::PolygonSlice
            | Tool::CurveSlice
            | Tool::PenSlice
            | Tool::PrimitiveSlice
            | Tool::AxisSlice => {
                crate::slice::preview(live.tool, &mut scene, &p.slice, self.active_symmetry())?
            }
            Tool::Contour => crate::contour::preview(&mut scene, &p.contour)?,
            Tool::Mirror
            | Tool::Lathe
            | Tool::Clone
            | Tool::Array
            | Tool::RadialArray
            | Tool::CurveExtrude
            | Tool::CurveClone
            | Tool::ScatterClone
            | Tool::Symmetrize => crate::duplicate::preview(
                live.tool,
                &mut scene,
                &p.duplicate,
                self.next_item_id,
                self.active_symmetry(),
            )?,
            Tool::VertexSlide => {
                let points = scene.selection.vertices.clone();
                let item = scene.active().ok_or("Select a mesh for Slide")?;
                let world = scene.world_matrix(item.id)?;
                let rail = world.inverse().transform_vector3(p.delta);
                // Distance is a world length; the mesh is local (uniform
                // scale assumed, as for the other vertex tools).
                let amount = if p.edge_slide_distance {
                    p.shift / crate::scene::uniform_scale(world)
                } else {
                    p.shift
                };
                let (mesh, selected) = crate::vertex_commands::vertex_slide_extra(
                    &item.mesh,
                    &points,
                    rail,
                    amount,
                    crate::vertex_commands::SlideOptions {
                        by_distance: p.edge_slide_distance,
                        merge: p.vertex_slide_merge,
                    },
                    &p.slide_extras,
                )?;
                let merged = mesh.vertices.len() != item.mesh.vertices.len();
                let mut mesh = mesh;
                if !merged {
                    // SYM3-12: the rail is mirrored for the partner side.
                    let symmetry = self.active_symmetry();
                    let primary = self.symmetry_primary(&scene);
                    symmetry.mirror_moved(&item.mesh, &mut mesh, primary);
                }
                scene.active_mut().unwrap().mesh = mesh.into();
                // Merged points renumber the mesh; edge and polygon picks
                // would point at the wrong components.
                if merged {
                    scene.selection.edges.clear();
                    scene.selection.faces.clear();
                }
                scene.selection.vertices = selected;
            }
            Tool::Push | Tool::Smooth => {
                let points = crate::transform_vertices(&scene)?;
                let item = scene.active().ok_or("Select a mesh")?;
                let weights = self.falloff_weights(&scene, item.id)?;
                let mesh = if live.tool == Tool::Push {
                    // Distance is a world length; the mesh is local (uniform
                    // scale assumed, as for Vertex Extrude).
                    let scale = crate::scene::uniform_scale(scene.world_matrix(item.id)?);
                    let faces = (scene.selection.mode == Mode::Polygon)
                        .then_some(scene.selection.faces.as_slice());
                    // Element Push (hands-on 2026-11-17): with an Element
                    // falloff the picked element's normal is the one
                    // direction, so a polygon moves as one flat piece and
                    // its Range neighbours follow it.
                    let element = self
                        .falloff_element
                        .filter(|g| g.item == item.id)
                        .filter(|_| self.has_element_falloff())
                        .map(|g| crate::deform::element_normal(&item.mesh, g.element));
                    match element {
                        Some(dir) => crate::deform::push_along(
                            &item.mesh,
                            &points,
                            dir,
                            p.shift / scale,
                            weights.as_deref(),
                        )?,
                        None => crate::deform::push_geometry(
                            &item.mesh,
                            &points,
                            p.shift / scale,
                            p.push_normals,
                            faces,
                            weights.as_deref(),
                        )?,
                    }
                } else {
                    crate::deform::smooth_geometry(
                        &item.mesh,
                        &points,
                        &p.smooth,
                        weights.as_deref(),
                    )?
                };
                scene.active_mut().unwrap().mesh = mesh.into();
            }
            Tool::UvProject
            | Tool::UvViewProject
            | Tool::UvUnwrap
            | Tool::UvRelax
            | Tool::UvPelt
            | Tool::UvPeeler => {
                let item = scene.active().ok_or("Select a mesh")?;
                let weights = self.falloff_weights(&scene, item.id)?;
                if let Some(mesh) =
                    crate::uv_relax::uv_tool_geometry(live.tool, &scene, &p, weights.as_deref())?
                {
                    scene.active_mut().unwrap().mesh = mesh.into();
                }
            }
            // UVT-43: on the UV proxy (the pipe swapped it in); selected UV
            // positions move by the box's edit, weighted by the falloff.
            Tool::UvBoxTransform => {
                if p.uv.is_none() {
                    return Err("Box Transform works in a UV view".into());
                }
                p.uv_box.validate()?;
                let item = scene.active().ok_or("Select a mesh")?;
                let id = item.id;
                let weights = self.falloff_weights(&scene, id)?;
                let points = crate::transform_vertices(&scene)?;
                let (lo, hi) = crate::uv_box::bounds(&item.mesh.vertices, points.iter().copied())
                    .ok_or("Box Transform: no UVs selected")?;
                let mut mesh: crate::Mesh = (*item.mesh).clone();
                for &v in &points {
                    let q = mesh.vertices[v as usize];
                    let to = p.uv_box.apply(q.truncate(), lo, hi);
                    let w = weights
                        .as_ref()
                        .map_or(1., |w| w.get(v as usize).copied().unwrap_or(0.));
                    let moved = q.truncate().lerp(to, w);
                    mesh.vertices[v as usize] = DVec3::new(moved.x, moved.y, q.z);
                }
                scene.active_mut().unwrap().mesh = mesh.into();
            }
            Tool::Jitter | Tool::Quantize | Tool::RadialAlign | Tool::LinearAlign => {
                let item = scene.active().ok_or("Select a mesh")?;
                let weights = self.falloff_weights(&scene, item.id)?;
                let mesh =
                    crate::deform_finish::tool_geometry(live.tool, &scene, &p, weights.as_deref())?;
                // Symmetry: the primary side's result, mirrored (SYM3-12).
                let mut mesh = mesh;
                let symmetry = self.active_symmetry();
                let primary = self.symmetry_primary(&scene);
                let base = &scene.active().unwrap().mesh;
                symmetry.mirror_moved(base, &mut mesh, primary);
                scene.active_mut().unwrap().mesh = mesh.into();
            }
            Tool::SymmetryTool => {
                let item = scene
                    .active()
                    .ok_or("Select a mesh for the Symmetry Tool")?;
                // Threshold is a world distance; the mesh is local (uniform
                // scale assumed, as for the other tools).
                let scale = scene
                    .world_matrix(item.id)?
                    .transform_vector3(DVec3::X)
                    .length()
                    .max(1e-12);
                let o = p.symmetry_fix;
                let fixed = crate::symmetry_fix::symmetry_fix(
                    &item.mesh,
                    self.active_symmetry(),
                    o.threshold / scale,
                    o.topology,
                    o.reverse,
                )?;
                self.symmetry_unpaired = fixed.unpaired;
                scene.active_mut().unwrap().mesh = fixed.mesh.into();
            }
            Tool::Bend => {
                let points = crate::transform_vertices(&scene)?;
                let item = scene.active().ok_or("Select a mesh")?;
                let weights = self.falloff_weights(&scene, item.id)?;
                let world = scene.world_matrix(item.id)?;
                let axes = self.bend_axes(&scene);
                let direction = p.bend.direction(axes);
                // BN-45 (BH-Q12, provisional): an axis along the spine can't
                // turn it; the mesh stays as it was (see `bend_notice`).
                let bend = if p.bend.axis_along_spine(axes) {
                    crate::BendOptions {
                        angle: 0.,
                        ..p.bend
                    }
                } else {
                    p.bend
                };
                // SYM3-12: the side the user works on bends as entered, the
                // other side with the mirrored base, spine and lean; seam
                // points stay on the plane (SYM3-13).
                let symmetry = self.active_symmetry();
                let primary = self.symmetry_primary(&scene);
                let list: Vec<u32> = points.iter().copied().collect();
                let split =
                    crate::symmetry_sides::split_points(&item.mesh, symmetry, &list, primary);
                let mirror = crate::symmetry_sides::WorldMirror::new(symmetry, world);
                let mesh = match mirror.filter(|_| split.is_two_sided()) {
                    None => crate::bend::bend_geometry(
                        &item.mesh,
                        &points,
                        world,
                        &bend,
                        direction,
                        weights.as_deref(),
                    )?,
                    Some(m) => {
                        let first: std::collections::BTreeSet<u32> =
                            split.primary.iter().copied().collect();
                        let second: std::collections::BTreeSet<u32> =
                            split.other.iter().copied().collect();
                        let mut mesh = crate::bend::bend_geometry(
                            &item.mesh,
                            &first,
                            world,
                            &bend,
                            direction,
                            weights.as_deref(),
                        )?;
                        let mirrored = crate::BendOptions {
                            base: m.point(bend.base),
                            spine: m.vector(bend.spine),
                            ..bend
                        };
                        mesh = crate::bend::bend_geometry(
                            &mesh,
                            &second,
                            world,
                            &mirrored,
                            m.vector(direction),
                            weights.as_deref(),
                        )?;
                        let tolerance = crate::symmetry::match_tolerance(&item.mesh);
                        for &v in &first {
                            if symmetry.side(item.mesh.vertices[v as usize], tolerance)
                                == Some(crate::Side::Plane)
                            {
                                let q = &mut mesh.vertices[v as usize];
                                *q = symmetry.onto_plane(*q);
                            }
                        }
                        mesh
                    }
                };
                scene.active_mut().unwrap().mesh = mesh.into();
            }
            Tool::EdgeRelax => {
                let edges = scene.selection.edges.clone();
                let item = scene.active().ok_or("Select a mesh for Edge Relax")?;
                let mesh = crate::edge_relax_geometry(&item.mesh, &edges, p.shift)?;
                scene.active_mut().unwrap().mesh = mesh.into();
            }
            Tool::Chamfer => {
                let selected = scene.selection.faces.clone();
                let item = scene.active().ok_or("Select a mesh for Chamfer Edit")?;
                let output = crate::chamfer_geometry(&item.mesh, &selected, p.chamfer)?;
                // Rule 11: the selection stays when no polygons were added
                // or removed (CE02, CE10); a rebuilt strip clears it.
                let rebuilt = output.faces.len() != item.mesh.faces.len();
                scene.active_mut().unwrap().mesh = output.into();
                if rebuilt {
                    scene.selection.faces.clear();
                }
            }
            Tool::Inset | Tool::Shift => {
                let selected = scene.selection.faces.clone();
                let item = scene.active().ok_or("Select a mesh")?;
                // World distances on the item's own units (uniform scale
                // assumed, as for the vertex tools).
                let scale = crate::scene::uniform_scale(scene.world_matrix(item.id)?);
                let output = if live.tool == Tool::Inset {
                    crate::inset_geometry(&item.mesh, &selected, p.inset / scale)?
                } else {
                    crate::shift_geometry(&item.mesh, &selected, p.shift / scale)?
                };
                scene.active_mut().unwrap().mesh = output.mesh.into();
                scene.selection.faces = output.selected;
            }
            Tool::PolyHaul => {
                let selected = scene.selection.faces.clone();
                let item = scene.active().ok_or("Select a mesh for Poly Haul")?;
                let local = scene.world_matrix(item.id)?.inverse();
                let amount = crate::HaulAmount {
                    offset: p.poly_haul_offset,
                    vector: local.transform_vector3(p.poly_haul_amount.vector),
                    ..p.poly_haul_amount
                };
                let untouched = p.poly_haul_mode.uses_selection() && selected.is_empty()
                    || !p.poly_haul_mode.uses_selection()
                        && p.poly_haul_target == crate::HaulTarget::None;
                // Nothing to act on yet (no selection, or no drag started on
                // an element): the base scene stays.
                if !untouched {
                    let mut output = crate::poly_haul_mode_geometry(
                        &item.mesh,
                        &selected,
                        p.poly_haul_mode,
                        p.poly_haul_target,
                        amount,
                    )?;
                    // SYM3-10/12: a picked edge, point or polygon has its
                    // mirror partner hauled too, with the mirrored drag. The
                    // selection modes act on a symmetric selection already.
                    let symmetry = self.active_symmetry();
                    if !p.poly_haul_mode.uses_selection()
                        && let Some(target) = crate::symmetry_sides::mirrored_haul_target(
                            &item.mesh,
                            &output.mesh,
                            p.poly_haul_target,
                            symmetry,
                        )
                    {
                        let mirrored = crate::HaulAmount {
                            vector: symmetry.mirror_vector(amount.vector),
                            ..amount
                        };
                        let faces = output.faces.clone().unwrap_or_else(|| selected.clone());
                        output = crate::poly_haul_mode_geometry(
                            &output.mesh,
                            &faces,
                            p.poly_haul_mode,
                            target,
                            mirrored,
                        )?;
                    }
                    scene.active_mut().unwrap().mesh = output.mesh.into();
                    if let Some(faces) = output.faces {
                        scene.selection.faces = faces;
                    }
                }
            }
            Tool::SketchExtrude => {
                if !p.sketch_precision.is_finite() || !(1. ..=1000.).contains(&p.sketch_precision) {
                    return Err("Sketch Extrude Precision must be 1–1000 logical pixels".into());
                }
                if !p.sketch_path.is_empty() {
                    // SYM3-12: with Symmetry, the other side's polygons
                    // follow the mirrored path.
                    let symmetry = self.active_symmetry();
                    crate::symmetry_sides::two_pass(&mut scene, symmetry, |scene, mirrored| {
                        let selected = scene.selection.faces.clone();
                        let item = scene.active().ok_or("Select a mesh for Sketch Extrude")?;
                        // The drawn path is in world space; the mesh is local.
                        let local = scene.world_matrix(item.id)?.inverse();
                        let path: Vec<DVec3> = p
                            .sketch_path
                            .iter()
                            .map(|&q| local.transform_point3(q))
                            .map(|q| if mirrored { symmetry.mirror(q) } else { q })
                            .collect();
                        let (mesh, caps) =
                            crate::sketch_extrude_geometry(&item.mesh, &selected, &path)?;
                        scene.active_mut().unwrap().mesh = mesh.into();
                        scene.selection.faces = caps;
                        Ok(())
                    })?;
                }
            }
            Tool::SmoothShift | Tool::Thicken => {
                let selected = scene.selection.faces.clone();
                let item = scene.active().ok_or("Select an editable visible mesh")?;
                if !scene.shows(item) || scene.selection.mode != Mode::Polygon {
                    return Err("Select a visible mesh in Polygon mode".into());
                }
                // Q-SS-04 (Jason, 2026-11-18): Scale follows the selected
                // action centre; with none, the selection's centre. Origin
                // is the item's own origin (as the command oracles); the
                // world points of Element and Screen are taken into the item.
                let to_local = scene.world_matrix(item.id)?.inverse();
                let symmetry = self.active_symmetry();
                let point = |at: DVec3| crate::SmoothShiftPivot::Point {
                    at,
                    mirror: symmetry.axis.is_some().then_some(symmetry),
                };
                // ACS-20 (Jason, 2026-11-18): through the pivot service like
                // a transform tool: a placed centre (ACS-07) and, with
                // Symmetry, the service's primary pivot, the other side
                // scaling about its mirror image (ACS-04) and seam points
                // about its foot on the plane (ACS-05).
                use crate::CenterMode as C;
                let mirrored = symmetry.axis.is_some();
                let pivot = match (live.placed_centre, self.action_context.center) {
                    (Some(at), _) => point(to_local.transform_point3(at)),
                    (None, C::Local) => crate::SmoothShiftPivot::Local,
                    (None, C::None | C::Selection | C::SelectionAutoAxis | C::Automatic)
                        if mirrored =>
                    {
                        crate::SmoothShiftPivot::HalfBox(symmetry)
                    }
                    (None, C::SelectionBorder) if mirrored => {
                        crate::SmoothShiftPivot::HalfBorder(symmetry)
                    }
                    (None, C::None | C::Selection | C::SelectionAutoAxis | C::Automatic) => {
                        crate::SmoothShiftPivot::SelectionBox
                    }
                    (None, C::SelectionBorder) => crate::SmoothShiftPivot::SelectionBorder,
                    (None, C::Origin) => point(DVec3::ZERO),
                    (None, C::Element(at) | C::Screen(at)) => {
                        point(to_local.transform_point3(at.point()))
                    }
                    (None, C::Parent | C::Pivot) => point(
                        to_local
                            .transform_point3(live.action_frame.map_or(DVec3::ZERO, |f| f.center)),
                    ),
                };
                let result = crate::smooth_shift_geometry_about(
                    &item.mesh,
                    &selected,
                    p.smooth_shift,
                    None,
                    pivot,
                )?;
                scene.active_mut().unwrap().mesh = result.mesh.into();
                scene.selection.faces = result.selected;
                scene.selection.vertices.clear();
                scene.selection.edges.clear();
            }
            Tool::Extrude => {
                let item = scene.active().ok_or("Select a mesh")?;
                let source_ids = live
                    .identity_base
                    .as_ref()
                    .and_then(|ids| ids.meshes.get(&item.id))
                    .ok_or("Missing extrusion identity basis")?;
                source_ids.validate_shared(item.mesh.shared())?;
                // Stable ordered targets are resolved once into this private
                // kernel view; neither a coordinate nor a stale dense offset
                // supplies correspondence after the edit.
                // PE-O-03: with nothing selected every polygon is extruded.
                let faces: Vec<u32> = if scene.selection.faces.is_empty() {
                    (0..item.mesh.faces.len() as u32).collect()
                } else {
                    scene.selection.faces.clone()
                };
                let targets = faces
                    .iter()
                    .map(|&f| {
                        source_ids
                            .faces
                            .get(f as usize)
                            .copied()
                            .map(|face| crate::FaceHandle {
                                scope: source_ids.scope,
                                face,
                            })
                            .ok_or_else(|| String::from("Invalid source face target"))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let selected = targets
                    .into_iter()
                    .map(|h| source_ids.face_index(h))
                    .collect::<Result<Vec<_>>>()?;
                let world = scene.world_matrix(item.id)?;
                if !item.mesh.attributes.polygon_sets.is_empty()
                    && (world != glam::DMat4::IDENTITY || self.action_context != Default::default())
                {
                    return Err(
                        "I1-S named sets require identity world transform and neutral context"
                            .into(),
                    );
                }
                if !world.is_finite() || world.determinant().abs() < 1e-15 {
                    return Err("Extrude requires an invertible world transform".into());
                }
                if !apply && p.extent == DVec3::ZERO {
                    self.extrude_moved_caps = scene.selection.faces.clone();
                    self.scene = scene;
                    if let Some(ids) = &live.identity_base {
                        self.identities.current = ids.clone();
                        self.identities.observe(&self.scene)?;
                    }
                    let l = self.live.as_mut().unwrap();
                    l.identity_preview = None;
                    l.params = p;
                    l.applied = true;
                    l.preview_error = None;
                    return Ok(());
                }
                let mut source = item.mesh.clone();
                // Facing is evaluated in world coordinates, including mirrored
                // transforms; stored attributes remain authored local payloads.
                for v in &mut source.vertices {
                    *v = world.transform_point3(*v);
                }
                let map = p
                    .extrude_uv_map
                    .as_deref()
                    .filter(|n| source.uv_maps.contains_key(*n));
                let mut result = crate::extrude_geometry(
                    &source,
                    &selected,
                    p.extent,
                    p.extrude_sides,
                    p.extrude_mode,
                    p.extrude_uvs,
                    map,
                )?;
                let inverse = world.inverse();
                for (index, v) in result.mesh.vertices.iter_mut().enumerate() {
                    // Preserve untouched local coordinates exactly; a world/local
                    // roundtrip must not introduce drift in unselected geometry.
                    if index < source.vertices.len() && *v == source.vertices[index] {
                        *v = item.mesh.vertices[index];
                    } else {
                        *v = inverse.transform_point3(*v);
                    }
                }
                // SYM3-12: the extrude offset is mirrored for the caps on the
                // partner side (a normal-only extrude is symmetric anyway).
                let symmetry = self.active_symmetry();
                if symmetry.axis.is_some() && p.extent != DVec3::ZERO {
                    let primary = self.symmetry_primary(&scene);
                    let v = inverse.transform_vector3(p.extent);
                    let moved: Vec<u32> = result
                        .moved_caps
                        .iter()
                        .filter_map(|&f| result.mesh.faces.get(f as usize))
                        .flat_map(|f| f.vertices.iter().copied())
                        .collect();
                    let tolerance = crate::symmetry::match_tolerance(&item.mesh);
                    symmetry.mirror_offset(
                        &mut result.mesh.vertices,
                        &moved,
                        v,
                        primary,
                        tolerance,
                    );
                }
                let topology_changed = result.mesh != *item.mesh;
                let source_ids = live
                    .identity_base
                    .as_ref()
                    .and_then(|ids| ids.meshes.get(&item.id))
                    .ok_or("Missing extrusion stage identity basis")?;
                let ids = if let Some((old_plan, ids)) = &live.identity_preview
                    && old_plan == &result.correspondence
                {
                    ids.validate(&result.mesh)?;
                    ids.clone()
                } else {
                    self.identities.with_ids(|next| {
                        source_ids.transported(
                            &item.mesh,
                            &result.mesh,
                            &result.correspondence,
                            next,
                        )
                    })?
                };
                self.identities
                    .current
                    .meshes
                    .insert(item.id, ids.clone().into());
                self.live.as_mut().unwrap().identity_preview =
                    Some((result.correspondence.clone(), ids));
                scene.active_mut().unwrap().mesh = result.mesh.into();
                // While the tool is live the moving caps stay selected (a
                // continuation extrudes them again); Apply, and Drop
                // (`select_extrude_caps`), select both caps (PE-M-03).
                scene.selection.faces = if apply {
                    result.caps
                } else {
                    result.moved_caps.clone()
                };
                if topology_changed {
                    scene.selection.vertices.clear();
                    scene.selection.edges.clear();
                }
                scene.validate()?;
                self.extrude_moved_caps = result.moved_caps;
                self.identities.observe(&scene)?;
            }
            Tool::TwistBevel => twist_note = self.preview_twist_bevel(&mut scene, live, &p)?,
            Tool::Bevel => {
                // PB-O-01: nothing selected bevels every polygon and leaves
                // nothing selected. bevel-activation-2026-11-05 BA11 says the
                // opposite; kept as measured in polygon-bevel-2026-11-02 until
                // the spec side settles it (SPECIFICATION_QUESTIONS BA-Q01).
                let everything = scene.selection.faces.is_empty();
                let selected = if everything {
                    (0..scene.active().ok_or("Select an item")?.mesh.faces.len() as u32).collect()
                } else {
                    scene.selection.faces.clone()
                };
                let item = scene.active().ok_or("Select an item")?;
                let weights = self.falloff_weights(&scene, item.id)?;
                let bevel = if apply {
                    crate::bevel_geometry_applied
                } else {
                    crate::bevel_geometry_with_profile
                };
                let result = bevel(
                    &item.mesh,
                    &selected,
                    p.shift,
                    p.inset,
                    p.group_polygons,
                    &p.bevel,
                    weights.as_deref(),
                    self.bevel_profile,
                )
                .map_err(|e| format!("Bevel Shift/Inset: {e}"))?;
                let topology_changed = result.mesh != *item.mesh;
                scene.active_mut().unwrap().mesh = result.mesh.into();
                scene.selection.faces = if everything {
                    Vec::new()
                } else {
                    result.selected
                };
                if topology_changed {
                    scene.selection.vertices.clear();
                    scene.selection.edges.clear();
                }
            }
            Tool::Tweak => {
                // The grabbed element's points move by the world offset
                // `delta`, nearby points by their falloff weight;
                // selection, topology and IDs stay as they were (TWK-22).
                if !p.delta.is_finite() {
                    return Err("Tweak offset must be finite".into());
                }
                if let Some(grab) = p.tweak {
                    let item = scene.active().ok_or("Select a mesh")?;
                    if item.id != grab.item {
                        return Err("Tweak edits the active mesh only".into());
                    }
                    let world = live.base.world_matrix(grab.item)?;
                    let offset = world.inverse().transform_vector3(p.delta);
                    // TWK-12/15/16 and TWK-20 (provisional Symmetry): which
                    // points move and by how much comes from the press-time
                    // mesh, worked out once per drag.
                    let symmetry = self.active_symmetry();
                    let base_mesh = &live.base.active().ok_or("Select a mesh")?.mesh;
                    // In a UV view (UVT-131) Range may be in screen pixels
                    // (UVT-Q11) and connectivity read from the 3D mesh
                    // (UVT-Q12); see uv_pipe.rs.
                    let uv_space = p.uv.as_ref();
                    let mut options = p.tweak_options;
                    if let Some(space) = uv_space
                        && crate::uv_pipe::RANGE_UNITS == crate::uv_pipe::RangeUnits::Pixels
                        && space.pixels_per_uv > 0.
                    {
                        options.range /= space.pixels_per_uv;
                    }
                    let uv_proxy = uv_space.and_then(|s| s.proxy.clone()).filter(|_| {
                        crate::uv_pipe::CONNECTIVITY == crate::uv_pipe::UvConnectivity::Mesh
                    });
                    let pieces =
                        |seeds: &[u32]| uv_proxy.as_ref().expect("checked").connected_in_3d(seeds);
                    // The plan rides on the live parameters from frame to frame.
                    let plan = match p
                        .tweak_plan
                        .take()
                        .or_else(|| live.params.tweak_plan.clone())
                    {
                        Some(plan)
                            if plan.grab == grab
                                && plan.options == options
                                && plan.symmetry == symmetry =>
                        {
                            plan
                        }
                        _ => crate::tweak::tweak_plan_with(
                            base_mesh,
                            world,
                            grab,
                            options,
                            symmetry,
                            uv_proxy
                                .is_some()
                                .then_some(&pieces as crate::tweak::PieceFinder<'_>),
                        )?,
                    };
                    let item = scene.active_mut().ok_or("Select a mesh")?;
                    crate::tweak_apply(&mut item.mesh, base_mesh, &plan, offset)?;
                    p.tweak_plan = Some(plan);
                } else if p.delta != DVec3::ZERO {
                    return Err("Tweak: press on a vertex, edge or polygon and drag".into());
                }
            }
            Tool::Transform | Tool::Move | Tool::Rotate | Tool::Scale => {
                if !p.delta.is_finite()
                    || !p.rotation_delta.is_finite()
                    || !p.scale_delta.is_finite()
                {
                    return Err("Transform must be finite".into());
                }
                let factors = if live.tool == Tool::Transform {
                    Some(p.scale_delta)
                } else {
                    (live.tool == Tool::Scale).then_some(p.delta)
                };
                // IT-04: with Negative Scale on, an item scale may flip sign
                // (never zero). Components keep the positive-only rule.
                let negative_ok = p.negative_scale && scene.selection.mode == Mode::Item;
                if let Some(f) = factors
                    && (f.abs().min_element() <= 1e-8 || (!negative_ok && f.min_element() <= 1e-8))
                {
                    return Err(if negative_ok {
                        "Scale can't be zero".into()
                    } else {
                        "Scale must be positive; turn on Negative Scale for items".into()
                    });
                }
                if scene.selection.mode == Mode::Item
                    && let Some(targets) = live.base.item_transform_targets()
                {
                    // Several selected items move together (multi_transform.rs).
                    crate::multi_transform::transform_items(
                        &mut scene, &live.base, &targets, live.tool, &p,
                    )?;
                    // Planar spec: a Texture Locator's Size stops at 1 mm.
                    scene.clamp_locator_sizes();
                } else if scene.selection.mode == Mode::Item {
                    let item = scene
                        .active_mut()
                        .ok_or("Select an item; empty selection does not target all items")?;
                    // CP-A02: a locked item refuses item transforms.
                    if item.is_locked() {
                        return Err(format!("{} is locked (Assembly > Lock)", item.name));
                    }
                    // ENVIRONMENT-PLACEMENT §2: an environment has no
                    // transform; its sky turns on its layers' locators.
                    if item.environment.is_some() {
                        return Err(crate::environment::NO_TRANSFORM.into());
                    }
                    match live.tool {
                        Tool::Transform => {
                            item.transform.translation += p.delta;
                            item.transform.rotation += p.rotation_delta.map(f64::to_radians);
                            item.transform.scale *= p.scale_delta;
                        }
                        Tool::Move => item.transform.translation += p.delta,
                        Tool::Rotate => item.transform.rotation += p.delta.map(f64::to_radians),
                        Tool::Scale => item.transform.scale *= p.delta,
                        _ => {}
                    }
                    // Planar spec: a Texture Locator's Size stops at 1 mm.
                    if item.texture_locator().is_some() {
                        item.transform.scale =
                            crate::texture_locator::clamp_size(item.transform.scale);
                    }
                } else {
                    // Every active mesh with a selection moves about the one
                    // shared frame (multi_transform.rs); just the primary
                    // when it is the only one.
                    let targets: Vec<(u64, std::collections::BTreeSet<u32>)> =
                        match live.base.component_transform_meshes() {
                            Some(ids) => ids
                                .into_iter()
                                .map(|id| Ok((id, live.base.mesh_transform_vertices(id)?)))
                                .collect::<Result<_>>()?,
                            None => vec![(
                                scene.active().ok_or("Select a mesh")?.id,
                                crate::transform_vertices(&live.base)?,
                            )],
                        };
                    for (item_id, mut vertices) in targets {
                        // FO-47/48/45: Soft Selection, Coplanar and Element
                        // falloffs also move the points they reach outside
                        // the selection.
                        if let Some(f) = self.active_falloff().filter(|f| f.reaches()) {
                            vertices.extend(f.reach(&live.base, item_id)?);
                        }
                        // The action frame is in world space: move each point there,
                        // apply the transform, and map the result back to local.
                        let world = live.base.world_matrix(item_id)?;
                        if world.determinant() == 0. || !world.determinant().is_finite() {
                            return Err("Component tools need an invertible item transform".into());
                        }
                        let item = scene.item_mut(item_id).ok_or("Select a mesh")?;
                        let frame = live.action_frame.ok_or("Missing action frame")?;
                        let angles = if live.tool == Tool::Transform {
                            p.rotation_delta
                        } else {
                            p.delta
                        };
                        let rotation = glam::DQuat::from_euler(
                            glam::EulerRot::XYZ,
                            angles.x.to_radians(),
                            angles.y.to_radians(),
                            angles.z.to_radians(),
                        );
                        // The pivot service's mirror rules (ACS-03..06): the
                        // primary side is the side the pivot is on; selected
                        // points there take the values as entered (typed or
                        // dragged alike), the other side's are mirrored,
                        // edited and mirrored back, and seam points go back
                        // onto the plane. Several meshes share one frame and
                        // each mirrors in its own space (MAM-Q09).
                        let symmetry = self.active_symmetry();
                        let map = crate::pivot_service::MirrorMap::new(
                            &live.base,
                            item_id,
                            symmetry,
                            frame.center,
                        )?;
                        // FAL-M-02 / FAL-B2-02/03: with a falloff each point moves to
                        // p + w·(T(p) − p), its weight's share of the full
                        // transform. With Symmetry the weight is read at the
                        // point's positive-side position (FAL-B2-04).
                        let falloff = self.active_falloff();
                        // VMF-03: a Vertex Map falloff reads the point's own
                        // value (copied out, the mesh is edited below).
                        // The selection-driven falloffs read the drag's
                        // starting mesh and selection (once per drag).
                        let prepared = match falloff.as_ref() {
                            Some(f) => f.prepare(&live.base, item_id)?,
                            None => Default::default(),
                        };
                        // TW-Q1: Twist and Swirl turn each point by weight ×
                        // Angle instead of blending positions (deform.rs).
                        let angle_weighted = live.tool == Tool::Rotate
                            && self.preset.as_ref().is_some_and(|p| p.angle_by_weight);
                        // Local: each connected piece about its own centre (F7).
                        let pieces =
                            (self.action_context.center == crate::CenterMode::Local).then(|| {
                                let selection = if live.base.selection.item == Some(item_id) {
                                    live.base.selection.clone()
                                } else {
                                    live.base.selection.sole(item_id)
                                };
                                crate::local_piece_frames(&item.mesh, &vertices, &selection)
                            });
                        for i in vertices {
                            let stored = &mut item.mesh.vertices[i as usize];
                            let role = map.role(*stored);
                            // Local: a point from the other side is edited in
                            // its island's mirror image (it is mirrored into
                            // the partner island first, ACS-04, ACS-10/11).
                            let frame = match pieces.as_ref().and_then(|p| p.get(&i)) {
                                Some(&piece) if role == crate::pivot_service::Role::Other => {
                                    piece.mirrored(symmetry).frame(world, frame.basis)
                                }
                                Some(&piece) => piece.frame(world, frame.basis),
                                None => frame,
                            };
                            // ACS-13: the weight is read where the point's
                            // primary-side image is.
                            let sample = map.falloff_sample(*stored);
                            let w = falloff.as_ref().map_or(1., |f| {
                                f.weight_at_local(
                                    world.transform_point3(sample),
                                    sample,
                                    i,
                                    prepared.inputs(),
                                )
                            });
                            let edit = |start: DVec3| -> DVec3 {
                                let v = start;
                                let moved = match live.tool {
                                    Tool::Transform => {
                                        frame.center
                                            + frame.basis
                                                * (rotation
                                                    * ((frame.basis.transpose()
                                                        * (v - frame.center))
                                                        * p.scale_delta)
                                                    + p.delta)
                                    }
                                    Tool::Move => v + frame.basis * p.delta,
                                    Tool::Rotate if angle_weighted => {
                                        let turned = angles * w;
                                        let rotation = glam::DQuat::from_euler(
                                            glam::EulerRot::XYZ,
                                            turned.x.to_radians(),
                                            turned.y.to_radians(),
                                            turned.z.to_radians(),
                                        );
                                        frame.center
                                            + frame.basis
                                                * (rotation
                                                    * (frame.basis.transpose()
                                                        * (v - frame.center)))
                                    }
                                    Tool::Rotate => {
                                        frame.center
                                            + frame.basis
                                                * (rotation
                                                    * (frame.basis.transpose()
                                                        * (v - frame.center)))
                                    }
                                    Tool::Scale => {
                                        frame.center
                                            + frame.basis
                                                * ((frame.basis.transpose() * (v - frame.center))
                                                    * p.delta)
                                    }
                                    _ => v,
                                };
                                if angle_weighted {
                                    moved
                                } else {
                                    start + (moved - start) * w
                                }
                            };
                            *stored = map.apply(*stored, edit);
                        }
                    }
                }
            }
        }
        // A drag that only moves points skips re-checking every polygon.
        if let Some(live) = &self.live {
            for item in &scene.items {
                if let Some(before) = live.base.item(item.id) {
                    item.mesh.shared().prove_points_moved(before.mesh.shared());
                }
            }
        }
        // ACS-31: a dragged guide's twin follows in the preview too.
        if let Some(live) = &self.live {
            crate::character_rig::sync(&mut scene, &live.base);
        }
        scene.validate()?;
        // Item-mode copies take new ids (DS-12).
        self.next_item_id = self.next_item_id.max(scene.next_id);
        self.scene = scene;
        if twist_note.is_some() {
            self.symmetry_note = twist_note;
        }
        let l = self.live.as_mut().ok_or("No tool")?;
        l.params = p;
        l.preview_error = None;
        l.applied = true;
        Ok(())
    }

    /// Twist Bevel on the active mesh (twist-bevel-2026-11-15): the shared
    /// pipe's falloff (one weight per tube, read at its base centre),
    /// symmetry (mirror-image tubes) and the action axis (Rotation Axis =
    /// Action Axis) go to `twist_bevel_geometry`. Returns the symmetry
    /// note, if any.
    fn preview_twist_bevel(
        &self,
        scene: &mut Scene,
        live: &Live,
        p: &Parameters,
    ) -> Result<Option<String>> {
        // PB-O-01 as Bevel: nothing selected works on every polygon and
        // leaves nothing selected.
        let everything = scene.selection.faces.is_empty();
        let item = scene.active().ok_or("Select an item")?;
        let selected: Vec<u32> = if everything {
            (0..item.mesh.faces.len() as u32).collect()
        } else {
            scene.selection.faces.clone()
        };
        let world = scene.world_matrix(item.id)?;
        // Distances are entered in world units on the item's own scale
        // (uniform scale assumed, as for Inset and Shift).
        let scale = crate::scene::uniform_scale(world);
        let options = crate::TwistBevelOptions {
            shift: p.twist_bevel.shift / scale,
            inset: p.twist_bevel.inset / scale,
            ..p.twist_bevel
        };
        // FAL (spec section 5, TB-Q05): a position falloff is read at each
        // tube's base centre; one that reads points (a weight map, the
        // selection) gives the mean of the tube's point weights.
        let field = self.active_falloff();
        let point_weights = match &field {
            Some(f) if f.reads_map() || f.reads_selection() => Some(f.weights(scene, item.id)?),
            _ => None,
        };
        let weight = |centre: DVec3, points: &[u32]| -> f64 {
            let Some(field) = &field else { return 1. };
            match &point_weights {
                Some(w) => {
                    points.iter().map(|&v| w[v as usize]).sum::<f64>() / points.len().max(1) as f64
                }
                None => field.weight_at_local(
                    world.transform_point3(centre),
                    centre,
                    0,
                    Default::default(),
                ),
            }
        };
        let symmetry = self.active_symmetry();
        let primary = self.symmetry_primary(scene);
        let action_axis = live
            .action_frame
            .map(|f| world.inverse().transform_vector3(f.basis.y_axis))
            .unwrap_or(DVec3::Y);
        let pipe = crate::TwistBevelPipe {
            weight: field
                .is_some()
                .then_some(&weight as &dyn Fn(DVec3, &[u32]) -> f64),
            symmetry: symmetry.axis.map(|_| (symmetry, primary)),
            action_axis,
        };
        let result = crate::twist_bevel_geometry(&item.mesh, &selected, &options, &pipe)?;
        let topology_changed = result.mesh != *item.mesh;
        scene.active_mut().unwrap().mesh = result.mesh.into();
        scene.selection.faces = if everything {
            Vec::new()
        } else {
            result.selected
        };
        if topology_changed {
            scene.selection.vertices.clear();
            scene.selection.edges.clear();
        }
        Ok(result.note)
    }
}

/// What `dispatch_inner` does after a command handler. Most handlers end by
/// advancing `revision`; a few finish early and must leave it unchanged.
pub(crate) enum AfterCommand {
    BumpRevision,
    Done,
}

/// The polygons an applied Extrude selects on Drop: the moving caps in
/// `faces` plus each one's kept bottom cap (`select_extrude_caps`). `None`
/// when the session has no applied stage.
pub(crate) fn extrude_caps_on_drop(l: &Live, faces: &[u32]) -> Option<Vec<u32>> {
    let (plan, _) = l.identity_preview.as_ref().filter(|_| l.applied)?;
    let source_faces = plan.source_layout.len();
    let mut out = faces.to_vec();
    let bottoms: Vec<u32> = faces
        .iter()
        .filter(|&&f| f as usize >= source_faces)
        .filter_map(|&f| plan.face_sources.get(f as usize).map(|&s| s as u32))
        .filter(|s| !faces.contains(s))
        .collect();
    out.extend(bottoms);
    out.sort_unstable();
    out.dedup();
    Some(out)
}

/// Index of a primitive tool in per-primitive settings (Cube, Sphere, Cylinder).
fn primitive_slot(tool: Tool) -> usize {
    match tool {
        Tool::Sphere => 1,
        Tool::Cylinder => 2,
        _ => 0,
    }
}

/// Sphere or Cylinder geometry from the live settings (`p`'s Make UVs,
/// axis, Cylinder caps and Sphere mode); `radial` gives size and resolution.
/// One flat `sides`-gon when exactly one radius is 0, facing that axis.
fn flat_disc(position: DVec3, radial: crate::RadialParameters) -> Result<Option<crate::Mesh>> {
    let zero: Vec<usize> = (0..3).filter(|&a| radial.radius[a] == 0.).collect();
    let [normal] = zero[..] else {
        return Ok(None);
    };
    crate::flat_ngon(
        position,
        normal,
        radial.radius[(normal + 1) % 3],
        radial.radius[(normal + 2) % 3],
        radial.sides,
    )
    .map(Some)
}

fn radial_mesh(
    tool: Tool,
    position: DVec3,
    radial: crate::RadialParameters,
    p: &Parameters,
) -> Result<crate::Mesh> {
    // Drag creation's flat base (spec drag-creation-2026-11-06 rule 1): with
    // one radius 0 the shape is one Sides-gon disc in the other two axes.
    if let Some(disc) = flat_disc(position, radial)? {
        return Ok(disc);
    }
    if tool != Tool::Sphere {
        crate::cylinder_geometry_with(position, radial, p.cylinder_caps, p.radial_axis, p.make_uvs)
    } else if let Some(level) = p.sphere_level {
        crate::tessellated_sphere(position, radial.radius, level, p.radial_axis, p.make_uvs)
    } else {
        crate::globe_geometry_with(position, radial, p.radial_axis, p.make_uvs)
    }
}
