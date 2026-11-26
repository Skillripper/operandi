//! The B0 corpus: small original scenes, one per acceptance case in
//! meshblend-2026-11-24 `cases.json` (BLD-C01..C27) and per row of
//! FAILURES.md that a scene can show. Only the inputs live here; what each
//! case expects is in the spec.
use super::poly;
use super::sharp::{Role, Source};
use crate::Mesh;
use glam::DVec3;

/// One scene: sources and strip settings.
#[derive(Clone, Debug)]
pub struct Scene {
    /// Short file-safe name.
    pub name: &'static str,
    /// The spec case it comes from (BLD-Cnn or a FAILURES id).
    pub case: &'static str,
    pub sources: Vec<Source>,
    /// Strip Width: distance from the old crease to each band edge (Q05).
    pub width: f64,
    /// Strip Profile in percent (0 chamfer, 100 round).
    pub profile: f64,
    /// Strip Rows (2 × rows quads across, SB-33).
    pub rows: u32,
    /// Subdivision level of every source (SB-20); 0 = the cage.
    pub level: u8,
}

/// A box of `size` centred on `centre` with `segments` per axis; `open`
/// leaves out the polygons on the +Y side.
pub fn boxed(centre: [f64; 3], size: [f64; 3], segments: [u32; 3], open: bool) -> Mesh {
    let mut m = crate::primitive::segmented_cube(
        DVec3::from_array(centre),
        DVec3::from_array(size),
        segments,
    )
    .expect("corpus box");
    if open {
        let top = centre[1] + size[1] * 0.5;
        let faces: Vec<_> = m
            .faces
            .iter()
            .filter(|f| {
                !f.vertices
                    .iter()
                    .all(|&v| (m.vertices[v as usize].y - top).abs() < 1e-9)
            })
            .cloned()
            .collect();
        m.faces = faces.into();
    }
    m
}

/// A Y-axis cylinder with n-gon caps.
pub fn cylinder(centre: [f64; 3], radius: f64, half: f64, sides: u32, segments: u32) -> Mesh {
    crate::radial::cylinder_geometry_with(
        DVec3::from_array(centre),
        crate::radial::RadialParameters {
            radius: DVec3::new(radius, half, radius),
            sides,
            segments,
        },
        [crate::radial::CylinderCap::Single; 2],
        1,
        false,
    )
    .expect("corpus cylinder")
}

/// BLD-C20: a unit box with one extra polygon standing on a top edge, so
/// that edge has three polygons (non-manifold).
fn finned() -> Mesh {
    let mut m = boxed([0., 0., 0.], [1., 1., 1.], [1, 1, 1], false);
    let top: Vec<u32> = (0..m.vertices.len() as u32)
        .filter(|&v| m.vertices[v as usize].y > 0.49 && m.vertices[v as usize].z > 0.49)
        .collect();
    let (a, b) = (top[0], top[1]);
    let n = m.vertices.len() as u32;
    let up = DVec3::new(0., 0.5, 0.);
    m.vertices.push(m.vertices[b as usize] + up);
    m.vertices.push(m.vertices[a as usize] + up);
    m.faces.push(poly(vec![a, b, n, n + 1]));
    m
}

/// FAILURES F06: a box whose top corner is pushed down through its bottom,
/// so the surface passes through itself.
fn self_crossing() -> Mesh {
    let mut m = boxed([0., 0., 0.], [1., 1., 1.], [1, 1, 1], false);
    let v = (0..m.vertices.len())
        .find(|&v| m.vertices[v] == DVec3::splat(0.5))
        .expect("corner");
    m.vertices[v] = DVec3::new(0.5, -1.0, 0.5);
    m
}

fn source(name: &str, mesh: Mesh, role: Role) -> Source {
    Source {
        name: name.into(),
        mesh,
        role,
    }
}

fn unit(centre: [f64; 3]) -> Mesh {
    boxed(centre, [1., 1., 1.], [2, 2, 2], false)
}

fn plate_post() -> Vec<Source> {
    vec![
        source(
            "Plate",
            boxed([0., 0., 0.], [2., 0.4, 2.], [8, 2, 8], false),
            Role::Primary,
        ),
        source(
            "Post",
            cylinder([0., 0.4, 0.], 0.5, 1.0, 16, 4),
            Role::Primary,
        ),
    ]
}

fn three_boxes() -> Vec<Source> {
    vec![
        source("A", unit([0., 0., 0.]), Role::Primary),
        source(
            "B",
            boxed([0.6, 0.3, 0.], [1., 0.6, 0.6], [2, 2, 2], false),
            Role::Primary,
        ),
        source(
            "C",
            boxed([0., 0.3, 0.6], [0.6, 0.6, 1.], [2, 2, 2], false),
            Role::Primary,
        ),
    ]
}

fn sixteen_cutters() -> Vec<Source> {
    let mut s = vec![source(
        "Plate",
        boxed([0., 0., 0.], [4., 0.4, 4.], [8, 2, 8], false),
        Role::Primary,
    )];
    for k in 0..16 {
        let (x, z) = (-1.5 + (k % 4) as f64, -1.5 + (k / 4) as f64);
        s.push(source(
            &format!("Cut{k}"),
            cylinder([x, 0.2, z], 0.25, 0.5, 12, 2),
            Role::Subtract(vec![0]),
        ));
    }
    s
}

