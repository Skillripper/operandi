//! Tidy the faces a strip cuts (MeshBlend B2/B4 "good and never broken").
//!
//! The cut leaves each touched source polygon as a few pieces whose edges
//! run from the strip's edge to the polygon's old corners. On a big flat
//! polygon (a one-segment cube's side, a cylinder's cap) no split into
//! triangles of only those points avoids long slivers. So each cut polygon
//! that is flat is re-meshed: its outline (every edge it shares with a
//! neighbour, kept exactly so the weld holds) becomes a constrained
//! Delaunay triangulation, refined with new points inside until no corner
//! is sharper than about 25° (spade, MIT OR Apache-2.0). Polygons that
//! aren't flat (smooth surfaces) are left as they are: new points would
//! have to sit on the curved surface, which is later work.
use super::evaluate::Origin;
use crate::Mesh;
use glam::{DVec2, DVec3};
use spade::{
    AngleLimit, ConstrainedDelaunayTriangulation, Point2, RefinementParameters, Triangulation,
};
use std::collections::{BTreeMap, HashMap};

/// Re-mesh the cut flat polygons of `mesh` in place. `origin` says which
/// source polygon each polygon came from and whether it may be re-meshed
/// (surface pieces only); it is updated to match. `width` sets the
/// smallest new triangle (no smaller than a quarter of the strip width).
pub(crate) fn tidy(mesh: &mut Mesh, origin: &mut Vec<(Origin, bool)>, width: f64) {
    let mut groups: BTreeMap<(u64, u32), Vec<usize>> = BTreeMap::new();
    for (f, (o, surface)) in origin.iter().enumerate() {
        if *surface && o.polygon != u32::MAX {
            groups.entry((o.item, o.polygon)).or_default().push(f);
        }
    }
    let mut replace: Vec<(Vec<usize>, Vec<Vec<u32>>)> = Vec::new();
    for faces in groups.into_values() {
        // Only polygons the cut split into pieces with a thin triangle.
        if faces.len() < 2 || !faces.iter().any(|&f| thin(mesh, f)) {
            continue;
        }
        if let Some(tris) = remesh(mesh, &faces, width) {
            replace.push((faces, tris));
        }
    }
    if replace.is_empty() {
        return;
    }
    let mut gone = vec![false; mesh.faces.len()];
    let mut added: Vec<(usize, Vec<u32>)> = Vec::new();
    for (faces, tris) in replace {
        for &f in &faces {
            gone[f] = true;
        }
        added.extend(tris.into_iter().map(|t| (faces[0], t)));
    }
    let mut new_faces = Vec::with_capacity(mesh.faces.len());
    let mut new_origin = Vec::with_capacity(origin.len());
    for (f, face) in mesh.faces.iter().enumerate() {
        if !gone[f] {
            new_faces.push(face.clone());
            new_origin.push(origin[f]);
        }
    }
    for (template, vertices) in added {
        let mut face = mesh.faces[template].clone();
        face.vertices = vertices;
        new_faces.push(face);
        new_origin.push(origin[template]);
    }
    mesh.faces = new_faces.into();
    *origin = new_origin;
}

/// A triangle with a corner under 12°.
fn thin(mesh: &Mesh, f: usize) -> bool {
    let v = &mesh.faces[f].vertices;
    if v.len() != 3 {
        return false;
    }
    let p = v
        .iter()
        .map(|&i| mesh.vertices[i as usize])
        .collect::<Vec<_>>();
    let ang = |a: DVec3, b: DVec3, c: DVec3| (b - a).angle_between(c - a);
    ang(p[0], p[1], p[2])
        .min(ang(p[1], p[2], p[0]))
        .min(ang(p[2], p[0], p[1]))
        < 12f64.to_radians()
}

