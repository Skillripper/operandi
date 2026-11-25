//! Operandi's own MeshOp types (Mesh Stack v1, SCOPE Stage 0 first batch):
//! their values and how they run. Each is a [`TypedOp`] with a table entry
//! in [`builtin`]; the stack itself lives in `mesh_stack.rs`.
//!
//! The ops call the direct tools' kernels (`bevel_geometry_applied`,
//! `smooth_shift_geometry`, `copy_faces` ...), never their live sessions.
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub(crate) fn finite(values: &[f64], what: &str) -> Result<()> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(format!("{what} must be a finite number"))
    }
}

/// Bevel-like results: `front` = the moved polygons, `sides` = the new
/// polygons around them (provisional, MOQ-22: new polygons past the
/// input's count that aren't fronts).
fn front_and_sides(ctx: &OpContext, mut mesh: Mesh, front: &[u32]) -> Mesh {
    let n = ctx.input.faces.len() as u32;
    let sides: Vec<u32> = (n..mesh.faces.len() as u32)
        .filter(|f| !front.contains(f))
        .collect();
    ctx.tag(&mut mesh, "front", front.iter().copied());
    ctx.tag(&mut mesh, "sides", sides);
    mesh
}
/// Clone ops' groups (PG table): `clone` = every copy, `clone1`.. = each
/// copy in order; the original is not a clone.
fn clones(ctx: &OpContext, mut mesh: Mesh, new_faces: usize, per_copy: usize) -> Mesh {
    let copies = new_faces.checked_div(per_copy).unwrap_or(0);
    let end = mesh.faces.len() as u32;
    // Copies land after the input's polygons; anything else (merged
    // away) gets no groups rather than wrong ones.
    let Some(start) = end.checked_sub(new_faces as u32) else {
        return mesh;
    };
    ctx.tag(&mut mesh, "clone", start..end);
    if copies > 0 && new_faces.is_multiple_of(copies) {
        let per = (new_faces / copies) as u32;
        for k in 0..copies as u32 {
            let a = start + k * per;
            ctx.tag(&mut mesh, &format!("clone{}", k + 1), a..a + per);
        }
    }
    mesh
}

