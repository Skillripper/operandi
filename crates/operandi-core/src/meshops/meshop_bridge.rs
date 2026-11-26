//! Bridge as a Mesh Stack op (`edge.bridge.item`, TYPE-IDS.md "Bridge"):
//! joins two edge runs or loops with new polygons, through the direct
//! tool's engine (`edge_bridge`).
//!
//! Provisional (MOQ-32): with no Selection the op bridges the mesh's
//! border edges, so a mesh with two open holes bridges them. Mode is
//! stored by position (0 Linear, 1 Curve, 2 Smooth; MOQ-25). The channels
//! Operandi's Bridge doesn't build yet are saved and kept, not shown.
use crate::mesh_stack_builtin::{check, choice, count, finite};
use crate::mesh_stack_tools::{NEW_POLYGON, chosen_edges, hidden, new_polygons};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BridgeOp {
    pub segments: u32,
    /// Whole turns of the pairing; a fraction is dropped.
    pub twist: f64,
    /// 0 Linear, 1 Curve, 2 Smooth.
    pub mode: u32,
    pub remove: bool,
    pub flip: bool,
    /// Reverse Direction.
    pub orient: bool,
    /// 1.0 = 100%.
    pub tension: f64,
    // Saved and kept, not built yet.
    pub uvs: u32,
    pub inset: f64,
    pub reversex: bool,
    pub reversey: bool,
    pub aspect: bool,
    pub connect: bool,
    pub auto_step: bool,
    pub steps: u32,
    pub continuous: bool,
    pub guide_curve: bool,
    pub close_loop: bool,
}
impl Default for BridgeOp {
    fn default() -> Self {
        let p = crate::BridgeParameters::default();
        Self {
            segments: p.segments,
            twist: p.twist as f64,
            mode: 1,
            remove: p.remove,
            flip: p.flip,
            orient: p.reverse,
            tension: p.tension,
            uvs: 0,
            inset: 0.,
            reversex: false,
            reversey: false,
            aspect: true,
            connect: true,
            auto_step: false,
            steps: 10,
            continuous: true,
            guide_curve: false,
            close_loop: false,
        }
    }
}

/// The edges with a polygon on one side only.
fn border_edges(mesh: &Mesh) -> Vec<(u32, u32)> {
    let counts = mesh.edge_counts();
    mesh.edges()
        .into_iter()
        .filter(|e| counts.get(e).copied() == Some(1))
        .collect()
}

impl TypedOp for BridgeOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        finite(&[self.tension, self.twist], "Tension and Twist")?;
        let mode = match self.mode {
            0 => crate::BridgeMode::Linear,
            1 => crate::BridgeMode::Curve,
            2 => crate::BridgeMode::Smooth,
            _ => return Err("Bridge Mode must be 0 (Linear), 1 (Curve) or 2 (Smooth)".into()),
        };
        let p = crate::BridgeParameters {
            segments: self.segments,
            twist: crate::BridgeParameters::twist_field(self.twist)?,
            mode,
            remove: self.remove,
            flip: self.flip,
            reverse: self.orient,
            tension: self.tension,
        };
        let edges = if ctx.selection.is_some() {
            chosen_edges(ctx)
        } else {
            border_edges(ctx.input)
        };
        let (mesh, _) = crate::edge_commands::edge_bridge(ctx.input, &edges, p)?;
        Ok(new_polygons(ctx, mesh))
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let mut fields = vec![
        count("segments", "Segments", 0, 256),
        Field::new("twist", "Twist", FieldKind::Number),
        choice("mode", "Mode", &["Linear", "Curve", "Smooth"]),
        check("remove", "Remove Polygons"),
        check("flip", "Flip Polygons"),
        check("orient", "Reverse Direction"),
        Field::new("tension", "Tension", FieldKind::Percent),
    ];
    fields.extend(
        [
            "uvs",
            "inset",
            "reversex",
            "reversey",
            "aspect",
            "connect",
            "autoStep",
            "steps",
            "continuous",
            "guideCurve",
            "closeLoop",
        ]
        .map(hidden),
    );
    crate::meshop_types::with_element(
        vec![Arc::new(
            Typed::<BridgeOp>::new("edge.bridge.item", "Bridge", "Edit", true, fields)
                .with_groups(NEW_POLYGON),
        )],
        SelElement::Edge,
    )
}