/// New triangles for the flat region made of `faces`, or `None` when it
/// isn't flat or the mesher can't do it cleanly.
fn remesh(mesh: &mut Mesh, faces: &[usize], width: f64) -> Option<Vec<Vec<u32>>> {
    // The region's outline: directed edges with no partner inside it.
    let mut inside: HashMap<(u32, u32), ()> = HashMap::new();
    let mut normal = DVec3::ZERO;
    for &f in faces {
        let v = &mesh.faces[f].vertices;
        for k in 0..v.len() {
            inside.insert((v[k], v[(k + 1) % v.len()]), ());
        }
        for k in 1..v.len().saturating_sub(1) {
            let (a, b, c) = (
                mesh.vertices[v[0] as usize],
                mesh.vertices[v[k] as usize],
                mesh.vertices[v[k + 1] as usize],
            );
            normal += (b - a).cross(c - a);
        }
    }
    let normal = normal.try_normalize()?;
    let mut outline: Vec<(u32, u32)> = inside
        .keys()
        .copied()
        .filter(|&(a, b)| !inside.contains_key(&(b, a)))
        .collect();
    outline.sort_unstable();
    // Flat: every point within a millionth of the region's size of one plane.
    let mut points: Vec<u32> = outline.iter().flat_map(|&(a, b)| [a, b]).collect();
    points.sort_unstable();
    points.dedup();
    let origin = mesh.vertices[points[0] as usize];
    let (lo, hi) = crate::boolean::bbox(
        &points
            .iter()
            .map(|&i| mesh.vertices[i as usize])
            .collect::<Vec<_>>(),
    );
    let size = (hi - lo).length();
    if points
        .iter()
        .any(|&i| (mesh.vertices[i as usize] - origin).dot(normal).abs() > 1e-6 * size)
    {
        return None;
    }
    let u = normal.any_orthonormal_vector();
    let v = normal.cross(u);
    let flat = |p: DVec3| DVec2::new((p - origin).dot(u), (p - origin).dot(v));
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::new();
    let mut handle_of: HashMap<u32, spade::handles::FixedVertexHandle> = HashMap::new();
    let mut mesh_of: HashMap<usize, u32> = HashMap::new();
    for &i in &points {
        let q = flat(mesh.vertices[i as usize]);
        let h = cdt.insert(Point2::new(q.x, q.y)).ok()?;
        if mesh_of.insert(h.index(), i).is_some() {
            // Two outline points at one place: leave this region alone.
            return None;
        }
        handle_of.insert(i, h);
    }
    for &(a, b) in &outline {
        if !cdt.can_add_constraint(handle_of[&a], handle_of[&b]) {
            return None;
        }
        cdt.add_constraint(handle_of[&a], handle_of[&b]);
    }
    // Seed points inside: a grid at 2.5 × the outline's typical edge
    // length, kept away from the outline. Refinement alone can't fix the
    // slivers along the outline (it may not split those edges), but with
    // points to connect to, the triangulation is even.
    let mut lengths: Vec<f64> = outline
        .iter()
        .map(|&(a, b)| mesh.vertices[a as usize].distance(mesh.vertices[b as usize]))
        .collect();
    lengths.sort_by(f64::total_cmp);
    let segs: Vec<(DVec2, DVec2)> = outline
        .iter()
        .map(|&(a, b)| {
            (
                flat(mesh.vertices[a as usize]),
                flat(mesh.vertices[b as usize]),
            )
        })
        .collect();
    let (lo2, hi2) = segs.iter().fold(
        (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)),
        |(lo, hi), (a, b)| (lo.min(a.min(*b)), hi.max(a.max(*b))),
    );
    let mut step = 2.5 * lengths[lengths.len() / 2].max(1e-9);
    while ((hi2 - lo2) / step).element_product() > 4000. {
        step *= 1.5;
    }
    let mut y = lo2.y + step * 0.5;
    while y < hi2.y {
        let mut x = lo2.x + step * 0.5;
        while x < hi2.x {
            let q = DVec2::new(x, y);
            let near = segs.iter().any(|&(a, b)| {
                let ab = b - a;
                let t = ((q - a).dot(ab) / ab.length_squared().max(1e-300)).clamp(0., 1.);
                // A long outline edge can't be split, so points keep far
                // enough off it for the triangles on it to stay fat.
                (a + ab * t).distance(q) < (0.6 * step).max(0.15 * ab.length())
            });
            if !near {
                cdt.insert(Point2::new(q.x, q.y)).ok()?;
            }
            x += step;
        }
        y += step;
    }
    let min_area = (0.25 * width).powi(2).max(1e-12 * size * size);
    let result = cdt.refine(
        RefinementParameters::<f64>::new()
            .exclude_outer_faces(true)
            .keep_constraint_edges()
            .with_angle_limit(AngleLimit::from_deg(25.))
            .with_min_required_area(min_area)
            .with_max_additional_vertices(4 * points.len() + 200),
    );
    // Splitting an outline edge would break the weld with the neighbour.
    if cdt.num_constraints() != outline.len() {
        return None;
    }
    let excluded: std::collections::HashSet<_> = result.excluded_faces.into_iter().collect();
    let mut tris = Vec::new();
    for face in cdt.inner_faces() {
        if excluded.contains(&face.fix()) {
            continue;
        }
        let mut t = [0u32; 3];
        for (k, vh) in face.vertices().iter().enumerate() {
            let index = vh.fix().index();
            t[k] = match mesh_of.get(&index) {
                Some(&i) => i,
                None => {
                    let q = vh.position();
                    let p = origin + u * q.x + v * q.y;
                    mesh.vertices.push(p);
                    let i = (mesh.vertices.len() - 1) as u32;
                    mesh_of.insert(index, i);
                    i
                }
            };
        }
        // spade's faces run counterclockwise in (u, v); with u × v = normal
        // that is the region's own winding.
        tris.push(t.to_vec());
    }
    Some(tris)
}
