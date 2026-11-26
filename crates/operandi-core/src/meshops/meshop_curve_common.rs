//! Helpers the curve ops share (Mesh Stack batch 3): which chosen
//! elements are curves, adding a curve element, and dropping elements
//! with the points only they used.
use crate::curves::CurveKind;
use crate::meshop_types::OpContext;
use crate::{Mesh, PolygonRepresentation, Result};
use glam::DVec3;
use std::collections::BTreeSet;

/// The chosen elements that are curves or lines (every one with no
/// Selection), in order.
pub(crate) fn chosen_curves(ctx: &OpContext) -> Result<Vec<usize>> {
    Ok(ctx
        .chosen_or_all()?
        .into_iter()
        .map(|f| f as usize)
        .filter(|&f| f < ctx.input.faces.len() && ctx.input.is_polyline(f))
        .collect())
}

/// Element `f`'s point positions, in order.
pub(crate) fn points_of(mesh: &Mesh, f: usize) -> Vec<DVec3> {
    mesh.faces[f]
        .vertices
        .iter()
        .map(|&v| mesh.vertices[v as usize])
        .collect()
}

/// Element `f`'s curve kind and closure (a line for anything else).
pub(crate) fn kind_of(mesh: &Mesh, f: usize) -> (CurveKind, bool) {
    crate::curve_convert::curve_of(mesh, f).unwrap_or((CurveKind::Line, false))
}

/// Adds a curve element through new points `points`, taking its material
/// and part from `template`. Returns its index.
pub(crate) fn push_curve(
    mesh: &mut Mesh,
    points: &[DVec3],
    curve: CurveKind,
    closed: bool,
    template: Option<usize>,
) -> usize {
    let first = mesh.vertices.len() as u32;
    mesh.vertices.extend_from_slice(points);
    let ids = (first..mesh.vertices.len() as u32).collect();
    crate::vertex_commands::append_face(mesh, ids, template);
    let f = mesh.faces.len() - 1;
    crate::curve_commands::set_representation(
        mesh,
        f,
        PolygonRepresentation::Polyline { closed, curve },
    );
    f
}

/// `mesh` without the elements in `gone`, and without the points only
/// they used (points that were already loose stay).
pub(crate) fn drop_faces(mesh: &Mesh, gone: &BTreeSet<usize>) -> Result<Mesh> {
    if gone.is_empty() {
        return Ok(mesh.clone());
    }
    let loose: BTreeSet<u32> = mesh
        .used_points()
        .iter()
        .enumerate()
        .filter(|(_, used)| !**used)
        .map(|(v, _)| v as u32)
        .collect();
    let kept = crate::clipboard::whole_faces(mesh, |f| !gone.contains(&f));
    Ok(crate::clipboard::subset(mesh, &kept, &loose)?.0)
}
