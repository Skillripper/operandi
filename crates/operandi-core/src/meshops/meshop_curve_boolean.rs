//! Curve Boolean as a Mesh Stack op (`curveBoolean.meshop.item`,
//! TYPE-IDS.md): the chosen closed curves cut against the others, as the
//! Curve Boolean command does (CB-10..19): Slice, Union, Intersect or
//! Subtract, in the plane Axis names (Auto: the curves' best fit).
//!
//! Provisional (MOQ-44): the op's Selection plays the command's selected
//! curves, so the driver is always the unselected curves (Driver Curve is
//! kept, not shown); with no Selection every curve is selected and they
//! cut each other (CB-18). Close and Trim Open Curves, Offset and Self
//! Intersection aren't built and are kept, not shown. Merge Curves starts
//! off, as the op's reference default says (the button starts it on).
use crate::curve_boolean::{CurveBooleanOp, CurveBooleanOptions, curve_boolean};
use crate::mesh_stack_builtin::{check, choice};
use crate::mesh_stack_tools::hidden;
use crate::meshop_curve_common::chosen_curves;
use crate::meshop_types::{OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CurveBooleanMeshOp {
    /// 0 Slice, 1 Union, 2 Intersect, 3 Subtract.
    pub mode: u32,
    /// 0 Auto, 1 X, 2 Y, 3 Z.
    pub axis: u32,
    /// 0 Unselected (the only one built).
    pub driver: u32,
    pub clip: bool,
    pub merge: bool,
    pub close: bool,
    pub trim: bool,
    pub offset: f64,
    pub self_int: bool,
}
impl TypedOp for CurveBooleanMeshOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if self.mode > 3 {
            return Err("Operation must be 0 (Slice) to 3 (Subtract)".into());
        }
        if self.axis > 3 {
            return Err("Axis must be 0 (Auto), 1 (X), 2 (Y) or 3 (Z)".into());
        }
        let curves: Vec<u32> = chosen_curves(ctx)?.into_iter().map(|f| f as u32).collect();
        if curves.is_empty() {
            return Ok(ctx.input.clone());
        }
        let o = CurveBooleanOptions {
            op: CurveBooleanOp::from_index(self.mode as usize),
            axis: self.axis as u8,
            clip: self.clip,
            merge: self.merge,
        };
        curve_boolean(ctx.input, &curves, &o)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<CurveBooleanMeshOp>::new(
        "curveBoolean.meshop.item",
        "Curve Boolean",
        "Curves",
        true,
        vec![
            choice("mode", "Operation", &CurveBooleanOp::NAMES),
            choice("axis", "Axis", &["Auto", "X", "Y", "Z"]),
            hidden("driver"),
            check("clip", "Clip"),
            check("merge", "Merge Curves"),
            hidden("close"),
            hidden("trim"),
            hidden("offset"),
            hidden("selfInt"),
        ],
    ))]
}
