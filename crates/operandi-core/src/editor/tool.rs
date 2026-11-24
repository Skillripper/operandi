//! The tool list (`Tool`): ids, names and the families each belongs to.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Box,
    Sphere,
    Cylinder,
    Bevel,
    /// Twist Bevel (twist-bevel-2026-11-15): options in `twist_bevel`
    /// (`twist_bevel.rs`).
    TwistBevel,
    Chamfer,
    PolyHaul,
    SketchExtrude,
    Extrude,
    SmoothShift,
    Thicken,
    Transform,
    Move,
    Rotate,
    Scale,
    /// ME-T03 Extend: Offset X/Y/Z in `delta`, Segments in `segments[0]`.
    EdgeExtend,
    /// ME-T02 Edge Extrude: Extrude distance in `shift`, Width in `inset`.
    EdgeExtrude,
    /// ME-T05 Slide: amount (−1..1 of the rail) in `shift`.
    EdgeSlide,
    /// ME-T10 / EW-07 Edge Weight: weight (-1..1, shown as %) in `shift`.
    EdgeCrease,
    /// ME-T01 Edge Bevel: Inset in `chamfer.offset`, rows (2·Round Level, EBC-B3-02)
    /// in `chamfer.segments`, Depth in `chamfer.depth`.
    EdgeBevel,
    /// ME-T07 Edge Chamfer: parallel Offset in `chamfer.offset`, Segments in
    /// `chamfer.segments`, Depth in `chamfer.depth`.
    EdgeChamfer,
    /// ME-T06 Edge Relax: Offset in `shift`.
    EdgeRelax,
    /// MV-T01 Create: clicked world points in `sketch_path`.
    VertexCreate,
    /// MV-T02 Vertex Bevel: Inset in `inset`.
    VertexBevel,
    /// MV-T03 Vertex Extrude: Extrude in `shift`, Width in `inset`.
    VertexExtrude,
    /// MV-T04 Merge Tool: Distance in `shift`.
    VertexMerge,
    /// MV-T06 Tear: world drag offset in `delta`.
    VertexTear,
    /// MV-T07 Slide (Vertex mode): amount (−1..1) in `shift`, world rail
    /// direction in `delta`.
    VertexSlide,
    /// Tweak (Element Move, key T): drags the element in `tweak` by the
    /// world offset in `delta` (TWK-1).
    Tweak,
    /// Deform tab Push: every selected point moves along its normal by
    /// Distance (`shift`), scaled by the falloff.
    Push,
    /// Deform tab Smooth: `smooth` options; relaxes the selected points.
    Smooth,
    /// Deform finish (deform_finish.rs): `jitter` options; random offsets.
    Jitter,
    /// `quantize` options; snaps points to a world grid.
    Quantize,
    /// Radial Align (`xfrm.radialAlign`): `radial_align` options.
    RadialAlign,
    /// Linear Align (`xfrm.linearAlign`): `linear_align` options.
    LinearAlign,
    /// Deform tab Bend (`xfrm.bend`): `bend` options; curls the target into
    /// an arc (bend.rs).
    Bend,
    /// Symmetry Tool (symmetry-2026-11-13 section 9, SYM-B3-09): dragging
    /// raises `symmetry_fix.threshold`; points whose mirror partner is
    /// within it snap to match (symmetry_fix.rs).
    SymmetryTool,
    /// Edit tab Slice tools (toolbox-panels-2026-11-06): options and placed
    /// cuts in `slice` (`slice.rs`).
    LoopSlice,
    EdgeSlice,
    PolygonSlice,
    /// Polygon Inset (spec inset-2026-11-05): distance in `inset`.
    Inset,
    /// Polygon Shift (tool audit Tier 1, provisional): distance in `shift`.
    Shift,
    /// Edit tab wave 2 path slicing (WAVE2-PATH-SLICE): paths and the
    /// shared cutter in `slice` (`path_slice.rs`).
    CurveSlice,
    PenSlice,
    PrimitiveSlice,
    /// Edit tab wave 3 Polygon Axis Slice (WAVE3-AXIS-SLICE), `slice.axis_slice`.
    AxisSlice,
    /// Topology tab Contour (missing-tools-g4, contour.rs), `contour`.
    Contour,
    /// Duplicate tab (specifications/duplicate-tools): settings in
    /// `duplicate` (`duplicate.rs`, Lathe in `duplicate_sweep.rs`).
    Mirror,
    Lathe,
    Clone,
    Array,
    RadialArray,
    /// Path tools (CX-xx, path_tools.rs): a path generator feeding a Sweep
    /// (Curve / Bezier / B-Spline Extrude) or a Clone (Curve / Bezier /
    /// B-Spline / Pen Clone); settings in `duplicate.curve_extrude` and
    /// `duplicate.curve_clone`.
    CurveExtrude,
    CurveClone,
    /// Scatter Clone (GO-01..07): settings in `duplicate.scatter`.
    ScatterClone,
    /// Symmetrize, the Duplicate tab tool (DX-40..45): settings in
    /// `duplicate.symmetrize` (symmetrize_tool.rs).
    Symmetrize,
    /// UV tab Project (E2, UVT-10..18): `uv_project`; writes the selected
    /// polygons' UVs from a projection (uv_project.rs).
    UvProject,
    /// Project from View (UVT-19): `uv_view_project`; a click in a 3D view
    /// writes the selected polygons' screen positions as UVs.
    UvViewProject,
    /// UV Unwrap as a live tool (UVS-1 §1): `uv_unwrap`.
    UvUnwrap,
    /// UV Relax (E4, UVS-1 §3): `uv_relax` (uv_relax.rs).
    UvRelax,
    /// Unwrap & Relax (UVS-1 §1.5, UVT-20): Unwrap then Relax, `uv_pelt`.
    UvPelt,
    /// Peeler (E5, UVT-30): `uv_peel`; quad strips laid out as a grid.
    UvPeeler,
    /// Box Transform (UVT-43): `uv_box`; UV-only, through the UV pipe.
    UvBoxTransform,
}
impl Tool {
    pub(super) fn initial_radial(self) -> Option<crate::RadialParameters> {
        match self {
            Self::Sphere => Some(crate::RadialParameters {
                radius: DVec3::splat(0.5),
                sides: 24,
                segments: 12,
            }),
            Self::Cylinder => Some(crate::RadialParameters {
                radius: DVec3::new(0.5, 0.75, 0.5),
                sides: 24,
                segments: 1,
            }),
            _ => None,
        }
    }
    /// Bevel and Twist Bevel: live from the first edit, Shift+click starts
    /// a fresh one on the caps, one undo step per bevel (TA-01..07).
    pub fn is_bevel(self) -> bool {
        matches!(self, Self::Bevel | Self::TwistBevel)
    }
    /// Smooth and the Deform finish point tools show their result for the
    /// panel's values as soon as they are live; drags and fields then
    /// rework that one result.
    pub fn shows_result_at_once(self) -> bool {
        matches!(
            self,
            Self::Smooth
                | Self::Jitter
                | Self::Quantize
                | Self::RadialAlign
                | Self::LinearAlign
                | Self::UvProject
                | Self::UvUnwrap
                | Self::UvRelax
                | Self::UvPelt
                | Self::UvPeeler
                | Self::Symmetrize
        )
    }
    pub fn is_smooth_shift(self) -> bool {
        matches!(self, Self::SmoothShift | Self::Thicken)
    }
    pub fn is_primitive(self) -> bool {
        self.card().family == crate::tool_card::Family::Primitive
    }
    /// Each drag is its own undo step and Undo/Redo keep the tool active
    /// (Move, Rotate, Scale, Transform and Tweak; `ToolCard::undo_per_drag`).
    pub fn is_transform(self) -> bool {
        self.card().undo_per_drag
    }
    /// Moves whole items in Item mode (Move, Rotate, Scale and Transform;
    /// `ToolCard::transforms_items`).
    pub fn transforms_items(self) -> bool {
        self.card().transforms_items
    }
    /// Every tool, in toolbox order. `all_lists_every_tool` fails to compile
    /// when a new variant is not added here too.
    pub const ALL: [Tool; 63] = [
        Tool::Box,
        Tool::Sphere,
        Tool::Cylinder,
        Tool::Bevel,
        Tool::TwistBevel,
        Tool::Chamfer,
        Tool::PolyHaul,
        Tool::SketchExtrude,
        Tool::Extrude,
        Tool::SmoothShift,
        Tool::Thicken,
        Tool::Transform,
        Tool::Move,
        Tool::Rotate,
        Tool::Scale,
        Tool::EdgeExtend,
        Tool::EdgeExtrude,
        Tool::EdgeSlide,
        Tool::EdgeCrease,
        Tool::EdgeBevel,
        Tool::EdgeChamfer,
        Tool::EdgeRelax,
        Tool::VertexCreate,
        Tool::VertexBevel,
        Tool::VertexExtrude,
        Tool::VertexMerge,
        Tool::VertexTear,
        Tool::VertexSlide,
        Tool::Tweak,
        Tool::Push,
        Tool::Smooth,
        Tool::Jitter,
        Tool::Quantize,
        Tool::RadialAlign,
        Tool::LinearAlign,
        Tool::Bend,
        Tool::LoopSlice,
        Tool::EdgeSlice,
        Tool::PolygonSlice,
        Tool::Inset,
        Tool::Shift,
        Tool::CurveSlice,
        Tool::PenSlice,
        Tool::PrimitiveSlice,
        Tool::AxisSlice,
        Tool::Contour,
        Tool::Mirror,
        Tool::Lathe,
        Tool::Clone,
        Tool::Array,
        Tool::RadialArray,
        Tool::CurveExtrude,
        Tool::CurveClone,
        Tool::ScatterClone,
        Tool::Symmetrize,
        Tool::SymmetryTool,
        Tool::UvProject,
        Tool::UvViewProject,
        Tool::UvUnwrap,
        Tool::UvRelax,
        Tool::UvPelt,
        Tool::UvPeeler,
        Tool::UvBoxTransform,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Box => "Cube",
            Self::Sphere => "Sphere",
            Self::Cylinder => "Cylinder",
            Self::Bevel => "Bevel",
            Self::TwistBevel => "Twist Bevel",
            Self::Chamfer => "Chamfer Edit",
            Self::PolyHaul => "Poly Haul",
            Self::SketchExtrude => "Sketch Extrude",
            Self::Extrude => "Extrude",
            Self::SmoothShift => "Smooth Shift",
            Self::Thicken => "Thicken",
            Self::Transform => "Transform",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::Scale => "Scale",
            Self::EdgeExtend => "Extend",
            Self::EdgeExtrude => "Edge Extrude",
            Self::EdgeSlide => "Slide",
            Self::EdgeCrease => "Edge Weight",
            Self::EdgeBevel => "Edge Bevel",
            Self::EdgeChamfer => "Edge Chamfer",
            Self::EdgeRelax => "Edge Relax",
            Self::VertexCreate => "Create",
            Self::VertexBevel => "Vertex Bevel",
            Self::VertexExtrude => "Vertex Extrude",
            Self::VertexMerge => "Merge Tool",
            Self::VertexTear => "Tear",
            Self::VertexSlide => "Vertex Slide",
            // The reference's name (Jason, 2026-11-06); the variant keeps
            // its old name.
            Self::Tweak => "Element Move",
            Self::Push => "Push",
            Self::Smooth => "Smooth",
            Self::Jitter => "Jitter",
            Self::Quantize => "Quantize",
            // The toolbox buttons say Radial and Linear (DEFORM §1).
            Self::RadialAlign => "Radial Align",
            Self::LinearAlign => "Linear Align",
            Self::Bend => "Bend",
            Self::SymmetryTool => "Symmetry Tool",
            Self::LoopSlice => "Loop Slice",
            Self::EdgeSlice => "Edge Slice",
            Self::PolygonSlice => "Polygon Slice",
            Self::Inset => "Inset",
            Self::Shift => "Shift",
            Self::CurveSlice => "Curve Slice",
            Self::PenSlice => "Pen Slice",
            Self::PrimitiveSlice => "Primitive Slice",
            Self::AxisSlice => "Polygon Axis Slice",
            Self::Contour => "Contour",
            Self::Mirror => "Mirror",
            // DS-61 (spec v1.4, Jason): the button says Radial Sweep;
            // "lathe" stays a command-line alias. The variant keeps its name.
            Self::Lathe => "Radial Sweep",
            Self::Clone => "Clone",
            Self::Array => "Array",
            Self::RadialArray => "Radial Array",
            Self::CurveExtrude => "Curve Extrude",
            Self::CurveClone => "Curve Clone",
            Self::ScatterClone => "Scatter Clone",
            Self::Symmetrize => "Symmetrize",
            Self::UvProject => "Project",
            Self::UvViewProject => "Project from View",
            Self::UvUnwrap => "UV Unwrap",
            Self::UvRelax => "UV Relax",
            Self::UvPelt => "Unwrap & Relax",
            Self::UvPeeler => "Peeler",
            Self::UvBoxTransform => "Box Transform",
        }
    }
}
