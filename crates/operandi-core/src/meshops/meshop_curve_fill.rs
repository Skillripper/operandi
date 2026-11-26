//! Curve Fill as a Mesh Stack op (`curvefill`, TYPE-IDS.md): a flat
//! polygon for each chosen closed curve, as Make Curve Fill makes them
//! (CC-10), of the chosen Polygon Type. Delete Source Curve (on by
//! default) removes the curves; Flip Source Curves turns the fills over.
//!
//! Provisional (MOQ-43): only Make Curve Fill's single-polygon fill is
//! built, so Fill Steps, Edge Weight, Make UVs and the curve index range
//! are kept but not shown, and nested curves don't make holes (CC-11).
//! Open curves and polygons among the chosen elements pass through.
use crate::mesh_stack_builtin::{check, choice};
use crate::mesh_stack_tools::hidden;
use crate::meshop_curve_common::{chosen_curves, drop_faces, kind_of};
use crate::meshop_types::{OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, PolygonRepresentation, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurveFillOp {
    /// 0 Face, 1 Subdivision, 2 Catmull-Clark.
    pub polygon_type: u32,
    pub step_res: u32,
    pub curve_selection: String,
    pub enable_uv: bool,
    pub uvmap_name: String,
    pub weight: f64,
    pub delete: bool,
    pub flip: bool,
}
impl Default for CurveFillOp {
    fn default() -> Self {
        Self {
            polygon_type: 0,
            step_res: 10,
            curve_selection: String::new(),
            enable_uv: false,
            uvmap_name: "Texture".into(),
            weight: 0.,
            delete: true,
            flip: false,
        }
    }
}
impl TypedOp for CurveFillOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let kind = match self.polygon_type {
            0 => PolygonRepresentation::Face,
            1 => PolygonRepresentation::Subdivs,
            2 => PolygonRepresentation::CatmullClark,
            _ => {
                return Err(
                    "Polygon Type must be 0 (Face), 1 (Subdivision) or 2 (Catmull-Clark)".into(),
                );
            }
        };
        let closed: Vec<u32> = chosen_curves(ctx)?
            .into_iter()
            .filter(|&f| kind_of(ctx.input, f).1)
            .map(|f| f as u32)
            .collect();
        if closed.is_empty() {
            return Ok(ctx.input.clone());
        }
        let (mut out, fills) = crate::curve_commands::make_curve_fill(ctx.input, &closed)?;
        for &f in &fills {
            crate::curve_commands::set_representation(&mut out, f as usize, kind);
            if self.flip {
                out.faces[f as usize].vertices.reverse();
            }
        }
        let n = out.faces.len() as u32;
        ctx.tag(&mut out, "fill", n - fills.len() as u32..n);
        if self.delete {
            let gone = closed.iter().map(|&f| f as usize).collect();
            out = drop_faces(&out, &gone)?;
            // The fills were the last elements; they still are.
            let n = out.faces.len() as u32;
            ctx.tag(&mut out, "fill", n - fills.len() as u32..n);
        }
        out.validate()?;
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(
        Typed::<CurveFillOp>::new(
            "curvefill",
            "Curve Fill",
            "Curves",
            true,
            vec![
                choice(
                    "polygon_type",
                    "Polygon Type",
                    &["Face", "Subdivision", "Catmull-Clark"],
                ),
                hidden("step_res"),
                hidden("curve_selection"),
                hidden("enable_uv"),
                hidden("uvmap_name"),
                hidden("weight"),
                check("delete", "Delete Source Curve"),
                check("flip", "Flip Source Curves"),
            ],
        )
        .with_groups(FILL_GROUP),
    )]
}
const FILL_GROUP: &[(&str, &str)] = &[("fill", "Fill")];
