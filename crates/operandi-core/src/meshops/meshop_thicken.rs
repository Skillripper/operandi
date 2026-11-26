//! Thicken as a Mesh Stack op (TYPE-IDS.md "Thicken"): Smooth Shift with
//! Thicken on, so a sheet gets a back side and walls, as its own Add
//! Operator entry with its own defaults.
//!
//! Provisional (MOQ-33): the reference saves it as a Smooth Shift
//! (`poly.smshift.item` named "Thicken"); Operandi gives it its own type
//! id, `poly.thicken.item`, because two entries can't share one. The values
//! and the engine are Smooth Shift's.
use crate::mesh_stack_builtin::SmoothShiftOp;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ThickenOp(pub SmoothShiftOp);
impl Default for ThickenOp {
    fn default() -> Self {
        Self(SmoothShiftOp {
            thicken: true,
            ..SmoothShiftOp::default()
        })
    }
}
impl TypedOp for ThickenOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        self.0.run(ctx)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let check = |k, l| Field::new(k, l, FieldKind::Check);
    vec![Arc::new(
        Typed::<ThickenOp>::new(
            "poly.thicken.item",
            "Thicken",
            "Polygon",
            true,
            vec![
                Field::new("shift", "Offset", FieldKind::Distance),
                Field::new("scale", "Scale", FieldKind::Percent),
                Field::new("maxAngle", "Max Smoothing Angle", FieldKind::Angle),
                check("thicken", "Thicken"),
                check("sharp", "Sharp"),
                check("angleWeight", "Angle Weight"),
                check("bothSides", "Both Sides"),
            ],
        )
        .with_groups(&[("front", "Front"), ("sides", "Sides")]),
    )]
}
