//! Curve Winding Aligner as a Mesh Stack op (`curveWinding.meshop.item`,
//! TYPE-IDS.md): every chosen closed curve runs the same way round, as
//! Align Winding does (CC-20/21), with Merge Curves first joining open
//! curves that share end points.
//!
//! Provisional (MOQ-42): Axis is kept but only Automatic is built (the
//! other choices are CC-Q4), so it isn't shown. With no curves among the
//! chosen elements the op passes its input through.
use crate::curve_commands::{WindingOptions, align_winding};
use crate::mesh_stack_builtin::check;
use crate::mesh_stack_tools::hidden;
use crate::meshop_curve_common::chosen_curves;
use crate::meshop_types::{OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurveWindingOp {
    /// 0 Automatic (the only one built).
    pub axis: u32,
    pub merge: bool,
    pub reverse: bool,
}
impl Default for CurveWindingOp {
    fn default() -> Self {
        Self {
            axis: 0,
            merge: true,
            reverse: false,
        }
    }
}
impl TypedOp for CurveWindingOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let curves: Vec<u32> = chosen_curves(ctx)?.into_iter().map(|f| f as u32).collect();
        if curves.is_empty() {
            return Ok(ctx.input.clone());
        }
        let o = WindingOptions {
            merge: self.merge,
            reverse: self.reverse,
        };
        Ok(align_winding(ctx.input, &curves, o)?.0)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<CurveWindingOp>::new(
        "curveWinding.meshop.item",
        "Curve Winding Aligner",
        "Curves",
        true,
        vec![
            hidden("axis"),
            check("merge", "Merge Curves"),
            check("reverse", "Reverse"),
        ],
    ))]
}
