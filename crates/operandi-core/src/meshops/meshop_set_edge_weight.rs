//! Set Edge Weight as a Mesh Stack op (`setvmap.subd.item`, TYPE-IDS.md):
//! gives the op's edges a subdivision weight, through the Edge Weight
//! tool's engine (`edge_crease_geometry`). Geometry is unchanged; 0
//! removes the edges' weights. With no Selection every edge is weighted.
use crate::mesh_stack_tools::chosen_edges;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetEdgeWeightOp {
    /// 1.0 = 100%.
    #[serde(rename = "MapValue")]
    pub map_value: f64,
}
impl TypedOp for SetEdgeWeightOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let edges = chosen_edges(ctx);
        if edges.is_empty() {
            return Ok(ctx.input.clone());
        }
        crate::edge_crease_geometry(ctx.input, &edges, self.map_value, false)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<SetEdgeWeightOp>::new(
            "setvmap.subd.item",
            "Set Edge Weight",
            "Vertex Map",
            true,
            vec![Field::new("MapValue", "Map Value", FieldKind::Percent)],
        ))],
        SelElement::Edge,
    )
}
