//! Edge Bevel's Corner Pattern (a tester's request, 2026-11-24, from a
//! picture of an L-shaped block's inner corner): how the opening where
//! three rounded strips meet is filled. Patch is the measured EBC-J-05
//! grid (the default); Uniform fans lines to one centre point; Radial adds
//! rings around it; None closes the opening with one polygon.
use crate::edge_bevel_loop_slide::hole;
use glam::DVec3;
use operandi_core::*;

fn cube() -> Mesh {
    primitive::cube_geometry(DVec3::ZERO, DVec3::splat(2.)).unwrap()
}

/// The three edges at the cube's corner (1, 1, 1).
fn cube_corner(m: &Mesh) -> Vec<(u32, u32)> {
    let corner = m
        .vertices
        .iter()
        .position(|p| p.distance(DVec3::ONE) < 1e-9)
        .unwrap() as u32;
    m.edges()
        .into_iter()
        .filter(|&(a, b)| a == corner || b == corner)
        .collect()
}

/// The hole's rim and its four vertical corners: three strips meet at
/// each rim corner, with the inset's diagonal running in too.
const HOLE: [(u32, u32); 8] = [
    (4, 5),
    (5, 6),
    (6, 7),
    (4, 7),
    (4, 8),
    (5, 9),
    (6, 10),
    (7, 11),
];

fn bevel(m: &Mesh, edges: &[(u32, u32)], level: u32, corner: CornerPattern) -> Mesh {
    let settings = EdgeBevelSettings {
        width: 0.1,
        rows: 2 * level,
        depth: 100.,
        chamfer: false,
        options: EdgeBevelOptions {
            corner,
            ..Default::default()
        },
    };
    let out = edge_bevel_with(m, edges, &settings, None)
        .unwrap_or_else(|e| panic!("{corner:?} level {level}: {e}"));
    out.mesh
        .validate()
        .unwrap_or_else(|e| panic!("{corner:?} level {level}: {e}"));
    out.mesh
}

fn sides(m: &Mesh, n: usize) -> usize {
    m.faces.iter().filter(|f| f.vertices.len() == n).count()
}

#[test]
fn every_pattern_closes_the_corner() {
    let c = cube();
    let shapes: [(&str, Mesh, Vec<(u32, u32)>, usize); 2] = [
        ("cube corner", c.clone(), cube_corner(&c), 1),
        ("hole corners", hole(), HOLE.to_vec(), 4),
    ];
    for (name, m, edges, corners) in shapes {
        for level in [1, 2, 3] {
            let n = 2 * level as usize;
            // The opening at each corner: three strip corners, each with
            // n − 1 profile rows after it.
            let opening = 3 * n;
            let patch = bevel(&m, &edges, level, CornerPattern::Patch);
            let uniform = bevel(&m, &edges, level, CornerPattern::Uniform);
            let radial = bevel(&m, &edges, level, CornerPattern::Radial);
            let none = bevel(&m, &edges, level, CornerPattern::None);
            let what = format!("{name}, level {level}");
            assert_eq!(
                sides(&none, opening),
                corners + sides(&patch, opening),
                "{what}: None closes each corner with one {opening}-sided polygon"
            );
            assert_eq!(
                sides(&uniform, 3),
                corners * opening + sides(&none, 3),
                "{what}: Uniform fans {opening} triangles at each corner"
            );
            let rings = level as usize;
            assert_eq!(
                sides(&radial, 3),
                sides(&uniform, 3),
                "{what}: Radial ends in the same fan"
            );
            assert_eq!(
                sides(&radial, 4),
                sides(&none, 4) + corners * opening * rings,
                "{what}: Radial adds {rings} rings of {opening} quads"
            );
            assert_ne!(
                patch.faces.len(),
                none.faces.len(),
                "{what}: Patch is its own fill"
            );
        }
    }
}

#[test]
fn the_centre_sits_on_the_rounded_corner() {
    // On the cube's corner, Uniform's centre point is the one point the
    // corner's triangles share; it lies inside the cube, past the strips'
    // chord and short of the old corner, on the corner's diagonal.
    let c = cube();
    let m = bevel(&c, &cube_corner(&c), 2, CornerPattern::Uniform);
    let mut uses = vec![0usize; m.vertices.len()];
    for f in m.faces.iter().filter(|f| f.vertices.len() == 3) {
        for &v in &f.vertices {
            uses[v as usize] += 1;
        }
    }
    let centre = m.vertices[(0..uses.len()).max_by_key(|&i| uses[i]).unwrap()];
    let diagonal = DVec3::ONE.normalize();
    assert!(
        (centre - diagonal * centre.dot(diagonal)).length() < 1e-9,
        "{centre} is off the corner's diagonal"
    );
    assert!(centre.max_element() < 1., "{centre} is outside the cube");
    assert!(centre.min_element() > 0.9, "{centre} is too far in");
}

#[test]
fn patch_is_the_default() {
    assert_eq!(EdgeBevelOptions::default().corner, CornerPattern::Patch);
}
