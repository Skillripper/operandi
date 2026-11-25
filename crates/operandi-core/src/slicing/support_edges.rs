//! Edge Slice's Add Support Edges (Jason 2026-11-25; Operandi's own
//! option, not in the reference): after a cut, every polygon holding one
//! of the cut's new points that has more than four sides is split into
//! quads (and one triangle when its side count is odd) by new edges
//! between its existing corners. No point is added and polygons the cut
//! didn't reach are left alone.
//!
//! Each polygon is planned on its own with a small dynamic programme over
//! its corner chain (like the classic minimum-weight triangulation, with
//! quads as well as triangles). A plan's cost is its pieces (a quad costs
//! less than a triangle, so quads win where the count allows), how far
//! their corners are from square, and a small charge per new edge that is
//! higher when neither end is one of the cut's new points. Pieces must be
//! convex and every new edge must run inside the polygon.
use crate::Mesh;
use crate::slice_cut::{Chord, Node};
use crate::topology::edge_key;
use glam::{DVec2, DVec3};
use std::collections::BTreeSet;

/// Polygons with more corners than this are left as they are (the plan
/// grows with the fourth power of the corner count).
const MAX_CORNERS: usize = 32;

/// The support chords for `mesh` after a cut whose new points are
/// `new_points`: one chord per new edge, between existing corners.
pub(crate) fn support_chords(mesh: &Mesh, new_points: &BTreeSet<u32>) -> Vec<Chord> {
    let point_faces = mesh.point_faces();
    let mut faces: BTreeSet<usize> = BTreeSet::new();
    for &v in new_points {
        if (v as usize) < mesh.vertices.len() {
            faces.extend(point_faces.of(v).iter().map(|&f| f as usize));
        }
    }
    let edges = mesh.edge_counts();
    let mut chords = Vec::new();
    for face in faces {
        let vs = &mesh.faces[face].vertices;
        let n = vs.len();
        if mesh.is_polyline(face) || n <= 4 || n > MAX_CORNERS {
            continue;
        }
        let distinct: BTreeSet<u32> = vs.iter().copied().collect();
        if distinct.len() != n {
            continue;
        }
        let corners = flatten(mesh, face);
        let is_new: Vec<bool> = vs.iter().map(|v| new_points.contains(v)).collect();
        let existing = |i: usize, j: usize| edges.contains_key(&edge_key(vs[i], vs[j]));
        let Some(diagonals) = plan(&corners, &is_new, &existing) else {
            continue;
        };
        for (i, j) in diagonals {
            chords.push(Chord {
                face,
                from: Node::Point(vs[i]),
                to: Node::Point(vs[j]),
                through: Vec::new(),
            });
        }
    }
    chords
}

/// The corners of polygon `face` in its own plane.
fn flatten(mesh: &Mesh, face: usize) -> Vec<DVec2> {
    let normal = mesh.normal(face);
    let normal = if normal.length_squared() > 0. {
        normal.normalize()
    } else {
        DVec3::Y
    };
    let u = normal.any_orthonormal_vector();
    let w = normal.cross(u);
    let origin = mesh.center(face);
    mesh.faces[face]
        .vertices
        .iter()
        .map(|&v| {
            let d = mesh.vertices[v as usize] - origin;
            DVec2::new(d.dot(u), d.dot(w))
        })
        .collect()
}

fn cross(o: DVec2, a: DVec2, b: DVec2) -> f64 {
    (a - o).perp_dot(b - o)
}

/// Whether segments `a`-`b` and `c`-`d` cross at a point inside both.
fn crosses(a: DVec2, b: DVec2, c: DVec2, d: DVec2) -> bool {
    let (d1, d2) = (cross(a, b, c), cross(a, b, d));
    let (d3, d4) = (cross(c, d, a), cross(c, d, b));
    d1 * d2 < 0. && d3 * d4 < 0.
}

fn inside(polygon: &[DVec2], p: DVec2) -> bool {
    let n = polygon.len();
    let mut odd = false;
    for i in 0..n {
        let (a, b) = (polygon[i], polygon[(i + 1) % n]);
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x) {
            odd = !odd;
        }
    }
    odd
}

/// A piece's cost, or `None` when it isn't convex. Corners on a straight
/// run (a cut point on an edge) are allowed but cost a lot: such a "quad"
/// looks like a triangle.
fn piece_cost(p: &[DVec2], scale: f64) -> Option<f64> {
    let m = p.len();
    let ideal = if m == 4 { 90f64 } else { 60. };
    let mut cost = if m == 4 { 1.0 } else { 1.6 };
    for k in 0..m {
        let (prev, at, next) = (p[(k + m - 1) % m], p[k], p[(k + 1) % m]);
        let turn = cross(prev, at, next);
        let (a, b) = (prev - at, next - at);
        if a.length_squared() == 0. || b.length_squared() == 0. {
            return None;
        }
        let straight = turn.abs() <= 1e-9 * scale * scale;
        if turn < 0. && !straight {
            return None;
        }
        let angle = a.angle_to(b).abs().to_degrees();
        cost += 0.25 * (angle - ideal).abs() / ideal;
        if straight || angle > 179. {
            cost += 5.;
        }
    }
    Some(cost)
}

