//! The tester's corner-pattern shape (2026-11-24, from Maya and Blender
//! pictures): an L-shaped block, a 2 × 2 × 2 box with its front-right
//! quarter cut out, its top split into three quads whose lines meet at the
//! inner corner. The border loop of the two inner walls is bevelled: the
//! top and bottom inner edges, the vertical inner corner and the two outer
//! verticals. At the inner top corner three strips meet (two top ones and
//! the vertical one) and the top's split lines run in too.
use glam::DVec3;
use operandi_core::*;

/// Grid points x, z in {-1, 0, 1} except the cut-out corner (1, -1), at
/// y = 1 (top) and y = -1 (bottom).
const GRID: [(f64, f64); 8] = [
    (-1., -1.),
    (0., -1.),
    (-1., 0.),
    (0., 0.),
    (1., 0.),
    (-1., 1.),
    (0., 1.),
    (1., 1.),
];

fn at(x: f64, z: f64, top: bool) -> u32 {
    let i = GRID.iter().position(|&g| g == (x, z)).expect("grid point") as u32;
    if top { i } else { i + 8 }
}

/// A polygon wound to face `out`.
fn face(m: &Mesh, mut v: Vec<u32>, out: DVec3) -> Face {
    let p = |i: u32| m.vertices[i as usize];
    let mut n = DVec3::ZERO;
    for k in 0..v.len() {
        n += p(v[k]).cross(p(v[(k + 1) % v.len()]));
    }
    if n.dot(out) < 0. {
        v.reverse();
    }
    Face {
        vertices: v,
        uv: None,
        material: String::new(),
        smooth: false,
        part: None,
        material_label: None,
    }
}

pub(crate) fn l_shape() -> Mesh {
    let mut m = Mesh::default();
    for top in [true, false] {
        for &(x, z) in &GRID {
            m.vertices
                .push(DVec3::new(x, if top { 1. } else { -1. }, z));
        }
    }
    let mut faces = Vec::new();
    // Top and bottom: the three remaining cells.
    for (x0, z0) in [(-1., -1.), (-1., 0.), (0., 0.)] {
        let (x1, z1) = (x0 + 1., z0 + 1.);
        for top in [true, false] {
            let v = vec![
                at(x0, z0, top),
                at(x0, z1, top),
                at(x1, z1, top),
                at(x1, z0, top),
            ];
            faces.push(face(&m, v, if top { DVec3::Y } else { DVec3::NEG_Y }));
        }
    }
    // Walls: each from (x0, z0) to (x1, z1) along the outline, facing `out`.
    type Wall = ((f64, f64), (f64, f64), DVec3);
    let walls: [Wall; 8] = [
        ((-1., -1.), (-1., 0.), DVec3::NEG_X),
        ((-1., 0.), (-1., 1.), DVec3::NEG_X),
        ((-1., 1.), (0., 1.), DVec3::Z),
        ((0., 1.), (1., 1.), DVec3::Z),
        ((1., 1.), (1., 0.), DVec3::X),
        ((-1., -1.), (0., -1.), DVec3::NEG_Z),
        // The two inner walls of the cut-out.
        ((0., -1.), (0., 0.), DVec3::X),
        ((0., 0.), (1., 0.), DVec3::NEG_Z),
    ];
    for ((x0, z0), (x1, z1), out) in walls {
        let v = vec![
            at(x0, z0, true),
            at(x1, z1, true),
            at(x1, z1, false),
            at(x0, z0, false),
        ];
        faces.push(face(&m, v, out));
    }
    m.faces = faces.into();
    m
}

/// The border loop of the two inner walls.
pub(crate) fn l_edges() -> Vec<(u32, u32)> {
    let key = |a: u32, b: u32| (a.min(b), a.max(b));
    let mut e = Vec::new();
    for top in [true, false] {
        e.push(key(at(0., -1., top), at(0., 0., top)));
        e.push(key(at(0., 0., top), at(1., 0., top)));
    }
    for (x, z) in [(0., -1.), (0., 0.), (1., 0.)] {
        e.push(key(at(x, z, true), at(x, z, false)));
    }
    e
}

#[test]
fn the_l_shape_is_a_closed_solid() {
    let m = l_shape();
    m.validate().unwrap();
    assert_eq!(m.faces.len(), 14);
    let all = m.edges();
    for e in l_edges() {
        assert!(all.binary_search(&e).is_ok(), "{e:?} isn't an edge");
    }
}

