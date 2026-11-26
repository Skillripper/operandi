//! Set Polygon Type as a Mesh Stack op (`poly.setType.meshop.item`,
//! TYPE-IDS.md): makes the chosen polygons plain faces, subdivision
//! polygons or Catmull-Clark polygons. Unlike Toggle Subdivision it sets
//! the type rather than flipping it, so it gives the same result however
//! often the stack runs. Lines are left as they are (provisional, MOQ-34).
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, PolygonRepresentation, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetPolygonTypeOp {
    /// 0 Face, 1 Subdivision, 2 Catmull-Clark (stored by position, MOQ-25).
    #[serde(rename = "type")]
    pub kind: u32,
}
impl TypedOp for SetPolygonTypeOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let want = match self.kind {
            0 => PolygonRepresentation::Face,
            1 => PolygonRepresentation::Subdivs,
            2 => PolygonRepresentation::CatmullClark,
            _ => {
                return Err(
                    "Polygon Type must be 0 (Face), 1 (Subdivision) or 2 (Catmull-Clark)".into(),
                );
            }
        };
        let input = ctx.input;
        let mut out = input.clone();
        let reps = &mut out.attributes.representations;
        reps.resize(input.faces.len(), PolygonRepresentation::Face);
        for f in ctx.chosen_or_all()? {
            if !input.is_polyline(f as usize) {
                reps[f as usize] = want;
            }
        }
        // An all-Face mesh keeps the compact empty list (as Toggle does).
        if reps.iter().all(|r| *r == PolygonRepresentation::Face) {
            reps.clear();
        }
        out.validate()?;
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<SetPolygonTypeOp>::new(
        "poly.setType.meshop.item",
        "Set Polygon Type",
        "Polygon",
        true,
        vec![Field::new(
            "type",
            "Polygon Type",
            FieldKind::Choice(vec![
                "Face".into(),
                "Subdivision".into(),
                "Catmull-Clark".into(),
            ]),
        )],
    ))]
}