// Op values use the reference tools' attribute names (registry thread,
// MESHOPS-NAMES 2026-11-11), so `item.channel <name> <value> item:<op>`
// ports. Distances in metres, angles in radians, percents as fractions
// (1.0 = 100%). Only the attributes Operandi builds are stored; an
// unknown name is refused.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BevelOp {
    pub shift: f64,
    pub inset: f64,
    /// Group Polygons: bevel touching polygons as one region.
    pub group: bool,
}
impl Default for BevelOp {
    fn default() -> Self {
        Self {
            shift: 0.,
            inset: 0.,
            group: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SmoothShiftOp {
    pub shift: f64,
    /// 1.0 = 100%.
    pub scale: f64,
    /// Max. Smoothing Angle, radians.
    pub max_angle: f64,
    pub thicken: bool,
    pub sharp: bool,
    pub angle_weight: bool,
    pub both_sides: bool,
}
impl Default for SmoothShiftOp {
    fn default() -> Self {
        let p = crate::SmoothShiftParameters::default();
        Self {
            shift: p.offset,
            scale: p.scale,
            max_angle: p.angle.to_radians(),
            thicken: p.thicken,
            sharp: p.sharp,
            angle_weight: p.angle_weight,
            both_sides: p.both_sides,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MirrorOp {
    /// 0 X, 1 Y, 2 Z: the plane's normal.
    pub axis: u32,
    #[serde(rename = "cenX")]
    pub cen_x: f64,
    #[serde(rename = "cenY")]
    pub cen_y: f64,
    #[serde(rename = "cenZ")]
    pub cen_z: f64,
    /// Merge Vertices on the plane, within `dist`.
    pub merge: bool,
    pub dist: f64,
}
impl Default for MirrorOp {
    fn default() -> Self {
        let tool = crate::duplicate::Mirror::default();
        Self {
            axis: 0,
            cen_x: 0.,
            cen_y: 0.,
            cen_z: 0.,
            merge: tool.copies.merge,
            dist: tool.copies.distance,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrayOp {
    /// Cells along X, Y, Z, the original included.
    #[serde(rename = "numX")]
    pub num_x: u32,
    #[serde(rename = "numY")]
    pub num_y: u32,
    #[serde(rename = "numZ")]
    pub num_z: u32,
    #[serde(rename = "offX")]
    pub off_x: f64,
    #[serde(rename = "offY")]
    pub off_y: f64,
    #[serde(rename = "offZ")]
    pub off_z: f64,
    /// Merge Vertices, within `dist` (Clone Effector channels).
    pub merge: bool,
    /// Missing from v1 files, which saved Array flat (`array.clone.item`).
    #[serde(default)]
    pub dist: f64,
}
impl Default for ArrayOp {
    fn default() -> Self {
        let tool = crate::duplicate::Array::default();
        Self {
            num_x: tool.count[0],
            num_y: tool.count[1],
            num_z: tool.count[2],
            off_x: tool.offset.x,
            off_y: tool.offset.y,
            off_z: tool.offset.z,
            merge: tool.copies.merge,
            dist: tool.copies.distance,
        }
    }
}

/// subdivide.tool attributes (registry rev 2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubdivideOp {
    /// 0 Faceted, 1 Smooth, 2 SDS Subdivide, 3 Catmull-Clark (dialog
    /// order; the reference default is 3, the rest unpublished, SUBT-Q01).
    #[serde(rename = "type")]
    pub method: u32,
    /// Max. Smoothing Angle, radians.
    pub angle: f64,
    #[serde(rename = "linearUV")]
    pub linear_uv: bool,
    /// 0 Smooth All, 1 Crease All, 2 Crease Edges.
    pub boundary: u32,
    #[serde(rename = "noEdgeWeight")]
    pub no_edge_weight: bool,
}
impl Default for SubdivideOp {
    fn default() -> Self {
        let o = crate::SubdivideOptions::default();
        Self {
            method: crate::SubdivideMethod::ALL
                .iter()
                .position(|m| *m == o.method)
                .unwrap_or(3) as u32,
            angle: o.max_angle_degrees.to_radians(),
            linear_uv: o.linear_uvs,
            boundary: 0,
            no_edge_weight: o.ignore_edge_weights,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriangulateOp {}

// ---- running ----

impl TypedOp for BevelOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let o = self;
        finite(&[o.shift, o.inset], "Shift and Inset")?;
        let whole = ctx.chosen()?.is_empty();
        let faces = ctx.chosen_or_all()?;
        let options = crate::BevelOptions::default();
        let bevel = |group: bool| {
            crate::bevel_geometry_applied(
                ctx.input, &faces, o.shift, o.inset, group, &options, None, None,
            )
        };
        let r = match bevel(o.group) {
            Ok(r) => r,
            // A whole closed mesh has no region edge to group by; bevel
            // each polygon instead (MOQ-11, provisional).
            Err(_) if whole && o.group => bevel(false)?,
            Err(e) => return Err(e),
        };
        Ok(front_and_sides(ctx, r.mesh, &r.selected))
    }
}
impl TypedOp for SmoothShiftOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let o = self;
        finite(&[o.max_angle], "Max. Smoothing Angle")?;
        let p = crate::SmoothShiftParameters {
            offset: o.shift,
            scale: o.scale,
            angle: o.max_angle.to_degrees(),
            thicken: o.thicken,
            sharp: o.sharp,
            angle_weight: o.angle_weight,
            both_sides: o.both_sides,
        };
        let r = crate::smooth_shift_geometry(ctx.input, &ctx.chosen()?, p, None)?;
        Ok(front_and_sides(ctx, r.mesh, &r.selected))
    }
}
impl TypedOp for MirrorOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let o = self;
        let center = DVec3::new(o.cen_x, o.cen_y, o.cen_z);
        finite(&center.to_array(), "Center")?;
        finite(&[o.dist], "Distance")?;
        if o.axis > 2 {
            return Err("Mirror Axis must be 0 (X), 1 (Y) or 2 (Z)".into());
        }
        if o.dist < 0. {
            return Err("Mirror Distance can't be negative".into());
        }
        let axis = o.axis as usize;
        let (left, up) = crate::duplicate::Mirror::axis_frame(axis);
        let tool = crate::duplicate::Mirror {
            axis,
            center,
            left,
            up,
            ..Default::default()
        };
        let copies = crate::duplicate::Copies {
            merge: o.merge,
            distance: o.dist,
            ..tool.copies
        };
        let source = faces_usize(ctx.chosen_or_all()?);
        let r = crate::duplicate::copy_faces(
            ctx.input,
            &source,
            &[tool.transform()?],
            &copies,
            crate::duplicate::WeldAt::Mean,
            copies.distance,
        )?;
        Ok(clones(ctx, r.mesh, r.new_faces, source.len()))
    }
}
impl TypedOp for ArrayOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let o = self;
        let offset = DVec3::new(o.off_x, o.off_y, o.off_z);
        finite(&offset.to_array(), "Offset")?;
        let count = [o.num_x, o.num_y, o.num_z];
        if count.contains(&0) {
            return Err("Array needs at least 1 cell on every axis".into());
        }
        let source = faces_usize(ctx.chosen_or_all()?);
        let centre = faces_centre(ctx.input, &source);
        let mut tool = crate::duplicate::Array {
            count,
            offset,
            ..Default::default()
        };
        tool.copies.merge = o.merge;
        finite(&[o.dist], "Distance")?;
        if o.dist < 0. {
            return Err("Array Distance can't be negative".into());
        }
        tool.copies.distance = o.dist;
        let transforms: Vec<DMat4> = crate::duplicate::array_transforms(&tool, centre)?;
        let r = crate::duplicate::copy_faces(
            ctx.input,
            &source,
            &transforms,
            &tool.copies,
            crate::duplicate::WeldAt::Original,
            tool.copies.distance,
        )?;
        Ok(clones(ctx, r.mesh, r.new_faces, source.len()))
    }
}
impl TypedOp for SubdivideOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let o = self;
        finite(&[o.angle], "Max. Smoothing Angle")?;
        let method = *crate::SubdivideMethod::ALL
            .get(o.method as usize)
            .ok_or("Subdivide type must be 0 to 3")?;
        let boundary = *crate::polygon_subdivide::SubdivideBoundary::ALL
            .get(o.boundary as usize)
            .ok_or("Subdivide boundary must be 0 to 2")?;
        let options = crate::SubdivideOptions {
            method,
            max_angle_degrees: o.angle.to_degrees(),
            linear_uvs: o.linear_uv,
            boundary,
            ignore_edge_weights: o.no_edge_weight,
            ..Default::default()
        };
        // Fractal isn't an op value; a fixed seed keeps it repeatable
        // (SAFE-11).
        Ok(
            crate::polygon_subdivide::subdivide_polygons(ctx.input, &ctx.chosen()?, &options, 1)?
                .mesh,
        )
    }
}
impl TypedOp for TriangulateOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let input = ctx.input;
        let faces: Vec<u32> = ctx
            .chosen_or_all()?
            .into_iter()
            .filter(|&f| {
                !input.is_polyline(f as usize) && !input.attributes.degenerate_faces.contains(&f)
            })
            .collect();
        if faces.is_empty() {
            return Ok(input.clone());
        }
        Ok(match crate::triple_faces(input, &faces)? {
            Some(outcome) => outcome.mesh,
            None => input.clone(),
        })
    }
}

