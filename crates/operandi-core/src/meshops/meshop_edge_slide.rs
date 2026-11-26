//! Edge Slide as a Mesh Stack op (`edge.slide.item`, TYPE-IDS.md): slides
//! the op's edges along their rails, through the direct tool's engine
//! (`edge_slide_extra`).
//!
//! Provisional (MOQ-36): the reference's Selection input picks points;
//! Operandi's Edge Slide works on edges, so the op picks edges. With no
//! Selection every edge is chosen, which slides nothing on a closed mesh
//! that has no rails, so pick edges for it. Interpolation is stored by
//! position (0 Percent, 1 Distance; MOQ-25). The handle, inset, profile
//! and point-adding channels are saved and kept, not used.
use crate::mesh_stack_builtin::{check, choice, distance, finite};
use crate::mesh_stack_tools::{chosen_edges, hidden};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result, SlideExtras};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EdgeSlideOp {
    /// 0 Percent, 1 Distance.
    pub interpolation: u32,
    pub dist: f64,
    /// 1.0 = 100% of each rail.
    pub percent: f64,
    /// Stop Negative Slide.
    pub stop: bool,
    /// Loop Slide.
    #[serde(rename = "loop")]
    pub loop_: bool,
    /// Preserve Curvature.
    pub curvature: bool,
    pub tension: f64,
    /// Slip UVs.
    #[serde(rename = "lockUV")]
    pub lock_uv: bool,
    // Saved and kept, not used.
    pub mode: u32,
    pub vert_mode: u32,
    pub handle_id: i64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub dir_z: f64,
    pub merge: bool,
    pub duplicate: bool,
    pub snap: bool,
    pub segs: u32,
    pub inset: f64,
    pub reversex: bool,
    pub reversey: bool,
    pub aspect: bool,
    pub handle_line_width: f64,
    pub txuv_name: String,
    pub alpha1: f64,
    pub alpha2: f64,
    #[serde(rename = "alphaN")]
    pub alpha_n: f64,
    pub project: bool,
    pub shape: u32,
    pub p0: f64,
    pub p1: f64,
    pub depth: f64,
    pub add_point: u32,
    pub add_mode: u32,
    pub arc_approx: bool,
}
impl Default for EdgeSlideOp {
    fn default() -> Self {
        let x = SlideExtras::default();
        Self {
            interpolation: 0,
            dist: 0.,
            percent: 0.,
            stop: x.stop_negative,
            loop_: x.loop_slide,
            curvature: x.preserve_curvature,
            tension: x.tension,
            lock_uv: x.slip_uvs,
            mode: 0,
            vert_mode: 0,
            handle_id: -1,
            dir_x: 0.,
            dir_y: 0.,
            dir_z: 0.,
            merge: true,
            duplicate: false,
            snap: false,
            segs: 0,
            inset: 0.,
            reversex: true,
            reversey: false,
            aspect: true,
            handle_line_width: 2.5,
            txuv_name: String::new(),
            alpha1: 0.,
            alpha2: 0.,
            alpha_n: 0.,
            project: false,
            shape: 0,
            p0: 0.,
            p1: 0.,
            depth: 1.,
            add_point: 0,
            add_mode: 0,
            arc_approx: false,
        }
    }
}
impl TypedOp for EdgeSlideOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        finite(
            &[self.dist, self.percent, self.tension],
            "Edge Slide values",
        )?;
        let (amount, by_distance) = match self.interpolation {
            0 => (self.percent, false),
            1 => (self.dist, true),
            _ => return Err("Interpolation must be 0 (Percent) or 1 (Distance)".into()),
        };
        if amount == 0. {
            return Ok(ctx.input.clone());
        }
        let extras = SlideExtras {
            stop_negative: self.stop,
            loop_slide: self.loop_,
            preserve_curvature: self.curvature,
            tension: self.tension,
            slip_uvs: self.lock_uv,
        };
        let (mesh, _) = crate::edge_slide_extra(
            ctx.input,
            &chosen_edges(ctx),
            amount,
            by_distance,
            None,
            &extras,
        )?;
        Ok(mesh)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let mut fields = vec![
        choice("interpolation", "Interpolation", &["Percent", "Distance"]),
        Field::new("percent", "Percent", FieldKind::Percent),
        distance("dist", "Distance"),
        check("stop", "Stop Negative Slide"),
        check("loop", "Loop Slide"),
        check("curvature", "Preserve Curvature"),
        Field::new("tension", "Tension", FieldKind::Percent),
        check("lockUV", "Slip UVs"),
    ];
    fields.extend(
        [
            "mode",
            "vertMode",
            "handleId",
            "dirX",
            "dirY",
            "dirZ",
            "merge",
            "duplicate",
            "snap",
            "segs",
            "inset",
            "reversex",
            "reversey",
            "aspect",
            "handleLineWidth",
            "txuvName",
            "alpha1",
            "alpha2",
            "alphaN",
            "project",
            "shape",
            "p0",
            "p1",
            "depth",
            "addPoint",
            "addMode",
            "arcApprox",
        ]
        .map(hidden),
    );
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<EdgeSlideOp>::new(
            "edge.slide.item",
            "Edge Slide",
            "Edge",
            true,
            fields,
        ))],
        SelElement::Edge,
    )
}
