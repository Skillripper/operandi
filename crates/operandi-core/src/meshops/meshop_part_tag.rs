//! Part Tag as a Mesh Stack op: names the chosen polygons' part, as Set
//! Part does, so later selection ops can pick by part.
//!
//! Provisional (MOQ-38): the type id `pmodel.partTag.item` and its one
//! channel `partName` follow Material Tag's; the reference's own weren't
//! read. Blank names are refused, as Set Part refuses them.
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PartTagOp {
    pub part_name: String,
}
impl Default for PartTagOp {
    fn default() -> Self {
        Self {
            part_name: "Default".into(),
        }
    }
}
impl TypedOp for PartTagOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        crate::polygon_part::validate_part_name(&self.part_name)?;
        let mut mesh = ctx.input.clone();
        for f in ctx.chosen_or_all()? {
            if !mesh.is_polyline(f as usize) {
                mesh.faces[f as usize].part = Some(self.part_name.clone());
            }
        }
        Ok(mesh)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<PartTagOp>::new(
        "pmodel.partTag.item",
        "Part Tag",
        "Polygon",
        true,
        vec![Field::new("partName", "Part Name", FieldKind::Text)],
    ))]
}
