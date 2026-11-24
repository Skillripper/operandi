//! Opt-in timings for heavy meshes (heavy-mesh-speed, 2026-11-24): building
//! the biggest primitives, and skinning a million points. Run it in release
//! and compare before/after numbers on one machine:
//! `cargo test --release --workspace -- heavy_mesh_bench:: --ignored --nocapture`
//!
//! Each line prints the best of a few runs (other work on the machine only
//! ever makes a run slower). The drag through the viewport's GPU buffers is
//! timed in `operandi-app` (`perf_drags`, `heavy_mesh_frames`).
use glam::{DMat4, DVec3};
use operandi_core::skin::{self, SkinValues};
use operandi_core::*;
use std::time::Instant;

/// Best of `runs` runs of `f`, in ms.
fn best<T>(runs: usize, mut f: impl FnMut() -> T) -> f64 {
    (0..runs)
        .map(|_| {
            let t = Instant::now();
            std::hint::black_box(f());
            t.elapsed().as_secs_f64() * 1000.
        })
        .fold(f64::MAX, f64::min)
}

fn line(name: &str, ms: f64, polygons: usize) {
    println!("BENCH {name} ms={ms:.2} polygons={polygons}");
}

#[test]
#[ignore = "opt-in timings; run in release with --nocapture"]
fn primitives() {
    let r = DVec3::splat(1.);
    let sphere = tessellated_sphere(DVec3::ZERO, r, 6, 1, true).unwrap();
    line(
        "tessellated_sphere_level6_uv",
        best(5, || {
            tessellated_sphere(DVec3::ZERO, r, 6, 1, true).unwrap()
        }),
        sphere.faces.len(),
    );
    line(
        "tessellated_sphere_level6",
        best(5, || {
            tessellated_sphere(DVec3::ZERO, r, 6, 1, false).unwrap()
        }),
        sphere.faces.len(),
    );
    // The biggest Globe, Cylinder and segmented Cube the 250k-polygon
    // budget allows.
    let globe = RadialParameters {
        radius: r,
        sides: 500,
        segments: 499,
    };
    line(
        "globe_500x499_uv",
        best(5, || {
            globe_geometry_with(DVec3::ZERO, globe, 1, true).unwrap()
        }),
        globe_geometry(DVec3::ZERO, globe).unwrap().faces.len(),
    );
    let tube = RadialParameters {
        radius: r,
        sides: 4096,
        segments: 61,
    };
    let caps = [CylinderCap::Quads; 2];
    line(
        "cylinder_4096x61_uv",
        best(5, || {
            cylinder_geometry_with(DVec3::ZERO, tube, caps, 1, true).unwrap()
        }),
        cylinder_geometry_with(DVec3::ZERO, tube, caps, 1, false)
            .unwrap()
            .faces
            .len(),
    );
    let cube = segmented_cube(DVec3::ZERO, r, [204; 3]).unwrap();
    line(
        "segmented_cube_204",
        best(5, || segmented_cube(DVec3::ZERO, r, [204; 3]).unwrap()),
        cube.faces.len(),
    );
}

/// A `side` × `side` quad grid in XZ.
fn grid(side: u32) -> Mesh {
    let row = side + 1;
    let mut mesh = Mesh {
        vertices: (0..row * row)
            .map(|i| DVec3::new((i % row) as f64, 0., (i / row) as f64))
            .collect(),
        ..Default::default()
    };
    *mesh.faces = (0..side * side)
        .map(|k| {
            let (x, y) = (k % side, k / side);
            Face {
                vertices: vec![
                    y * row + x,
                    (y + 1) * row + x,
                    (y + 1) * row + x + 1,
                    y * row + x + 1,
                ],
                uv: None,
                material: String::new(),
                smooth: true,
                part: None,
                material_label: None,
            }
        })
        .collect();
    mesh
}