fn scene(
    name: &'static str,
    case: &'static str,
    sources: Vec<Source>,
    width: f64,
    profile: f64,
    rows: u32,
    level: u8,
) -> Scene {
    Scene {
        name,
        case,
        sources,
        width,
        profile,
        rows,
        level,
    }
}

fn two(b_role: Role, b_centre: [f64; 3]) -> Vec<Source> {
    vec![
        source("A", unit([0., 0., 0.]), Role::Primary),
        source("B", unit(b_centre), b_role),
    ]
}

/// Every corpus scene, in case order.
pub fn all() -> Vec<Scene> {
    let half = [0.5, 0.5, 0.5];
    let sub = || Role::Subtract(vec![0]);
    let int = || Role::Intersect(vec![0]);
    vec![
        scene(
            "two-cubes",
            "BLD-C01",
            two(Role::Primary, half),
            0.1,
            100.,
            3,
            2,
        ),
        scene("subtract", "BLD-C02", two(sub(), half), 0.1, 100., 3, 2),
        scene("intersect", "BLD-C03", two(int(), half), 0.05, 100., 3, 2),
        scene("post-plate", "BLD-C07", plate_post(), 0.1, 100., 3, 2),
        scene("post-plate-w02", "BLD-C07", plate_post(), 0.2, 100., 3, 2),
        scene("post-chamfer", "BLD-C08", plate_post(), 0.2, 0., 3, 2),
        scene("post-rows1", "BLD-C11", plate_post(), 0.2, 100., 1, 2),
        scene("post-level0", "BLD-C12", plate_post(), 0.1, 100., 3, 0),
        scene("post-level1", "BLD-C12", plate_post(), 0.1, 100., 3, 1),
        scene("post-level3", "BLD-C12", plate_post(), 0.1, 100., 3, 3),
        scene(
            "sixteen-cutters",
            "BLD-C14",
            sixteen_cutters(),
            0.03,
            100.,
            3,
            2,
        ),
        scene(
            "open-primary",
            "BLD-C15",
            vec![
                source("A", boxed([0.; 3], [1.; 3], [2; 3], true), Role::Primary),
                source("B", unit([0.5, 0., 0.5]), Role::Primary),
            ],
            0.1,
            100.,
            3,
            2,
        ),
        scene(
            "open-trim",
            "BLD-C16",
            vec![
                source("A", unit([0.; 3]), Role::Primary),
                source("B", boxed(half, [1.; 3], [2; 3], true), sub()),
            ],
            0.1,
            100.,
            3,
            2,
        ),
        scene("identical", "BLD-C17", two(sub(), [0.; 3]), 0.1, 100., 3, 2),
        scene(
            "coplanar",
            "BLD-C18",
            two(sub(), [0.5, 0., 0.5]),
            0.1,
            100.,
            3,
            2,
        ),
        scene(
            "touching",
            "BLD-C19",
            two(Role::Primary, [1., 0., 0.]),
            0.1,
            100.,
            3,
            2,
        ),
        scene(
            "fin",
            "BLD-C20",
            vec![
                source(
                    "Base",
                    boxed([0., -0.6, 0.], [2., 0.4, 2.], [4, 2, 4], false),
                    Role::Primary,
                ),
                source("Fin", finned(), Role::Primary),
            ],
            0.1,
            100.,
            3,
            2,
        ),
        scene(
            "tiny-cutter",
            "BLD-C21",
            vec![
                source("A", unit([0.; 3]), Role::Primary),
                source("B", boxed(half, [0.002; 3], [1; 3], false), sub()),
            ],
            0.001,
            100.,
            3,
            2,
        ),
        scene(
            "wide-0.3",
            "BLD-C22",
            two(Role::Primary, half),
            0.3,
            100.,
            3,
            2,
        ),
        scene(
            "wide-0.6",
            "BLD-C22",
            two(Role::Primary, half),
            0.6,
            100.,
            3,
            2,
        ),
        scene(
            "wide-1.0",
            "BLD-C22",
            two(Role::Primary, half),
            1.0,
            100.,
            3,
            2,
        ),
        scene(
            "intersect-miss",
            "BLD-C23",
            two(int(), [3., 0., 0.]),
            0.1,
            100.,
            3,
            2,
        ),
        scene("three-boxes", "BLD-C25", three_boxes(), 0.05, 100., 3, 2),
        scene(
            "self-crossing",
            "F06",
            vec![
                source("A", self_crossing(), Role::Primary),
                source("B", unit([0.5, 0.5, 0.5]), Role::Primary),
            ],
            0.1,
            100.,
            3,
            2,
        ),
        scene("profile-200", "BLD-C10", plate_post(), 0.2, 200., 3, 2),
    ]
}

/// The five scenes on the comparison page (coordinator brief).
pub const PICTURED: [&str; 5] = [
    "two-cubes",
    "post-plate",
    "three-boxes",
    "sixteen-cutters",
    "wide-0.3",
];