/// The polygons within `reach` of the inner top corner (0, 1, 0).
fn at_corner(m: &Mesh, reach: f64) -> Vec<usize> {
    let corner = DVec3::new(0., 1., 0.);
    m.faces
        .iter()
        .map(|f| f.vertices.len())
        .zip(m.faces.iter())
        .filter(|(_, f)| {
            f.vertices
                .iter()
                .all(|&v| m.vertices[v as usize].distance(corner) < reach)
        })
        .map(|(n, _)| n)
        .collect()
}

/// The tester's report: every Corner Pattern left the inner top corner as
/// one many-sided polygon (the top's split lines add a second corner point
/// to its opening). Each pattern now fills it its own way.
#[test]
fn every_pattern_fills_the_inner_corner() {
    let m = l_shape();
    for level in [2u32, 3] {
        let n = 2 * level as usize;
        let mut seen = Vec::new();
        for corner in [
            CornerPattern::Patch,
            CornerPattern::Uniform,
            CornerPattern::Radial,
            CornerPattern::None,
        ] {
            let settings = EdgeBevelSettings {
                width: 0.3,
                rows: n as u32,
                depth: 100.,
                chamfer: false,
                options: EdgeBevelOptions {
                    corner,
                    ..Default::default()
                },
            };
            let out = edge_bevel_with(&m, &l_edges(), &settings, None).unwrap();
            out.mesh.validate().unwrap();
            let mut fill = at_corner(&out.mesh, 0.45);
            fill.sort_unstable();
            let what = format!("{corner:?}, level {level}");
            let biggest = *fill.last().unwrap();
            assert!(biggest <= 4, "{what}: a {biggest}-sided polygon is left");
            assert!(fill.len() >= 3 * n, "{what}: only {} polygons", fill.len());
            if corner == CornerPattern::None {
                // An even grid under a rounded outline: all quads.
                assert!(fill.iter().all(|&s| s == 4), "{what}: {fill:?}");
            }
            let mut at: Vec<[i64; 3]> = out
                .mesh
                .vertices
                .iter()
                .map(|p| (*p * 1e6).round().as_i64vec3().to_array())
                .collect();
            at.sort_unstable();
            assert!(!seen.contains(&at), "{what} fills like another pattern");
            seen.push(at);
            if matches!(corner, CornerPattern::Radial | CornerPattern::Uniform) {
                // The pole is the top mitre point: every triangle of the
                // fill has it as a corner (the tester's Maya pictures).
                let mitre = out
                    .mesh
                    .vertices
                    .iter()
                    .position(|p| {
                        let r = 0.3 * (1. - std::f64::consts::FRAC_1_SQRT_2);
                        p.distance(DVec3::new(-r, 1., r)) < 1e-9
                    })
                    .unwrap() as u32;
                let corner_point = DVec3::new(0., 1., 0.);
                let triangles: Vec<&Face> = out
                    .mesh
                    .faces
                    .iter()
                    .filter(|f| {
                        f.vertices.len() == 3
                            && f.vertices.iter().all(|&v| {
                                out.mesh.vertices[v as usize].distance(corner_point) < 0.45
                            })
                    })
                    .collect();
                assert_eq!(triangles.len(), n, "{what}: n triangles at the pole");
                assert!(
                    triangles.iter().all(|f| f.vertices.contains(&mitre)),
                    "{what}: a triangle misses the top mitre point"
                );
            }
        }
    }
}

/// The two top strips meet at one top point, the middle of the rounded
/// outline between where they would slide along the two split lines (0.3
/// from the corner), just inside the old corner, and both split lines end
/// there, as in the tester's Maya pictures.
#[test]
fn the_top_strips_meet_at_one_mitre_point() {
    let m = l_shape();
    let settings = EdgeBevelSettings {
        width: 0.3,
        rows: 4,
        depth: 100.,
        chamfer: false,
        options: Default::default(),
    };
    let out = edge_bevel_with(&m, &l_edges(), &settings, None).unwrap();
    let mm = &out.mesh;
    let find = |p: DVec3| {
        mm.vertices
            .iter()
            .position(|&q| q.distance(p) < 1e-9)
            .unwrap_or_else(|| panic!("no point at {p}")) as u32
    };
    let r = 0.3 * (1. - std::f64::consts::FRAC_1_SQRT_2);
    let mitre = find(DVec3::new(-r, 1., r));
    let key = |a: u32, b: u32| (a.min(b), a.max(b));
    let edges = mm.edges();
    for from in [DVec3::new(-1., 1., 0.), DVec3::new(0., 1., 1.)] {
        assert!(
            edges.binary_search(&key(find(from), mitre)).is_ok(),
            "the split line from {from} doesn't end at the mitre point"
        );
    }
}
