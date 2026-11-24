//! Every edit the editor takes (`Command`): what `Editor::dispatch` runs.
use super::*;

#[derive(Clone)]
pub enum Command {
    /// 3D painting: New Image, Add … Texture and the image commands.
    #[cfg(feature = "painting")]
    Paint(crate::paint::PaintEdit),
    FlipPolygons,
    /// SDV-01/02: Tab / Shift+Tab on the selected polygons (all when none).
    ToggleSubdivision(crate::subdivision::Toggle),
    /// SDV-03: the active mesh's subdivision levels and rules.
    SetSubdivision(crate::subdivision::SubdivisionSettings),
    /// FRZ-01..10 / SDV-12: freeze the active mesh's curves and (with
    /// Tessellation) its subdivision surfaces, one undo step.
    Freeze(crate::freeze::FreezeOptions),
    /// SS-01: the default material's smoothing. `continuing` folds the
    /// change into the previous Smoothing step (one undo step per field
    /// drag).
    SetSmoothing {
        settings: crate::shading::SmoothingSettings,
        continuing: bool,
    },
    /// The scene's final-render settings, one undo step.
    SetRenderSettings(crate::render_settings::RenderSettings),
    /// The scene's frame rate, time range and current time
    /// (scene_time.rs), one undo step.
    SetSceneTime(crate::scene_time::SceneTime),
    Pen(crate::PenInput),
    /// `tool.set solid.sketch on` (options: the bare tool's or `SS
    /// Default`'s). `resume` picks up the mesh's stored sketch (spec 5.5).
    SolidSketchStart {
        options: crate::solid_sketch::SketchOptions,
        resume: bool,
    },
    /// One edit of the live Solid Sketch; `continuing` joins the step of the
    /// edit before it (a drag's later frames).
    SolidSketch {
        edit: crate::SketchEdit,
        continuing: bool,
    },
    /// `operandi.solidSketch.strip` on the selected meshes.
    SolidSketchStrip,
    PenStart,
    PenCapture(DVec3),
    /// Make Quads: add one point that ends the strip with a triangle
    /// (Ctrl+click).
    PenCaptureEnd(DVec3),
    PenSelect(usize),
    /// Reverse the Pen draft's point order (so its polygon faces the other
    /// way); the current point follows its point.
    PenReverse,
    PenRefine(DVec3),
    /// Remove draft point `index` (SPEC-DRAW-POINT-MODE DPM-10, D1); with
    /// `and_after` every later point goes too. In a Make Quads strip the
    /// point's whole pair goes (D2).
    PenDelete {
        index: usize,
        and_after: bool,
    },
    /// Interactive Pen panel options (Type, Flip, Close, Select New).
    PenOptions(crate::PenOptions),
    BeginExtrude,
    /// First viewport click of an Extrude when the I2 stage does not apply:
    /// build the extrusion at zero distance (walls in place, caps unmoved).
    StartExtrude,
    ConfigureSmoothShift(crate::SmoothShiftParameters),
    ResetSmoothShift,
    CloneSmoothShift,
    SurfaceStateSave(String),
    SurfaceStateRecall(Option<String>),
    SurfaceStateRename(String),
    SurfaceStateDelete,

