//! Assign Selection Set as a Mesh Stack op (`assignselectionset.meshop.item`,
//! TYPE-IDS.md): stores the op's chosen polygons (every polygon with no
//! Selection) as a polygon selection set named Name on its result, so ops
//! above can pick it with Select by Selection Set. An existing set of that
//! name is replaced. With Name blank the op does nothing.
//!
//! Provisional (MOQ-40): the reference's Selection picks points; Operandi
//! stores polygons, the set kind Select by Selection Set falls back to.
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignSelectionSetOp {
    pub name: String,
}
impl TypedOp for AssignSelectionSetOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let name = self.name.trim();
        if name.is_empty() {
            return Ok(ctx.input.clone());
        }
        let mut mesh = ctx.input.clone();
        mesh.attributes
            .polygon_sets
            .insert(name.to_string(), ctx.chosen_or_all()?.into_iter().collect());
        Ok(mesh)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<AssignSelectionSetOp>::new(
        "assignselectionset.meshop.item",
        "Assign Selection Set",
        "Selection",
        true,
        vec![Field::new("name", "Name", FieldKind::Text)],
    ))]
}
