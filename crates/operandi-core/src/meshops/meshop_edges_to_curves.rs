//! Edges to Curves as a Mesh Stack op (`pmodel.edgeToCurve.item`,
//! TYPE-IDS.md): the chosen edges become curves, joined end to end into
//! chains as Edges To Curves joins them (Continuous), or one curve per
//! edge (Separate). Curve Type makes each a line, a Curve through the
//! points, or the Bezier that draws that Curve. Delete Geometry removes
//! the polygons the edges belonged to.
//!
//! Provisional (MOQ-45): the Curve Type and Curve Mode lists past their
//! defaults, and Delete Geometry's reach, are readings of the channel
//! names; Bezier Tangent Weight is kept, not shown (the Bezier always
//! draws the same shape as the Curve). With no Selection every border edge
//! (an edge of one polygon) is used, as Bridge does.
use crate::curves::CurveKind;
use crate::mesh_stack_builtin::{check, choice};
use crate::mesh_stack_tools::{chosen_edges, hidden};
use crate::meshop_curve_common::{drop_faces, points_of, push_curve};
use crate::meshop_types::{OpContext, OpDef, Typed, TypedOp};
use crate::{EdgeCommand, Mesh, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EdgesToCurvesOp {
    /// 0 Polyline, 1 Curve, 2 Bezier.
    pub crv_type: u32,
    /// 0 Continuous, 1 Separate.
    pub mode: u32,
    pub bezier_weight: f64,
    pub remove: bool,
}
impl Default for EdgesToCurvesOp {
    fn default() -> Self {
        Self {
            crv_type: 0,
            mode: 0,
            bezier_weight: 0.5,
            remove: false,
        }
    }
}
impl TypedOp for EdgesToCurvesOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if self.crv_type > 2 {
            return Err("Curve Type must be 0 (Polyline), 1 (Curve) or 2 (Bezier)".into());
        }
        if self.mode > 1 {
            return Err("Curve Mode must be 0 (Continuous) or 1 (Separate)".into());
        }
        let mesh = ctx.input;
        let mut edges = chosen_edges(ctx);
        if ctx.selection.is_none() {
            let counts = mesh.edge_counts();
            edges.retain(|e| counts.get(e).copied().unwrap_or(0) == 1);
        }
        if edges.is_empty() {
            return Ok(mesh.clone());
        }
        let first = mesh.faces.len();
        // Every new element is a line through its chain's points.
        let mut out = if self.mode == 0 {
            crate::edge_commands::edge_command_geometry(mesh, &edges, EdgeCommand::ToCurves)?.0
        } else {
            let mut out = mesh.clone();
            for &(a, b) in &edges {
                let pts = [mesh.vertices[a as usize], mesh.vertices[b as usize]];
                push_curve(&mut out, &pts, CurveKind::Line, false, None);
            }
            out
        };
        if self.crv_type > 0 {
            out = retype(&out, first, self.crv_type == 2)?;
        }
        if self.remove {
            let on: BTreeSet<(u32, u32)> = edges.iter().copied().collect();
            let gone: BTreeSet<usize> = (0..first)
                .filter(|&f| {
                    !mesh.is_polyline(f)
                        && mesh
                            .element_spans(f)
                            .any(|(a, b)| on.contains(&(a.min(b), a.max(b))))
                })
                .collect();
            out = drop_faces(&out, &gone)?;
        }
        out.validate()?;
        Ok(out)
    }
}

/// The lines from element `first` on, as Curves through their points or
/// as the Beziers that draw those Curves.
fn retype(mesh: &Mesh, first: usize, bezier: bool) -> Result<Mesh> {
    let curve = CurveKind::Curve {
        start_control: false,
        end_control: false,
    };
    let mut out = mesh.clone();
    let mut gone = BTreeSet::new();
    for f in first..mesh.faces.len() {
        let (_, closed) = crate::meshop_curve_common::kind_of(mesh, f);
        let pts = points_of(mesh, f);
        let made = if bezier {
            crate::curves::smooth_bezier(&pts, closed).map(|b| (b, CurveKind::Bezier))
        } else if pts.len() >= curve.minimum_points() {
            Some((pts, curve))
        } else {
            None
        };
        // Two-point chains stay lines: no curve passes through only two.
        if let Some((pts, kind)) = made {
            push_curve(&mut out, &pts, kind, closed, None);
            gone.insert(f);
        }
    }
    drop_faces(&out, &gone)
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<EdgesToCurvesOp>::new(
        "pmodel.edgeToCurve.item",
        "Edges to Curves",
        "Curves",
        true,
        vec![
            choice("crvType", "Curve Type", &["Polyline", "Curve", "Bezier"]),
            choice("mode", "Curve Mode", &["Continuous", "Separate"]),
            hidden("bezierWeight"),
            check("remove", "Delete Geometry"),
        ],
    ))]
}
