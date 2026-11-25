//! Every tool's live settings (`Parameters`) and their Reset values.
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Parameters {
    pub bevel: crate::BevelOptions,
    /// Twist Bevel's properties (spec section 2).
    pub twist_bevel: crate::TwistBevelOptions,
    pub chamfer: crate::ChamferParameters,
    /// Edge Bevel / Edge Chamfer options; their width, rows and Depth are
    /// `chamfer.offset`, `chamfer.segments` and `chamfer.depth`.
    pub edge_bevel: crate::EdgeBevelOptions,
    pub poly_haul_offset: f64,
    /// Poly Haul's mode (spec poly-haul-modes-2026-11-06).
    pub poly_haul_mode: crate::PolyHaulMode,
    /// The element the drag started on, for the element modes.
    pub poly_haul_target: crate::HaulTarget,
    /// The other modes' amounts; Offset and Extrude use
    /// `poly_haul_offset`. `vector` is in world space.
    pub poly_haul_amount: crate::HaulAmount,
    pub sketch_path: Vec<DVec3>,
    pub sketch_precision: f64,
    pub smooth_shift: crate::SmoothShiftParameters,
    pub position: DVec3,
    pub size: DVec3,
    pub inset: f64,
    pub shift: f64,
    pub extent: DVec3,
    pub group_polygons: bool,
    pub extrude_sides: u32,
    pub extrude_mode: crate::ExtrudeMode,
    pub extrude_uvs: crate::ExtrudeUvs,
    pub extrude_uv_map: Option<String>,
    pub extrude_uv_selected: bool,
    pub delta: DVec3,
    pub rotation_delta: DVec3,
    pub scale_delta: DVec3,
    pub box_handle_mode: crate::BoxHandleMode,
    pub show_corners: bool,
    pub segments: [u32; 3],
    pub radial: Option<crate::RadialParameters>,
    /// Cube/Sphere/Cylinder generate the "Texture" UV map (provisional).
    pub make_uvs: bool,
    /// ME-T05 Slide: `shift` is a distance along each rail (ES-03) instead
    /// of a fraction of it. Vertex Slide's Interpolation uses it too
    /// (VTO-T07: one Slide tool serves points and edges).
    pub edge_slide_distance: bool,
    /// VTO-T07 Vertex Slide Merge Vertices (recorded default on): a point
    /// that lands on its neighbour merges with it.
    pub vertex_slide_merge: bool,
    /// VTO-T07 Vertex Slide Face Handle (recorded default on). Stored and
    /// shown only: no behaviour depends on it yet (VTO-Q07).
    pub vertex_slide_face_handle: bool,
    /// Slide options from the tool audit (Edge and Vertex Slide).
    pub slide_extras: crate::SlideExtras,
    /// VTO-T02/T03 Use Material for Vertex Bevel and Vertex Extrude
    /// (recorded default off): the new polygons get `vertex_material`.
    pub vertex_use_material: bool,
    /// VTO-T02/T03 Material Name (recorded default empty).
    pub vertex_material: String,
    /// VX-M05 Merge Tool Keep 1-Vertex Polygons (starting value off).
    pub vertex_merge_keep_small: bool,
    /// VX-M05 Merge Tool Test Morph Map Positions (starting value off).
    pub vertex_merge_test_morphs: bool,
    /// VX-B02 Vertex Bevel Round Level (starting value 0): each side of the
    /// cap becomes a fillet of 2 × level segments.
    pub vertex_round_level: u32,
    /// Scale's Negative Scale option: item scales may go below zero
    /// (IT-04/05). Off refuses them and leaves the item as it was.
    pub negative_scale: bool,
    /// Scale Absolute and Size Absolute (snaps-precision B, C): the Scale
    /// tool's absolute handles. `None` is the plain Scale tool.
    pub absolute_handles: Option<crate::AbsoluteHandles>,
    /// Edge Weight's Additive option (EW2-05): add to each edge's weight.
    pub edge_weight_additive: bool,
    /// Tweak: the element the current drag holds; `None` between drags.
    pub tweak: Option<crate::TweakGrab>,
    /// Tweak's Mode, Range, Shape and Connected Elements.
    pub tweak_options: crate::TweakOptions,
    /// What the current Tweak drag moves (weights and Symmetry
    /// counterparts), resolved at its first frame and reused after (see
    /// `tweak_plan`).
    pub tweak_plan: Option<crate::TweakPlan>,
    /// Topology Pen's modes and options (topology_pen.rs, TP-10..TP-40).
    pub topology_pen: crate::TopologyPenOptions,
    /// Cylinder bottom and top caps (PRIM-Y-01).
    pub cylinder_caps: [crate::CylinderCap; 2],
    /// Sphere and Cylinder axis: 0 X, 1 Y, 2 Z.
    pub radial_axis: usize,
    /// Sphere mode: None is Globe, Some(level) Tessellated (PRIM-S-02).
    pub sphere_level: Option<u32>,
    /// Push's Selection Normals; its Distance is `shift`.
    pub push_normals: crate::PushNormals,
    /// Smooth's options.
    pub smooth: crate::SmoothOptions,
    /// Deform finish tools' options (deform_finish.rs).
    pub jitter: crate::JitterOptions,
    pub quantize: crate::QuantizeOptions,
    pub radial_align: crate::RadialAlignOptions,
    pub linear_align: crate::LinearAlignOptions,
    /// Bend's options.
    pub bend: crate::BendOptions,
    /// The Symmetry Tool's options.
    pub symmetry_fix: crate::symmetry_fix::SymmetryFix,
    /// Loop Slice, Edge Slice and Polygon Slice options and cuts.
    pub slice: crate::SliceParameters,
    /// Contour's settings and drawn line (boxed like `duplicate`).
    pub contour: Box<crate::contour::ContourParameters>,
    /// The Duplicate tools' settings (boxed: `Parameters` sits on the
    /// stack in many places).
    pub duplicate: Box<crate::DuplicateParameters>,
    /// Edge Extend's Handles, Plane, Haul and Make UVs. Its Offset is
    /// `delta`, Rotate `rotation_delta` (degrees), Scale `scale_delta`,
    /// Local Inset/Shift `inset`/`shift` and Segments `segments[0]`.
    pub edge_extend: crate::EdgeExtendOptions,
    /// Set when Move / Rotate / Scale / Transform run in a UV view: they
    /// edit the selected UV map in U and V instead of points in 3D (UVE-1
    /// section 2, uv_edit.rs).
    pub uv: Option<crate::uv_edit::UvToolSpace>,
    /// The UV tab's live tools (E2, E3, E4).
    pub uv_project: crate::uv_project::ProjectOptions,
    pub uv_view_project: crate::uv_project::ViewProjectOptions,
    pub uv_unwrap: crate::uv_tools::UnwrapOptions,
    pub uv_relax: crate::uv_relax::RelaxOptions,
    pub uv_pelt: crate::uv_relax::PeltOptions,
    pub uv_peel: crate::uv_peel::PeelOptions,
    pub uv_box: crate::uv_box::BoxTransform,
}
impl Default for Parameters {
    fn default() -> Self {
        Self {
            bevel: Default::default(),
            twist_bevel: Default::default(),
            chamfer: Default::default(),
            edge_bevel: Default::default(),
            poly_haul_offset: 0.,
            poly_haul_mode: Default::default(),
            poly_haul_target: Default::default(),
            poly_haul_amount: Default::default(),
            sketch_path: Vec::new(),
            sketch_precision: 30.,
            smooth_shift: Default::default(),
            position: DVec3::ZERO,
            size: DVec3::ONE,
            inset: 0.,
            shift: 0.,
            extent: DVec3::ZERO,
            group_polygons: true,
            extrude_sides: 1,
            extrude_mode: Default::default(),
            extrude_uvs: Default::default(),
            extrude_uv_map: Some("Texture".into()),
            extrude_uv_selected: true,
            delta: DVec3::ZERO,
            rotation_delta: DVec3::ZERO,
            scale_delta: DVec3::ONE,
            box_handle_mode: crate::BoxHandleMode::Resize,
            show_corners: false,
            segments: [1; 3],
            radial: None,
            make_uvs: true,
            edge_slide_distance: false,
            vertex_slide_merge: true,
            vertex_slide_face_handle: true,
            slide_extras: Default::default(),
            vertex_use_material: false,
            vertex_material: String::new(),
            vertex_round_level: 0,
            vertex_merge_keep_small: false,
            vertex_merge_test_morphs: false,
            negative_scale: false,
            absolute_handles: None,
            edge_weight_additive: false,
            tweak: None,
            tweak_options: Default::default(),
            tweak_plan: None,
            topology_pen: Default::default(),
            cylinder_caps: [crate::CylinderCap::Single; 2],
            radial_axis: 1,
            sphere_level: None,
            push_normals: Default::default(),
            smooth: Default::default(),
            jitter: Default::default(),
            quantize: Default::default(),
            radial_align: Default::default(),
            linear_align: Default::default(),
            bend: Default::default(),
            symmetry_fix: Default::default(),
            slice: Default::default(),
            contour: Default::default(),
            duplicate: Default::default(),
            edge_extend: Default::default(),
            uv: None,
            uv_project: Default::default(),
            uv_view_project: Default::default(),
            uv_unwrap: Default::default(),
            uv_relax: Default::default(),
            uv_pelt: Default::default(),
            uv_peel: Default::default(),
            uv_box: Default::default(),
        }
    }
}

/// The Cube, Sphere and Cylinder tools' settings as their shapes, the one
/// generator each shares with its MeshOp (GitHub #14). Every field is
/// named, so a new shape option doesn't build until the tool fills it.
impl Parameters {
    pub fn cube_shape(&self) -> crate::CubeShape {
        crate::CubeShape {
            position: self.position,
            size: self.size,
            segments: self.segments,
            make_uvs: self.make_uvs,
        }
    }
    pub fn sphere_shape(
        &self,
        position: DVec3,
        radial: crate::RadialParameters,
    ) -> crate::SphereShape {
        crate::SphereShape {
            position,
            radius: radial.radius,
            sides: radial.sides,
            segments: radial.segments,
            axis: self.radial_axis,
            level: self.sphere_level,
            make_uvs: self.make_uvs,
        }
    }
    pub fn cylinder_shape(
        &self,
        position: DVec3,
        radial: crate::RadialParameters,
    ) -> crate::CylinderShape {
        crate::CylinderShape {
            position,
            radius: radial.radius,
            sides: radial.sides,
            segments: radial.segments,
            axis: self.radial_axis,
            caps: self.cylinder_caps,
            make_uvs: self.make_uvs,
        }
    }
}