pub(crate) fn faces_usize(faces: Vec<u32>) -> Vec<usize> {
    faces.into_iter().map(|f| f as usize).collect()
}
pub(crate) fn faces_centre(mesh: &Mesh, faces: &[usize]) -> DVec3 {
    let mut lo = DVec3::splat(f64::INFINITY);
    let mut hi = DVec3::splat(f64::NEG_INFINITY);
    for &f in faces {
        for &v in &mesh.faces[f].vertices {
            let p = mesh.vertices[v as usize];
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    if lo.is_finite() && hi.is_finite() {
        (lo + hi) / 2.
    } else {
        DVec3::ZERO
    }
}

// ---- the table entries ----

pub(crate) fn xyz(prefix: &'static str, label: &'static str, kind: FieldKind) -> [Field; 3] {
    // Keys like `cenX`; labels like `Position X`.
    let f = |a: &str| Field {
        key: format!("{prefix}{a}").into(),
        label: format!("{label} {a}").into(),
        kind: kind.clone(),
    };
    [f("X"), f("Y"), f("Z")]
}
pub(crate) fn distance(key: &'static str, label: &'static str) -> Field {
    Field::new(key, label, FieldKind::Distance)
}
pub(crate) fn check(key: &'static str, label: &'static str) -> Field {
    Field::new(key, label, FieldKind::Check)
}
pub(crate) fn count(key: &'static str, label: &'static str, min: u32, max: u32) -> Field {
    Field::new(key, label, FieldKind::Count { min, max })
}
pub(crate) fn choice(key: &'static str, label: &'static str, names: &[&'static str]) -> Field {
    Field::new(
        key,
        label,
        FieldKind::Choice(names.iter().map(|n| (*n).into()).collect()),
    )
}
const FRONT_SIDES: &[(&str, &str)] = &[("front", "Front"), ("sides", "Sides")];
const CLONE_GROUPS: &[(&str, &str)] = &[
    ("clone", "Cloned Polygons"),
    ("clone1", "Clone 1"),
    ("clone2", "Clone 2"),
    ("clone3", "Clone 3"),
    ("clone4", "Clone 4"),
    ("clone5", "Clone 5"),
    ("clone6", "Clone 6"),
    ("clone7", "Clone 7"),
    ("clone8", "Clone 8"),
];

/// Operandi's own op types, in Add Operator order. Ids are the reference
/// tool ids (registry thread, MESHOPS-NAMES rev 3): what files store and
/// `meshop.create` takes.
pub(crate) fn builtin() -> Vec<Arc<dyn OpDef>> {
    let mut mirror = vec![choice("axis", "Axis", &["X", "Y", "Z"])];
    mirror.extend(xyz("cen", "Position", FieldKind::Distance));
    mirror.extend([
        check("merge", "Merge Vertices"),
        distance("dist", "Distance"),
    ]);
    let mut array = Vec::new();
    array.extend(xyz(
        "num",
        "Count",
        FieldKind::Count {
            min: 1,
            max: 100_000,
        },
    ));
    array.extend(xyz("off", "Offset", FieldKind::Distance));
    array.extend([
        check("merge", "Merge Vertices"),
        distance("dist", "Distance"),
    ]);
    let max_angle = |key| Field::new(key, "Max Smoothing Angle", FieldKind::Angle);
    // The primitives, each through its tool's one generator (primitive_ops.rs).
    let mut table: Vec<Arc<dyn OpDef>> = crate::primitive_ops::entries();
    table.extend::<Vec<Arc<dyn OpDef>>>(vec![
        Arc::new(
            Typed::<BevelOp>::new(
                "poly.bevel.item",
                "Polygon Bevel",
                "Polygon",
                true,
                vec![
                    distance("shift", "Shift"),
                    distance("inset", "Inset"),
                    check("group", "Group Polygons"),
                ],
            )
            .with_groups(FRONT_SIDES),
        ),
        Arc::new(
            Typed::<SmoothShiftOp>::new(
                "poly.smshift.item",
                "Smooth Shift",
                "Polygon",
                true,
                vec![
                    distance("shift", "Shift"),
                    Field::new("scale", "Scale", FieldKind::Percent),
                    max_angle("maxAngle"),
                    check("thicken", "Thicken"),
                    check("sharp", "Sharp"),
                    check("angleWeight", "Angle Weight"),
                    check("bothSides", "Both Sides"),
                ],
            )
            .with_groups(FRONT_SIDES),
        ),
        Arc::new(Typed::<SubdivideOp>::new(
            "subdivide.tool.item",
            "Subdivide",
            "Polygon",
            true,
            vec![
                choice(
                    "type",
                    "Method",
                    &["Faceted", "Smooth", "SDS Subdivide", "Catmull-Clark"],
                ),
                max_angle("angle"),
                check("linearUV", "Linear UVs"),
                choice(
                    "boundary",
                    "Boundary",
                    &["Smooth All", "Crease All", "Crease Edges"],
                ),
                check("noEdgeWeight", "Ignore Edge Weights"),
            ],
        )),
        Arc::new(Typed::<TriangulateOp>::new(
            "pmodel.triangulate.item",
            "Triangulate",
            "Polygon",
            true,
            Vec::new(),
        )),
        Arc::new(
            Typed::<MirrorOp>::new("mirror.clone.item", "Mirror", "Duplicate", true, mirror)
                .with_groups(CLONE_GROUPS),
        ),
        Arc::new(
            Typed::<ArrayOp>::new("array.clone.item", "Array", "Duplicate", true, array)
                .with_groups(CLONE_GROUPS),
        ),
    ]);
    // Ops that read other items (Boolean, Merge Meshes).
    table.extend(crate::mesh_stack_items::entries());
    table.extend(crate::meshblend::op::entries());
    // Direct tools as ops, by group (mesh_stack_tools.rs).
    table.extend(crate::mesh_stack_tools::polygon_entries());
    table.extend(crate::mesh_stack_tools::edge_entries());
    table.extend(crate::mesh_stack_tools::vertex_entries());
    table.extend(crate::mesh_stack_tools::duplicate_entries());
    table.extend(crate::mesh_stack_tools::edit_entries());
    table.extend(crate::mesh_stack_tools::deform_entries());
    table.extend(crate::mesh_stack_tools::uv_entries());
    // The Deform finish tools' ops (mesh_stack_deform.rs).
    table.extend(crate::mesh_stack_deform::entries());
    table.extend(crate::curve_ops::entries());
    table.extend(crate::generator_ops::entries());
    table.extend(crate::skin::entries());
    table
}
