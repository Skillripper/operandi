//! Hard Edge and Hard Edge Convert as Mesh Stack ops
//! (`hardedge.meshop.item`, `hardedgeconvert.meshop.item`, TYPE-IDS.md),
//! through the vertex normal tools' engines (`harden`, `soften`,
//! `convert_to_hard_edges`).
//!
//! Provisional (MOQ-37): Hard Edge's Selection picks edges (the reference's
//! Sel is VRTX). Mode is 0 Selection (the chosen edges, every edge with no
//! Selection) or 1 By Angle (the chosen edges sharper than Angle); the
//! reference's "auto" default reads as Selection. Hardness is 0 Hard,
//! 1 Soft. UV Map is saved and kept, not used. Hard Edge Convert reads the
//! default smoothing settings, since an op can't see the scene's.
use crate::mesh_stack_builtin::{check, choice};
use crate::mesh_stack_tools::{chosen_edges, hidden};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::topology::edge_key;
use crate::vertex_normals::{NormalOptions, convert_to_hard_edges, harden, soften};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardEdgeOp {
    /// Hardness: 0 Hard, 1 Soft.
    #[serde(rename = "type")]
    pub kind: u32,
    /// 0 Selection, 1 By Angle.
    pub mode: u32,
    /// Clear Other Hard Edges (with Hard).
    pub clear: bool,
    /// Radians.
    pub angle: f64,
    pub vmap: String,
}
impl TypedOp for HardEdgeOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        crate::mesh_stack_builtin::finite(&[self.angle], "Angle")?;
        let chosen: BTreeSet<(u32, u32)> = chosen_edges(ctx)
            .into_iter()
            .map(|(a, b)| edge_key(a, b))
            .collect();
        let edges: BTreeSet<(u32, u32)> = match self.mode {
            0 => chosen,
            1 => {
                let limit = self.angle;
                let faces = ctx.input.edge_faces();
                chosen
                    .into_iter()
                    .filter(|e| {
                        faces.get(e).is_some_and(|fs| {
                            fs.len() >= 2 && {
                                let (a, b) = (ctx.input.normal(fs[0]), ctx.input.normal(fs[1]));
                                a.length_squared() > 0.
                                    && b.length_squared() > 0.
                                    && a.angle_between(b) > limit + 1e-9
                            }
                        })
                    })
                    .collect()
            }
            _ => return Err("Hard Edge Mode must be 0 (Selection) or 1 (By Angle)".into()),
        };
        let o = NormalOptions::default();
        match self.kind {
            0 => harden(ctx.input, &edges, self.clear, &o),
            1 => soften(ctx.input, Some(&edges), &o),
            _ => Err("Hardness must be 0 (Hard) or 1 (Soft)".into()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct HardEdgeConvertOp {
    /// Remove Normal Map: drop the vertex normal maps afterwards.
    pub remove_normal: bool,
    /// Remove Smoothing Groups.
    #[serde(rename = "removeSMGP")]
    pub remove_smgp: bool,
}
impl TypedOp for HardEdgeConvertOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let smoothing = crate::shading::SmoothingSettings::default();
        let mut out = convert_to_hard_edges(ctx.input, &smoothing, self.remove_smgp)?;
        if self.remove_normal {
            out.attributes.corner_normals.clear();
            out.attributes.normal_map_order.clear();
        }
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    crate::meshop_types::with_element(
        vec![
            Arc::new(Typed::<HardEdgeOp>::new(
                "hardedge.meshop.item",
                "Hard Edge",
                "Edit",
                true,
                vec![
                    choice("type", "Hardness", &["Hard", "Soft"]),
                    choice("mode", "Mode", &["Selection", "By Angle"]),
                    check("clear", "Clear Other Hard Edges"),
                    Field::new("angle", "Angle", FieldKind::Angle),
                    hidden("vmap"),
                ],
            )),
            Arc::new(Typed::<HardEdgeConvertOp>::new(
                "hardedgeconvert.meshop.item",
                "Hard Edge Convert",
                "Edit",
                false,
                vec![
                    check("removeNormal", "Remove Normal Map"),
                    check("removeSMGP", "Remove Smoothing Groups"),
                ],
            )),
        ],
        SelElement::Edge,
    )
}
