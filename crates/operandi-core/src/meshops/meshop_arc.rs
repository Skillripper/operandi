//! Arc as a Mesh Stack op (`prim.arc.item`, TYPE-IDS.md): the Arc
//! primitive's pie-slice polygon (`arc_geometry`, NPR-ARC-01..04) added
//! to the op's input. Center and Radius place it; it starts toward Start
//! and turns Angle (up to 180°) toward End, in the plane through the
//! three; Reverse sweeps the rest of the disc instead.
//!
//! Provisional (MOQ-50): the channel list is the reference's help table
//! (it wasn't confirmed by creating the op) and its defaults are the Arc
//! tool's; Start and End only set the direction and the plane, Radius
//! the size.
use crate::mesh_stack_builtin::{check, count, distance};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{ArcParameters, Mesh, Result};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArcOp {
    /// Radians.
    pub angle: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub start_x: f64,
    pub start_y: f64,
    pub start_z: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub end_z: f64,
    pub radius: f64,
    pub reverse: bool,
    pub segments: u32,
}
impl Default for ArcOp {
    fn default() -> Self {
        let a = ArcParameters::default();
        let end = a.end();
        Self {
            angle: a.angle.to_radians(),
            center_x: a.center.x,
            center_y: a.center.y,
            center_z: a.center.z,
            start_x: a.start().x,
            start_y: a.start().y,
            start_z: a.start().z,
            end_x: end.x,
            end_y: end.y,
            end_z: end.z,
            radius: a.radius,
            reverse: a.reverse,
            segments: a.segments,
        }
    }
}
impl TypedOp for ArcOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let center = DVec3::new(self.center_x, self.center_y, self.center_z);
        let start = DVec3::new(self.start_x, self.start_y, self.start_z) - center;
        let end = DVec3::new(self.end_x, self.end_y, self.end_z) - center;
        let direction = start
            .try_normalize()
            .ok_or("Start must be away from Center")?;
        let normal = start
            .cross(end)
            .try_normalize()
            .ok_or("Start and End must not be in line with Center")?;
        let mut p = ArcParameters {
            center,
            start_direction: direction,
            normal,
            radius: self.radius,
            angle: 0.,
            segments: self.segments,
            reverse: self.reverse,
        };
        p.set_angle(self.angle.to_degrees());
        let arc = crate::arc_geometry(&p)?;
        let mut out = ctx.input.clone();
        let first = out.faces.len() as u32;
        crate::append_mesh(&mut out, arc)?;
        let n = out.faces.len() as u32;
        ctx.tag(&mut out, "arc", first..n);
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(
        Typed::<ArcOp>::new(
            "prim.arc.item",
            "Arc",
            "Create/Primitives",
            false,
            vec![
                distance("centerX", "Center X"),
                distance("centerY", "Center Y"),
                distance("centerZ", "Center Z"),
                distance("startX", "Start X"),
                distance("startY", "Start Y"),
                distance("startZ", "Start Z"),
                distance("endX", "End X"),
                distance("endY", "End Y"),
                distance("endZ", "End Z"),
                distance("radius", "Radius"),
                Field::new("angle", "Angle", FieldKind::Angle),
                count("segments", "Segments", 1, 10_000),
                check("reverse", "Reverse"),
            ],
        )
        .with_groups(&[("arc", "Arc")]),
    )]
}