#[test]
#[ignore = "opt-in timings; run in release with --nocapture"]
fn skinning_a_million_points() {
    let side = 999; // 1000² = 1,000,000 points
    let mut mesh = grid(side);
    let n = mesh.vertices.len();
    // Four joints along X; each point is weighted to the two nearest, the
    // usual shape of a bound limb.
    let joints = 4usize;
    let width = f64::from(side);
    for j in 0..joints {
        let centre = width * (j as f64 + 0.5) / joints as f64;
        let map: std::collections::BTreeMap<u32, f64> = (0..n as u32)
            .filter_map(|p| {
                let x = mesh.vertices[p as usize].x;
                let w = 1. - (x - centre).abs() / (width / joints as f64);
                (w > 0.).then_some((p, w))
            })
            .collect();
        mesh.attributes
            .point_weights
            .insert(format!("joint{j}"), map);
    }
    let moves: Vec<Option<DMat4>> = (0..joints)
        .map(|j| {
            Some(
                DMat4::from_translation(DVec3::new(0., j as f64, 0.))
                    * DMat4::from_rotation_z(0.1 * j as f64),
            )
        })
        .collect();
    for (label, interpolation) in [
        ("linear", skin::LINEAR),
        ("dual_quaternion", skin::DUAL_QUATERNION),
    ] {
        let v = SkinValues {
            interpolation,
            joints: (1..=joints as u64).collect(),
            maps: (0..joints).map(|j| format!("joint{j}")).collect(),
            enabled: vec![true; joints],
            smooth: vec![false; joints],
            ..Default::default()
        };
        let out = skin::deform(&mesh, &v, &moves);
        assert_ne!(out.vertices, mesh.vertices);
        // A posing drag re-skins every frame with the same weights.
        line(
            &format!("skin_1m_points_{label}"),
            best(5, || skin::deform(&mesh, &v, &moves)),
            mesh.faces.len(),
        );
    }
}

/// A 1M-point grid skinned to a two-joint arm along X (shoulder at the
/// start, elbow in the middle), weights blending across the middle third:
/// the scene a posing drag re-evaluates every frame.
fn skinned_arm(side: u32) -> (Editor, u64, u64) {
    use operandi_core::mesh_stack::{MeshOp, MeshStack, OpKind};
    let run = |e: &mut Editor, text: &str| {
        if let Err(err) = command_line::run_line(e, text).expect("headless") {
            panic!("{text}: {err}");
        }
    };
    let width = f64::from(side);
    let mut e = Editor::default();
    run(
        &mut e,
        "operandi.skel.add position:0,0,0 preserveAlignment:true",
    );
    let shoulder = e.scene.items.last().unwrap().id;
    run(
        &mut e,
        &format!(
            "operandi.skel.add position:{},0,0 parent:{shoulder} preserveAlignment:true",
            width / 2.
        ),
    );
    let elbow = e.scene.items.last().unwrap().id;
    e.scene.new_mesh().unwrap();
    let mesh = e.scene.items.last().unwrap().id;
    let mut m = grid(side);
    let mut upper = std::collections::BTreeMap::new();
    let mut lower = std::collections::BTreeMap::new();
    for (p, v) in m.vertices.iter().enumerate() {
        let w = ((v.x / width - 1. / 3.) * 3.).clamp(0., 1.);
        if w < 1. {
            upper.insert(p as u32, 1. - w);
        }
        if w > 0. {
            lower.insert(p as u32, w);
        }
    }
    m.attributes.point_weights.insert("Shoulder".into(), upper);
    m.attributes.point_weights.insert("Elbow".into(), lower);
    let mut values = SkinValues {
        interpolation: skin::LINEAR,
        ..Default::default()
    };
    values.add(shoulder, "Shoulder");
    values.add(elbow, "Elbow");
    let item = e.scene.item_mut(mesh).unwrap();
    item.mesh = m.into();
    item.mesh_stack = Some(MeshStack {
        ops: vec![MeshOp {
            id: 0,
            enabled: true,
            kind: OpKind::Op {
                type_id: skin::SKIN_OP.into(),
                values: serde_json::to_value(values).unwrap(),
            },
            selection: None,
            selops: Vec::new(),
        }],
        next_id: 1,
    });
    for j in [shoulder, elbow] {
        skin::ensure_setup_channels(&mut e.scene, j);
    }
    (e, mesh, elbow)
}

#[test]
#[ignore = "opt-in timings; run in release with --nocapture"]
fn posing_a_skinned_million_points() {
    let (mut e, mesh, elbow) = skinned_arm(999);
    let mut frame = 0;
    let ms = best(8, || {
        frame += 1;
        e.scene.item_mut(elbow).unwrap().transform.rotation.z = 0.01 * f64::from(frame);
        let s = e.evaluated(evaluate::Purpose::Final);
        assert_ne!(
            s.item(mesh).unwrap().mesh.open().vertices[999_999],
            e.scene.item(mesh).unwrap().mesh.vertices[999_999]
        );
    });
    line(
        "pose_skinned_1m_points_evaluate",
        ms,
        e.scene.item(mesh).unwrap().mesh.faces.len(),
    );
}
