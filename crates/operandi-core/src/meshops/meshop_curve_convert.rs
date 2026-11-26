//! Curve Convert as a Mesh Stack op (`curveconvert.meshop.item`,
//! TYPE-IDS.md): the chosen curves of type From become curves of type To
//! (Curve, Bezier or B-Spline), through the Convert Curves buttons'
//! conversions (CC-52/53). Its defaults (Curve to Bezier) are the "Curve to
//! Bezier" button, so that button needs no op of its own.
//!
//! Provisional (MOQ-41): the op converts in place (the buttons make a new
//! item, which an op can't); the new curves take the old ones' place at
//! the end of the element list. Curves of another type, lines and
//! polygons pass through. From and To are stored by position.
use crate::curve_convert::{CurveType, convert_points};
use crate::mesh_stack_builtin::choice;
use crate::meshop_curve_common::{chosen_curves, drop_faces, kind_of, points_of, push_curve};
use crate::meshop_types::{OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const TYPES: [CurveType; 3] = [CurveType::Curve, CurveType::Bezier, CurveType::BSpline];
const NAMES: [&str; 3] = ["Curve", "Bezier", "B-Spline"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurveConvertOp {
    /// 0 Curve, 1 Bezier, 2 B-Spline.
    pub from: u32,
    pub to: u32,
}
impl Default for CurveConvertOp {
    fn default() -> Self {
        Self { from: 0, to: 1 }
    }
}
fn curve_type(i: u32, what: &str) -> Result<CurveType> {
    TYPES
        .get(i as usize)
        .copied()
        .ok_or_else(|| format!("{what} must be 0 (Curve), 1 (Bezier) or 2 (B-Spline)"))
}
impl TypedOp for CurveConvertOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let from = curve_type(self.from, "From")?;
        let to = curve_type(self.to, "To")?;
        if from == to {
            return Ok(ctx.input.clone());
        }
        let mesh = ctx.input;
        let mut out = mesh.clone();
        let mut gone = std::collections::BTreeSet::new();
        for f in chosen_curves(ctx)? {
            let (kind, closed) = kind_of(mesh, f);
            if CurveType::of(kind) != Some(from) {
                continue;
            }
            let points = convert_points(&points_of(mesh, f), kind, closed, to)?;
            push_curve(&mut out, &points, to.kind(), closed, Some(f));
            gone.insert(f);
        }
        let out = drop_faces(&out, &gone)?;
        out.validate()?;
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<CurveConvertOp>::new(
        "curveconvert.meshop.item",
        "Curve Convert",
        "Curves",
        true,
        vec![choice("from", "From", &NAMES), choice("to", "To", &NAMES)],
    ))]
}