/// The cheapest split of the polygon `corners` (anticlockwise or
/// clockwise) into quads and triangles: the new edges as corner pairs.
fn plan(
    corners: &[DVec2],
    is_new: &[bool],
    existing: &dyn Fn(usize, usize) -> bool,
) -> Option<Vec<(usize, usize)>> {
    let n = corners.len();
    // Work anticlockwise so convex corners turn left.
    let area: f64 = (0..n)
        .map(|i| corners[i].perp_dot(corners[(i + 1) % n]))
        .sum();
    let order: Vec<usize> = if area >= 0. {
        (0..n).collect()
    } else {
        (0..n).rev().collect()
    };
    let at: Vec<DVec2> = order.iter().map(|&i| corners[i]).collect();
    let scale = at
        .iter()
        .fold(0f64, |m, q| m.max(q.abs().max_element()))
        .max(1e-12);
    // Which corner pairs may be joined by a new edge.
    let mut usable = vec![vec![false; n]; n];
    for i in 0..n {
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            if existing(order[i], order[j]) {
                continue;
            }
            let (a, b) = (at[i], at[j]);
            let clear = (0..n).all(|k| {
                let k2 = (k + 1) % n;
                [k, k2].iter().any(|x| *x == i || *x == j) || !crosses(a, b, at[k], at[k2])
            });
            if clear && inside(&at, (a + b) / 2.) {
                usable[i][j] = true;
            }
        }
    }
    let diagonal_cost = |i: usize, j: usize| {
        if is_new[order[i]] || is_new[order[j]] {
            0.05
        } else {
            0.4
        }
    };
    // best[i][j]: the chain i..=j closed by i-j (j > i).
    const NONE: f64 = f64::INFINITY;
    let mut best = vec![vec![NONE; n]; n];
    let mut choice: Vec<Vec<Option<(usize, usize)>>> = vec![vec![None; n]; n];
    for i in 0..n - 1 {
        best[i][i + 1] = 0.;
    }
    // The cost of using side a-b of a piece: 0 for a polygon side, the
    // sub-chain's plan plus the new edge otherwise.
    let side = |best: &Vec<Vec<f64>>, a: usize, b: usize| -> f64 {
        if b == a + 1 {
            0.
        } else if usable[a][b] {
            best[a][b] + diagonal_cost(a, b)
        } else {
            NONE
        }
    };
    for span in 2..n {
        for i in 0..n - span {
            let j = i + span;
            if !(usable[i][j] || (i == 0 && j == n - 1)) {
                continue;
            }
            let mut found = NONE;
            let mut pick = None;
            for k in i + 1..j {
                let (s1, s2) = (side(&best, i, k), side(&best, k, j));
                if s1.is_finite()
                    && s2.is_finite()
                    && let Some(c) = piece_cost(&[at[i], at[k], at[j]], scale)
                    && c + s1 + s2 < found
                {
                    found = c + s1 + s2;
                    pick = Some((k, k));
                }
                for l in k + 1..j {
                    let (s1, s2, s3) = (side(&best, i, k), side(&best, k, l), side(&best, l, j));
                    if s1.is_finite()
                        && s2.is_finite()
                        && s3.is_finite()
                        && let Some(c) = piece_cost(&[at[i], at[k], at[l], at[j]], scale)
                        && c + s1 + s2 + s3 < found
                    {
                        found = c + s1 + s2 + s3;
                        pick = Some((k, l));
                    }
                }
            }
            best[i][j] = found;
            choice[i][j] = pick;
        }
    }
    if !best[0][n - 1].is_finite() {
        return None;
    }
    let mut out = Vec::new();
    let mut stack = vec![(0, n - 1)];
    while let Some((i, j)) = stack.pop() {
        let (k, l) = choice[i][j]?;
        let mut cuts = vec![(i, k)];
        if k != l {
            cuts.push((k, l));
        }
        cuts.push((l, j));
        for (a, b) in cuts {
            if b > a + 1 {
                out.push((order[a], order[b]));
                stack.push((a, b));
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_with_side_points() -> Vec<DVec2> {
        // A unit square with a point in the middle of its bottom and top
        // sides: a hexagon that two quads fill.
        [(0., 0.), (0.5, 0.), (1., 0.), (1., 1.), (0.5, 1.), (0., 1.)]
            .map(|(x, y)| DVec2::new(x, y))
            .to_vec()
    }

    #[test]
    fn a_hexagon_becomes_two_quads_through_the_new_points() {
        let c = square_with_side_points();
        let new = [false, true, false, false, true, false];
        let d = plan(&c, &new, &|_, _| false).unwrap();
        assert_eq!(d.len(), 1);
        let (a, b) = d[0];
        assert_eq!((a.min(b), a.max(b)), (1, 4));
    }

    #[test]
    fn a_pentagon_becomes_a_quad_and_a_triangle_from_its_new_point() {
        let c = [(0., 0.), (0.5, 0.), (1., 0.), (1., 1.), (0., 1.)]
            .map(|(x, y)| DVec2::new(x, y))
            .to_vec();
        let new = [false, true, false, false, false];
        let d = plan(&c, &new, &|_, _| false).unwrap();
        assert_eq!(d.len(), 1);
        assert!(
            d[0].0 == 1 || d[0].1 == 1,
            "the new edge starts at the new point"
        );
    }

    #[test]
    fn existing_edges_are_never_doubled() {
        let c = square_with_side_points();
        let new = [false, true, false, false, true, false];
        let taken = |a: usize, b: usize| (a.min(b), a.max(b)) == (1, 4);
        let d = plan(&c, &new, &taken).unwrap_or_default();
        assert!(d.iter().all(|&(a, b)| (a.min(b), a.max(b)) != (1, 4)));
    }
}