    ArmBridge,
    ConfigureBridge(crate::BridgeParameters),
    ResetBridge,
    ArmSpike,
    ConfigureSpike(f64),
    ResetSpike,
    /// RED-001 `tool.set poly.reduct on`: arm with Number 0.
    ArmReduction,
    /// RED-002 stationary viewport click: capture the source and its count.
    InitializeReduction,
    /// RED-002 Number (and unavailable weight/Legacy refusals).
    ConfigureReduction(crate::ReductionParameters),
    ArmEllipsoid,
    ConfigureEllipsoid(crate::EllipsoidParameters),
    ResetEllipsoid,
    /// Put the live Cube/Sphere/Cylinder's segment settings back to their
    /// starting values and forget the ones kept from earlier uses.
    ResetPrimitive,
    /// Turn the live Cube/Sphere/Cylinder's generated UVs on or off.
    PrimitiveUvs(bool),
    InsertEllipsoidUnit(UnitDestination),
    /// Edit group (clipboard.rs). Copy is `Editor::copy_selection`: it
    /// changes nothing in the scene, so it isn't a command.
    CutSelection,
    PasteSelection,
    /// One Paste Tool copy.
    PasteAt(crate::PastePlacement),
    /// Merge group (item_merge.rs).
    MergeItems,
    /// Draw tools: add, change or remove one curve (one undo step).
    SetCurve(Box<crate::CurveEdit>),
    /// Sketch / Draw Patch: add world-space geometry (one undo step).
    AddGeometry(Box<crate::curve_edit::AddGeometry>),
    UnmergeItem,
    /// Add an item to the item selection or take it out (Ctrl/Shift-click
    /// in the Scenes list).
    ToggleItemSelection(u64),
    /// Align group (align_tools.rs).
    GroundAlign(crate::GroundAlign),
    /// Tack (TC-20..22): the selection onto a picked polygon, one step.
    Tack(crate::Tack),
    /// Relax Mesh... (`sculpt.relaxMesh`, DP-20..22): relax the selected
    /// points (whole mesh when none), one step.
    RelaxMesh(crate::RelaxMeshOptions),
    /// Select tab tools (select_tools.rs, SL-xx): one undo step that
    /// changes only the selection.
    SelectTool(crate::select_tools::SelectRequest),
    /// Merge Instance (merge_instance.rs, MT12).
    MergeInstance,
    /// Freeze Replicator (replicator.rs, G8 RP-5): the selected Replicator
    /// becomes instance items in its group.
    FreezeReplicator,
    /// A Replicator's settings (Properties, replicator.rs, G8 RP-2).
    SetReplicator {
        item: u64,
        settings: Box<crate::replicator::Replicator>,
    },
    /// Vertex Normal Tools (vertex_normals.rs, G9 VN-1..VN-10).
    VertexNormals(Box<crate::vertex_normals::NormalCommand>),
    /// The scene's Guides (guides.rs, G9 GD-1..GD-3).
    SetGuides(Box<crate::guides::Guides>),
    /// One Mesh Paint click or stroke (mesh_paint.rs, MP-16).
    MeshPaint(Box<crate::mesh_paint::MeshPaintStroke>),
    /// Diagonal Re-Direct (diagonal_redirect.rs, MT4).
    DiagonalRedirect,
    /// Edge Zip (edge_zip.rs, MT3).
    EdgeZip,
    /// Fix Black Dots (fix_black_dots.rs, MT2).
    FixBlackDots,
    /// Triple Fan (topology_cleanup.rs, TC-01..03).
    TripleFan,
    /// Collapse Selection (TC-10..12).
    CollapseSelection,
    /// Topology Pen Duplicate (TC-45): pull a new quad from edge `a`–`b`
    /// of `item`, its new edge moved by `offset` (world) onto the
    /// background.
    TopologyDuplicate {
        item: u64,
        a: u32,
        b: u32,
        offset: DVec3,
    },
    /// Topology Pen v2 edits (topology_pen.rs), one undo step each.
    TopologyPen(Box<crate::PenEdit>),
    QuickAlign(crate::QuickAlign),
    /// Unit Cone, Torus or Capsule (unit_insert.rs).
    InsertShapeUnit(crate::UnitShape, UnitDestination),
    /// Start Torus, Capsule, Arc or Tube with its last settings (again on
    /// the same tool: Drop it).
    ArmShape(crate::ShapeKind),
    ConfigureShape(crate::ShapeParameters),
    /// Put the active primitive back to its Reset values.
    ResetShape,
    ArmCone,
    ConfigureCone(crate::ConeParameters),
    InsertUnit {
        tool: Tool,
        destination: UnitDestination,
    },
    Activate(Tool),
    /// Deform tab presets by their handle; the same as `ActivatePreset`
    /// with the preset's row id.
    ActivateDeform(crate::DeformPreset),
    /// Start a tool preset row by id or phrase (tool_preset.rs, TP-10).
    ActivatePreset(String),
    /// Flip It / Flip It Origin (tool presets 6b): mirror the selected
    /// polygons across `axis` (0 X, 1 Y, 2 Z) through the row's action
    /// centre, facing out. One undo step.
    FlipIt {
        preset: String,
        axis: usize,
    },
    ActionContext(crate::ActionContext),
    /// Move a live Move/Rotate/Scale/Transform's pivot to a world point
    /// before it has been applied (Jason, 2026-11-03: a click away from the
    /// model puts the handle there; provisional, VAC-Q02).
    SetActionCenter(DVec3),
    /// A click in a 3D view that places the live tool's centre (CS-20,
    /// BN-24/30a/30b): `point` is where the click landed (snapped point,
    /// mesh surface or work plane) and `plane` the view's work plane
    /// (0 ground XZ, 1 side YZ, 2 front XY). Bend moves its centre there,
    /// refits its spine and puts its Axis on the plane's normal.
    ToolClick {
        point: DVec3,
        plane: u8,
    },
    /// Put a live Move/Rotate/Scale/Transform's pivot back where its
    /// Action Center puts it for the selection, undoing `SetActionCenter`
    /// (Jason, 2026-11-15: a press on the selected element brings the
    /// widget back; provisional, VAC-Q02).
    ResetActionCenter,
    /// End of one Move/Rotate/Scale/Transform drag: record the change so far
    /// as its own undo step and keep the tool live from the result (Jason,
    /// 2026-11-03). Nothing to record is not an error.
    CommitTransformStep,
    /// HM-02: Shift-drag with a transform tool takes up the drag just
    /// committed again, so its values keep growing instead of starting from
    /// zero. Nothing to continue (no drag yet, or the scene changed since)
    /// is not an error: the drag then starts fresh.
    ContinueTransformStep,
    /// HM-06 in Vertex/Edge/Polygon mode: copy the whole polygons the
    /// selection covers into the same mesh and select the copy, so a
    /// transform tool then moves the copy and leaves the original.
    DuplicateSelectedPolygons,
    /// Set the shared Symmetry axis and offset.
    Symmetry(crate::Symmetry),
    /// Symmetry on/off with one key (symmetry-2026-11-13 section 8, Q2):
    /// off remembers the setting, on brings it back (X the first time).
    ToggleSymmetry,
    /// SYM-SEL-01: add the counterparts of the selected elements.
    SelectSymmetrical,
    /// SYM-SZ-01: make the active mesh exactly symmetric from one side.
    Symmetrize(crate::SymmetrizeSide),
    /// Symmetrize (polygons): delete the target half, mirror the source
    /// half across, merge the seam (symmetry-2026-11-13 section 9).
    SymmetrizePolygons {
        side: crate::SymmetrizeSide,
        merge: bool,
    },
    Preview(Parameters),
    ConfigureBevel(Parameters),
    ConfigureChamfer(crate::ChamferParameters),
    ChamferOnce(crate::ChamferParameters),
    PolyHaulOnce(f64),
    /// Poly Haul settings back to their defaults (Offset 0, spec PH-D0).
    ResetPolyHaul,
    /// Sketch Extrude settings back to their defaults (Precision 30, SE-D0).
    ResetSketchExtrude,
    /// Merge Tool Distance back to 0.001 (VTO-D0-05).
    ResetVertexMerge,
    /// Bend back to Angle 0 and Spine (0, 1 m, 0) (BN-47).
    ResetBend,
    SketchExtrudeOnce(Vec<DVec3>),
    ResetBevel,
    /// MN-05: Reset Tool Attributes: the active tool's values back to the
    /// factory, whichever tool it is (`tool_reset.rs`).
    ResetToolAttributes,
    BevelStateSave(String),
    BevelStateRecall(Option<String>),
    BevelStateRename(String),
    BevelStateDelete,
    ResetBevelState,
    BevelProfile(Option<crate::BevelProfile>),
    SpatialFalloff(Option<crate::Falloff>),
    /// Constrain to Background on (with these options) or off. A live tool
    /// stays and re-applies.
    BackgroundConstraint(Option<crate::background_constraint::BackgroundConstraint>),
    /// The falloffs added on top of the active one (VMF-10), replacing the
    /// list. Refused with no active falloff.
    AddedFalloffs(Vec<(crate::Falloff, crate::MixMode)>),
    /// The active falloff's own Mix Mode (FO-11). Refused with no active
    /// falloff.
    SpatialFalloffMix(crate::MixMode),
    /// The Element falloff's pick (FO-45), or none. A live tool stays.
    FalloffElementPick(Option<crate::TweakGrab>),
    /// The Screen, Lasso and Airbrush stamps, per (layer, item), replacing
    /// them (FO-Q08). A live tool re-applies.
    FalloffStamps(std::collections::BTreeMap<(usize, u64), Vec<f64>>),
    /// The Path falloff's control points (FO-53), replacing them. A live
    /// tool re-applies.
    FalloffPath(Vec<glam::DVec3>),
    /// Select a weight map by name, or none (VM-02).
    SelectWeightMap(Option<String>),
    ConfigureExtrude(Parameters),
    ResetExtrude,
    Apply,
    Continue,
    Drop,
    Cancel,
    Undo,
    Redo,
    /// New Mesh Item: an empty top-level mesh, the only selected item.
    NewMesh,
    /// The item list's New: like New Mesh, but beside the selected item
    /// (same parent; spec new-mesh NM08).
    NewMeshBeside,
    NewNamedMesh(String),
    /// BD-01 Create Backdrop: a new Backdrop item at the origin, selected.
    NewBackdrop,
    /// BD-03 load an image file (already read by the app for its pixel
    /// size) and show it on Backdrop `item`. A file the scene already has
    /// reuses its entry.
    BackdropLoadImage {
        item: u64,
        path: String,
        width: u32,
        height: u32,
    },
    /// One Backdrop panel control.
    EditBackdrop {
        item: u64,
        edit: crate::backdrop::BackdropEdit,
    },
    /// BD-30..33, BD-61 Auto Fit to the active meshes' box.
    BackdropAutoFit(u64),
    /// BD-60 an image dropped into a 3D view, or `backdrop.add` with
    /// arguments: a new Backdrop (BD-01 origin and sole selection, its name
    /// or "Backdrop Item"; BD-02 defaults) facing `projection` and showing
    /// `image` (path and pixel size, loaded as an image entry, BD-03), if
    /// given. One undo step (provisional, BD-Q21).
    AddBackdrop {
        name: Option<String>,
        image: Option<(String, u32, u32)>,
        projection: crate::backdrop::Projection,
    },
    /// BD-60 an image dropped outside a 3D view (or `clip.addStill`): only
    /// a scene image entry (BD-03), no Backdrop.
    LoadImage {
        path: String,
        width: u32,
        height: u32,
    },
    /// BD-19 Backdrop Size (`height` set) or Backdrop Size Keep Aspect
    /// (`height` none); provisional, BD-Q08.
    BackdropSize {
        item: u64,
        width: f64,
        height: Option<f64>,
    },
    /// IL-03 Add Item: one new item of that type with factory defaults at
    /// the origin, alone selected, placed directly after the selected item
    /// as its sibling (end of the root list with nothing selected).
    AddItem(crate::item_list::AddItemKind),
    /// IL-05 Select popover action on the items `query` matches.
    ItemPattern {
        query: crate::item_list::PatternQuery,
        action: crate::item_list::PatternAction,
    },
    /// CL-1: a new camera at the origin with default settings, selected.
    NewCamera,
    /// CL-1/CL-2 Item > Create Light: a new light of that kind at the
    /// origin with an identity transform, alone selected (CL-Q03).
    NewLight(crate::camera_light::LightKind),
    /// One environment property (its item settings or its Environment
    /// Material), one undo step named after the property (ENV §11).
    EditEnvironment {
        item: u64,
        edit: crate::environment::EnvironmentEdit,
    },
    /// Add, delete or change a texture layer under an environment (ENV
    /// §5, §8), one undo step.
    EditEnvironmentLayers {
        item: u64,
        edit: crate::environment::EnvironmentLayerEdit,
    },
    /// One camera panel control, applied to `item` and to every other
    /// selected camera (CP-X3). One undo step named after the property.
    EditCamera {
        item: u64,
        edit: crate::camera_light::CameraEdit,
    },
    /// CP-E03 Autofocus: camera `item`'s Focus Distance to the surface
    /// under the centre of its view. One undo step.
    CameraAutofocus(u64),
    /// CP-V26: read a picture for camera `item`'s Background Image (one
    /// undo step).
    CameraBackgroundLoad {
        item: u64,
        path: String,
        width: u32,
        height: u32,
    },
    /// CP-V12 Sync to View: move camera `item` to the view's placement.
    SyncCameraToView {
        item: u64,
        transform: crate::scene::Transform,
        target_distance: f64,
    },
    /// CP-V07 / spec freeze-transforms section 3 Reset: part of every
    /// item's transform back to identity, one undo step (`transform.reset`).
    ResetTransform {
        items: Vec<u64>,
        part: crate::camera_props::TransformReset,
    },
    /// Spec freeze-transforms section 4 Freeze: part of every mesh item's
    /// transform baked into its points, nothing moving; one undo step
    /// (`transform.freeze`, item_transform_ops.rs).
    FreezeTransform {
        items: Vec<u64>,
        part: crate::camera_props::TransformReset,
    },
    /// One light panel control, applied to `item` and to every other
    /// selected light (LP-51). One undo step named after the property.
    EditLight {
        item: u64,
        edit: crate::camera_light::LightEdit,
    },
    /// One Display or Assembly tab control (item_common.rs), applied to
    /// every item in `items` (IC-2). One undo step named after the
    /// property (IC-1).
    EditItemCommon {
        items: Vec<u64>,
        edit: crate::item_common::ItemCommonEdit,
    },
    /// Geometry lock or quick hide (element_state.rs, LCK-30..33,
    /// HID-10..13); one undo step each.
    EditElements {
        kind: crate::ElementKind,
        op: crate::ElementOp,
    },
    /// Item groups (item_groups.rs), each one undo step: New Group from
    /// `members`, Rename, Delete (items stay), Add / Remove items.
    NewGroup {
        name: Option<String>,
        members: Vec<u64>,
    },
    RenameGroup {
        group: u64,
        name: String,
    },
    DeleteGroup(u64),
    AddToGroup {
        group: u64,
        items: Vec<u64>,
    },
    RemoveFromGroup {
        group: u64,
        items: Vec<u64>,
    },
    /// Select a group's members (Item mode), like clicking items: not an
    /// undo step.
    SelectGroup(u64),
    /// Select exactly these items, the first one primary (a Shift range or
    /// a parent's double-click in the Items list, UIC-31/38). Not an undo
    /// step.
    SelectItems(Vec<u64>),
    /// LP-PH-01/02 Photometry File: read the IES file at `path` and set the
    /// light's file, Radiant Intensity and Width/Height from it. One undo
    /// step; a bad file changes nothing.
    LoadPhotometry {
        item: u64,
        path: String,
    },
    /// Make `item` the scene's render camera.
    SetRenderCamera(u64),
    /// CL-14: with Move, Rotate or Transform active on a selected camera or
    /// light in Item mode, edit one of its own channels from the tool's
    /// panel or handles. A tweak of the tool: its own undo step inside the
    /// tool, kept on Drop, undone by Cancel.
    ToolItemChannel {
        item: u64,
        edit: crate::camera_light::ItemChannelEdit,
    },
    /// CL-1 Camera view navigation: move camera `item` (its own transform)
    /// and set its Target Distance, as one undo step per press-to-release.
    /// Light view navigation moves a light the same way; a light has no
    /// Target Distance, so `target_distance` is ignored for it.
    CameraView {
        item: u64,
        transform: crate::scene::Transform,
        target_distance: f64,
    },
    DeleteItem,
    /// Delete every selected item, deleting their children too or moving
    /// them to the root in place (item_delete.rs). One undo step. Plain
    /// `DeleteItem` on an item with children moves them to the root.
    DeleteItems(crate::item_delete::ChildrenOnDelete),
    /// POP-PHY-10 Duplicate: a coincident copy of the selected item, named
    /// "Name (2)" (the next free number), alone selected.
    DuplicateItem,
    /// DX-38 Instance: an instance of each selected mesh item in the same
    /// place (duplicate_items.rs); the instances end up selected.
    InstanceItems,
    /// DX-51 Duplicate Hierarchy: each selected item copied with all its
    /// children, links kept (duplicate_hierarchy.rs). One undo step.
    DuplicateHierarchy,
    /// Items arriving from another open scene (a drag between scenes): the
    /// first is the branch root, the rest its descendants, as made by
    /// `Scene::branch`. They get new ids and free names; the root goes
    /// under `parent` keeping its world position. Provisional (MS-Q10).
    InsertItems {
        items: Vec<crate::scene::Item>,
        parent: Option<u64>,
        /// The source scene's pictures the items' Backdrops show; each is
        /// matched to this scene's entry for the same file, or added.
        images: Vec<crate::backdrop::SceneImage>,
    },
    /// Remove an item and all its descendants (the source side of moving
    /// items to another scene).
    RemoveBranch(u64),
    ItemTransform {
        item: u64,
        transform: crate::scene::Transform,
    },
    Rename(String),
    Visibility(u64),
    /// Show or hide several items in one undo step (an eye sweep or Alt+click
    /// in a tree list, a group's eye; UIC-138). Locked items are left as
    /// they are (LCK-05); all of them locked is refused.
    SetItemsVisible {
        items: Vec<u64>,
        visible: bool,
        /// Hiding: the items' eyes also hide their descendants ("all off",
        /// Auto Hide Child Items on, GV-01); the children's own eyes are
        /// left alone. Ignored when showing.
        hides_children: bool,
    },
    /// Toggle an item's visibility and give all its descendants the same
    /// visibility (Jason, 2026-11-02). No eye click uses it since
    /// GV-01..06 (2026-11-10); scripts still can.
    VisibilityWithChildren(u64),
    /// SEL-IT-01..03: parent `item` to `parent` (None unparents). With
    /// `keep_position` the item stays where it is in the world and its own
    /// transform changes; without it the transform is kept and it moves.
    SetParent {
        item: u64,
        parent: Option<u64>,
        keep_position: bool,
    },
    /// GL-04 Parent to Group Locator: the selected items go under one new
    /// Group Locator, which becomes the only selected item (locator.rs).
    ParentToGroupLocator,
    /// UIC-143 tree drop: put `items` under `parent` (None: top level),
    /// just before the sibling `before` (None: last). Reparented items
    /// keep their world position. One undo step: "Reorder" when no parent
    /// changed, else "Parent" / "Unparent".
    MoveItems {
        items: Vec<u64>,
        parent: Option<u64>,
        before: Option<u64>,
    },
    /// Drop Action (drop-action-2026-11-06): `sources` dropped onto
    /// `destination` in the viewport; one undo step named after the action.
    /// `ray` is the eye-to-pointer ray Place and Place and Align use.
    DropAction {
        action: crate::DropAction,
        sources: Vec<u64>,
        destination: u64,
        ray: Option<crate::drop_action::Ray>,
    },
    Mode(Mode),
    /// Polygon tag selection mode (note 04): Polygon mode where a click
    /// selects whole tagged regions of `Editor::polygon_tag_kind`.
    /// `convert` (Alt+click) grows the current selection to its regions.
    EnterTagMode {
        convert: bool,
    },
    /// The global Polygon Tag Type (Select > Polygon Tag Type). In a tag
    /// mode it changes what clicks select; the mode stays.
    SetPolygonTagKind(crate::PolygonTagKind),
    SelectItem(Option<u64>),
    Select {
        item: u64,
        hit: crate::Component,
        replace: bool,
        remove: bool,
    },
    BeginSelection,
    EndSelection {
        cancel: bool,
    },
    ClearSelection,
    Selection(crate::SelectionOperation),
    /// Polygon Selection Fill from a double-clicked polygon inside the perimeter.
    SelectionFill {
        item: u64,
        face: u32,
    },
    /// Edge-mode double-click: select edge `a`-`b` of `item` and its edge
    /// loop; `add` (Shift) adds the loop to the existing edge selection.
    SelectEdgeLoop {
        item: u64,
        a: u32,
        b: u32,
        add: bool,
    },
    /// Polygon-mode double-click: select polygon `face` of `item` and every
    /// polygon connected to it; `add` (Shift) adds that piece to the
    /// existing polygon selection.
    SelectConnectedPolygons {
        item: u64,
        face: u32,
        add: bool,
    },
    SelectPolygonSet(String),
    /// SEL-081 Assign Selection Set: save the current Vertex, Edge or
    /// Polygon selection of the active mesh under a name (adds to an
    /// existing set of that name).
    AssignSelectionSet(String),
    /// SEL-082 Use Selection Set: select (Add), deselect (Remove) or
    /// Replace with a current-mode set.
    UseSelectionSet {
        name: String,
        mode: crate::UseSetMode,
    },
    /// SEL-082 Delete Selection Set: one current-mode set by name, or all of
    /// them with `None`. Never deletes geometry.
    DeleteSelectionSet(Option<String>),
    /// Create Selection Set Group: a named group of current-mode sets (all
    /// of them when the list is empty).
    CreateSelectionSetGroup {
        name: String,
        sets: Vec<String>,
    },
    /// Delete a selection set group; its sets stay.
    DeleteSelectionSetGroup(String),
    /// One-shot Set Part on the selected polygons, or all of them when
    /// nothing is selected (SP-05).
    SetPart(crate::PartAssignment),
    SetPosition(crate::PositionAssignment),
    /// One Absolute Scaling button press (absolute_scaling.rs).
    AbsoluteScale(crate::AbsoluteScale),
    SetMaterial(crate::MaterialAssignment),
    /// Shading panel edits (ST-1): add, delete, move, eye, layer edits.
    ShaderTree(crate::ShaderTreeEdit),
    /// Load a preset (preset.rs, PB-31): one undo step. `group` is the
    /// Material Group a material preset goes into.
    LoadPreset {
        preset: Box<crate::preset::Preset>,
        group: Option<crate::shader_tree::NodeId>,
    },
    /// Add a material preset's layers to the shader tree without replacing
    /// anything (`operandi.materialx.import`): directly above `beside`, in
    /// its group, or where Add Layer puts a group with none. One undo step.
    InsertMaterialPreset {
        preset: Box<crate::preset::Preset>,
        beside: Option<crate::shader_tree::NodeId>,
    },
    /// Make the picture at `path` an environment's image (preset browser
    /// CD-13, Load Image…, an HDR dropped on a view): one undo step.
    /// `width`/`height` are its pixel size. `item` is the environment;
    /// `None` is the top (newest) one, made when there is none.
    SetEnvironmentImage {
        path: String,
        width: u32,
        height: u32,
        item: Option<u64>,
    },
    /// Apply environment preset `preset` (EP §Apply) as `mode`, one undo
    /// step; its lights and meshes remember `name` (the preset's name).
    ApplyEnvironmentPreset {
        preset: Box<crate::preset::Preset>,
        mode: crate::preset::ApplyMode,
        name: String,
    },
    /// A material preset dropped onto geometry (PB-110): polygons `faces`
    /// of `item` (all when empty) get a new tag and the preset's material.
    ApplyMaterialPreset {
        preset: Box<crate::preset::Preset>,
        item: u64,
        faces: Vec<usize>,
    },
    UnifyPolygons(crate::UnifyRequest),
    SplitPolygon,
    QuadruplePolygon,
    SpinQuads,
    /// MP-C08 one-shot Align of an explicitly selected adjacent quad pair.
    AlignPolygons,
    /// MP-C04 one-shot Triple on the explicit polygon selection.
    Triple,
    /// MP-C10 one-shot Remove of the explicit polygon selection.
    RemovePolygons,
    /// Forward Delete in a component mode (`delete`): the elements go with
    /// the points they leave unused (Jason, 2026-11-10).
    DeleteComponents,
    /// MP-C09 one-shot Merge of the explicit polygon selection.
    MergePolygons,
    /// MP-C11 one-shot Collapse of one explicitly selected simple quad.
    CollapsePolygon,
    /// MP-C14 one-shot Flatten Ngons of the active mesh, scoped by an explicit
    /// polygon selection (core only; FN-NP-1A).
    FlattenNgons,
    /// MP-C13 one-shot Scale Offset of one isolated convex polygon by a
    /// signed boundary distance in scene units (positive contracts).
    ScaleOffsetPolygon(f64),
    /// PolyScale Offset Detach: inset the selected polygons onto new
    /// points, leaving them unconnected (spec polyscale-offset B4).
    ScaleOffsetDetach(f64),
    /// CLEAN-001: remove floating vertices from an explicit idle Item target.
    CleanupFloating(Option<crate::CleanupTarget>),
    /// Mesh Cleanup with its twelve options (`CleanupOption` order).
    MeshCleanup {
        flags: [bool; 12],
        target: Option<crate::CleanupTarget>,
    },
    /// MP-C01 one-shot Make: fill one four-corner opening from the two
    /// ordered selected boundary edges (`poly.make auto`).
    MakePolygon,
    /// Make Polygon with an explicit Type and Close Curve (the menu entry's
    /// dialog); `MakePolygon` is the P key's Auto.
    MakePolygonAs {
        kind: crate::MakeType,
        close: bool,
    },
    /// CC-01 Make Open / Closed Curve through the selected vertices in
    /// selection order (curve_commands.rs).
    MakeCurve {
        closed: bool,
    },
    /// CC-10 Make Curve Fill on the selected closed curves.
    MakeCurveFill,
    /// CC-20 Align Winding of the selected curves.
    AlignWinding(crate::curve_commands::WindingOptions),
    /// CB-xx Curve Boolean on the selected curves (curve_boolean.rs).
    CurveBoolean(crate::curve_boolean::CurveBooleanOptions),
    /// CC-50..53 Convert Curves into a new item (curve_convert.rs).
    ConvertCurves {
        from: crate::curve_convert::CurveType,
        to: crate::curve_convert::CurveType,
    },
    /// CC-40..43 Curve Offset (Trio: both sides).
    OffsetCurves {
        closed: bool,
        offset: f64,
        trio: bool,
    },
    /// CC-60..64 Bezier Tangent on the selected knots.
    BezierTangent(crate::curve_convert::TangentOptions),
    /// GO-20 UV Rectangle (generator_ops.rs).
    UvRectangle(crate::generator_ops::RectangleOptions),
    /// Dicer (dicer.rs): cut the active mesh into pieces.
    Dicer(crate::dicer::DicerOptions),
    /// GO-30 Transfer Vertex Map from the background mesh.
    TransferVertexMap {
        map: String,
        kind: crate::generator_ops::TransferType,
        /// Sampling Space World (placements count) or Local (GO-33).
        world: bool,
        sampling: crate::generator_ops::TransferSampling,
    },
    /// ME-C01..C13 one-shot Edge command rows on the selected edges.
    EdgeCommand(crate::EdgeCommand),
    /// CMD-B3-01 Subdivide with a Count (intervals, at least 2) or, with
    /// Count 1, a Percent along each edge.
    SubdivideEdges {
        count: u32,
        percent: Option<f64>,
    },
    /// SUB-05..15 Subdivide Polygons (`poly.subdivide`), `times` passes as
    /// one undo step (Alt+click SDS Subdivide: 2).
    SubdividePolygons {
        options: crate::SubdivideOptions,
        times: u32,
    },
    /// SUB-40..44 UnSubdivide (`poly.unsubdiv`).
    UnsubdividePolygons(crate::UnsubdivideOptions),
    /// Detriangulate the selected polygons (all when none), one step.
    Detriangulate(crate::DetriangulateOptions),
    /// UV Unwrap of the visible polygons into the target UV map, one step.
    UvUnwrap(crate::uv_tools::UnwrapOptions),
    /// Atlas Unwrap of the visible polygons into the target UV map, one step.
    UvAtlas(crate::uv_atlas::AtlasOptions),
    /// UV Relax of the UV map as one step (`uv.relax` command form).
    UvRelax(crate::uv_relax::RelaxOptions),
    /// The UV map list (UVE-1 section 1): new, select, rename, delete,
    /// copy, paste.
    UvMap(crate::uv_maps::UvMapEdit),
    /// One island affine run (E7: Fit, Flip, Mirror, Axis Align).
    UvIsland(crate::uv_edit::IslandOp),
    /// One of the UV tab's other buttons (uv_ops.rs: align, orient,
    /// texel density, split and sew, UV queries and values).
    UvOp(crate::uv_ops::UvOp),
    /// The Texel Density panel's values (no undo step).
    UvTexel(crate::uv_ops::TexelOptions),
    /// The Cut Map popover's seam map buttons (uv_seams.rs).
    UvSeam(crate::uv_seams::SeamEdit),
    /// The UV Symmetry layer (no undo step, like 3D Symmetry).
    UvSymmetry(crate::uv_symmetry::UvSymmetry),
    /// UV Symmetry's Center button: weld the UVs on the line.
    UvSymmetryCenter,
    /// Symmetrize UVs (UVT-71).
    UvSymmetrize(crate::uv_symmetry::SymmetrizeOptions),
    /// Convert UVs to Mesh (UVT-120): a new flat mesh item.
    UvToMesh,
    /// Transfer UVs (UVT-92): the primary mesh's map to the others.
    UvTransfer,
    /// A click on an E7 button of the tool preset table.
    UvIslandPreset {
        preset: String,
        click: crate::uv_edit::Click,
    },
    /// Put the open Move / Rotate / Scale / Transform in UV space on the
    /// named map (`None` in the name: the mesh's selected map), or back in
    /// 3D with `None`. Only before the tool has changed anything.
    UvToolSpace(Option<Option<String>>),
    /// Pack UVs: the islands of the target UV map, one step.
    PackUvs(crate::uv_tools::PackOptions),
    /// `mesh.retopo`, `sculpt.remesh` or `operandi.remesh` on the active
    /// mesh, one step (remesh.rs).
    Remesh(crate::remesh::RemeshOp),
    /// Select Connected in a UV view (UVV-04): the UV islands of the
    /// selection in that map (the mesh's UV map when `None`).
    SelectUvIslands(Option<String>),
    /// BOOL-01..25 Boolean (`poly.boolean`) of the active mesh, one step.
    Boolean(crate::BooleanOptions),
    /// Mesh Stack (mesh_stack_editor.rs): add an op of this type to the
    /// selected mesh (`meshop.create`).
    MeshOpCreate(String),
    /// MeshBlend recipe edits (meshblend/commands.rs).
    MeshBlend(crate::meshblend::MeshBlendCommand),
    /// Delete ops (`item.delete item:<op>`).
    MeshOpDelete(Vec<crate::mesh_stack_editor::OpRef>),
    /// Move an op to a position in evaluation order (`item.parent`).
    MeshOpMove {
        op: crate::mesh_stack_editor::OpRef,
        position: usize,
    },
    /// An op's eye (`deformer.enable`).
    MeshOpEnable {
        op: crate::mesh_stack_editor::OpRef,
        on: bool,
    },
    /// Set op values by attribute name (`item.channel ... item:<op>`).
    MeshOpSet {
        op: crate::mesh_stack_editor::OpRef,
        values: serde_json::Map<String, serde_json::Value>,
    },
    /// Select a row of the Mesh Operations list (`select.item <op>`).
    MeshOpSelect(Option<crate::mesh_stack_editor::OpRow>),
    /// Select by Index Set/Clear/Add/Remove (`selop.selection`).
    MeshOpIndex {
        op: crate::mesh_stack_editor::OpRef,
        edit: crate::mesh_stack_editor::IndexEdit,
    },
    /// Add, change or remove one of an op's selection ops (`selop.create`,
    /// `selop.edit`).
    MeshOpSelOp {
        op: crate::mesh_stack_editor::OpRef,
        edit: crate::mesh_stack_editor::SelOpEdit,
    },
    /// Freeze an op and those below it (`deformer.freeze`).
    MeshOpFreeze {
        op: crate::mesh_stack_editor::OpRef,
        duplicate: bool,
    },
    /// Freeze the selected mesh's whole stack (`mesh.freeze`).
    MeshOpFreezeAll,
    /// Edit Base Mesh on or off (`deformer.selectBaseMesh`).
    MeshOpEditBase {
        item: u64,
        on: bool,
    },
    /// DRILL-01..27 Solid Drill (`poly.solidDrill`) of the active mesh.
    SolidDrill(crate::DrillOptions),
    /// DRILL-01..27 Axis Drill (`poly.drill`) of the active mesh.
    AxisDrill(crate::DrillOptions),
    /// CEN-01..04 Center (`vert.center`) on the world axes that are on.
    CenterSelection([bool; 3]),
    /// M3 Vertex command rows (MV-C04..C15) on the selected points.
    VertexCommand(crate::VertexCommand),
    /// MV-C01 New Vertex at a world position.
    NewVertex(DVec3),
    /// MV-T05 Weld `sources` onto `target`.
    WeldVertices {
        sources: Vec<u32>,
        target: u32,
    },
    /// WD-01..05: one Weld drag of point `source`, released onto another
    /// point (merge) or at a world position (move). One undo step.
    WeldDrag {
        source: u32,
        drop: crate::WeldDrop,
    },
    /// One finished sculpt stroke (sculpt.rs, SC-90): the points it moved,
    /// one undo step named after the brush.
    SculptStroke(Box<crate::SculptEdit>),
    /// One finished Vertex Map Tools stroke (VMT-19): the weights it set,
    /// one undo step named after the tool.
    WeightStroke(Box<crate::sculpt::WeightEdit>),
    /// New Weight Map (VM-03) on the active meshes; `value` fills every
    /// point (Init Value), `None` leaves them with no value. Selects it.
    NewWeightMap {
        name: String,
        value: Option<f64>,
    },
    /// MV-C16 Sync Index Order from a captured source item and two polygons
    /// to the active item's two selected polygons.
    SyncVertexOrder {
        source_item: u64,
        source_faces: [u32; 2],
    },
    /// ME-T09 Add Point: split edge `a`–`b` of the active mesh at `t` (0..1).
    AddEdgePoint {
        a: u32,
        b: u32,
        t: f64,
    },
    /// ME-T08 Add Loop: cut the polygon band crossing edge `a`–`b` at `t`.
    AddEdgeLoop {
        a: u32,
        b: u32,
        t: f64,
    },
    /// Add Point / Add Loop with placement options (Both Sides, Absolute
    /// Distance, Preserve Curvature). Distances are world metres.
    AddEdgePointWith {
        a: u32,
        b: u32,
        t: f64,
        options: crate::EdgeInsertOptions,
    },
    AddEdgeLoopWith {
        a: u32,
        b: u32,
        t: f64,
        options: crate::EdgeInsertOptions,
    },
    Workplane(crate::Workplane),
}
