//! Whole-frame timings on a generated million-polygon mesh (heavy-mesh-speed,
//! 2026-11-24), waiting for the graphics device each frame so a frame
//! includes the CPU-side geometry rebuild, the upload and the GPU drawing.
//! No scene files needed:
//! `cargo test --release -p operandi-app heavy_mesh_frames -- --ignored --nocapture`
//!
//! Cases: a still frame; Move-dragging one polygon; Move-dragging every
//! point (what a deformer does to the viewport each frame); posing the
//! elbow of a skinned arm (evaluation + skin + redraw). Each drag also
//! prints the bytes uploaded per frame.
use super::perf_drags_tests::{Bench, drag, report};
use super::*;
use operandi_core::mesh_stack::{MeshOp, MeshStack, OpKind};
use operandi_core::skin::{self, SkinValues};
use std::collections::BTreeMap;

/// A `side` × `side` quad grid in XZ, centred on the origin, `size` wide.
fn grid(side: u32, size: f64) -> operandi_core::Mesh {
    let row = side + 1;
    let step = size / f64::from(side);
    let mut mesh = operandi_core::Mesh {
        vertices: (0..row * row)
            .map(|i| {
                DVec3::new(
                    f64::from(i % row) * step - size / 2.,
                    0.,
                    f64::from(i / row) * step - size / 2.,
                )
            })
            .collect(),
        ..Default::default()
    };
    *mesh.faces = (0..side * side)
        .map(|k| {
            let (x, y) = (k % side, k / side);
            operandi_core::Face {
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

fn bench() -> (Bench, Operandi) {
    let state = super::performance_tests::headless_gpu().expect("graphics device");
    let ctx = Context::default();
    theme::install(&ctx);
    ctx.data_mut(|d| d.insert_temp(Id::new("simulation-scale"), 2f32));
    let b = Bench {
        ctx,
        device: state.device.clone(),
        size: vec2(1440., 900.),
    };
    let mut a = Operandi::empty();
    a.viewport_gpu = Some(gpu::ViewportGpu::new(state));
    a.camera.view = crate::camera::View::Perspective;
    b.frame(&mut a, vec![]);
    (b, a)
}

fn uploaded(a: &Operandi) -> u64 {
    a.rebuild_totals().uploaded
}

#[test]
#[ignore = "opt-in whole-frame timings; run in release with --nocapture"]
fn heavy_mesh_frames() {
    let side = 1000; // 1,000,000 quads
    // OPERANDI_HEAVY_ONLY=pose: only the skinned posing (for a profiler).
    if std::env::var("OPERANDI_HEAVY_ONLY").is_ok_and(|v| v == "pose") {
        return posing(side);
    }
    let (b, mut a) = bench();
    a.editor.scene.new_mesh().unwrap();
    a.editor.scene.items.last_mut().unwrap().mesh = grid(side, 4.).into();
    a.run(Command::Mode(Mode::Polygon));
    a.frame(false);
    b.frame(&mut a, vec![]);
    let mut still: Vec<f64> = (0..20).map(|_| b.frame(&mut a, vec![])).collect();
    report("grid1m/still_frame", &mut still);
    let points = a.editor.scene.items[0].mesh.vertices.len() as u32;
    let polygons = a.editor.scene.items[0].mesh.faces.len() as u32;
    for (label, mode) in [
        ("drag_1_polygon", Mode::Polygon),
        ("drag_all_points", Mode::Vertex),
    ] {
        a.run(Command::Mode(mode));
        match mode {
            Mode::Vertex => a.editor.scene.selection.vertices = (0..points).collect(),
            _ => a.editor.scene.selection.faces = vec![polygons / 2],
        }
        a.activate(Tool::Move);
        let before = uploaded(&a);
        let (mut times, _) = drag(&b, &mut a, 30);
        let per_frame = (uploaded(&a) - before) / times.len().max(1) as u64;
        report(&format!("grid1m/{label}"), &mut times);
        println!("PERF grid1m/{label}_upload bytes_per_frame={per_frame}");
        a.run(Command::Drop);
        b.frame(&mut a, vec![]);
    }
    posing(side);
}

/// The grid skinned to a two-joint arm along X; each frame turns the
/// elbow a little, as a posing drag or animation playback does.
fn posing(side: u32) {
    let (b, mut a) = bench();
    let e = &mut a.editor;
    let run = |e: &mut operandi_core::Editor, text: &str| {
        if let Err(err) = operandi_core::command_line::run_line(e, text).expect("headless") {
            panic!("{text}: {err}");
        }
    };
    run(
        e,
        "operandi.skel.add position:-2,0,0 preserveAlignment:true",
    );
    let shoulder = e.scene.items.last().unwrap().id;
    run(
        e,
        &format!("operandi.skel.add position:0,0,0 parent:{shoulder} preserveAlignment:true"),
    );
    let elbow = e.scene.items.last().unwrap().id;
    e.scene.new_mesh().unwrap();
    let mesh = e.scene.items.last().unwrap().id;
    let mut m = grid(side, 4.);
    let (mut upper, mut lower) = (BTreeMap::new(), BTreeMap::new());
    for (p, v) in m.vertices.iter().enumerate() {
        let w = ((v.x + 2.) / 4. * 3. - 1.).clamp(0., 1.);
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
    a.frame(false);
    for _ in 0..3 {
        b.frame(&mut a, vec![]);
    }
    let before = uploaded(&a);
    let frames = std::env::var("OPERANDI_PERF_DRAG_FRAMES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20u32);
    let totals = a.rebuild_totals();
    let mut times: Vec<f64> = (1..=frames)
        .map(|k| {
            a.editor.scene.item_mut(elbow).unwrap().transform.rotation.z = 0.01 * f64::from(k);
            b.frame(&mut a, vec![])
        })
        .collect();
    let per_frame = (uploaded(&a) - before) / times.len() as u64;
    println!(
        "PERF   pose before={totals:?} after={:?}",
        a.rebuild_totals()
    );
    report("grid1m/pose_skinned_elbow", &mut times);
    println!("PERF grid1m/pose_skinned_elbow_upload bytes_per_frame={per_frame}");
}
