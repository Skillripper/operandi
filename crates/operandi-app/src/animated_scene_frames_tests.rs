//! Scrubbing and playing an animated scene shaped like an imported USD
//! shot (heavy-mesh-speed, 2026-11-24): 191 meshes under one moving group
//! locator, and a keyed camera. Only transforms change from frame to
//! frame, so a frame should cost what drawing costs, not a rebuild of
//! every mesh. Synthetic: no imported file is read.
//! `cargo test --release -p operandi-app animated_scene_frames -- --ignored --nocapture`
use super::perf_drags_tests::{Bench, report};
use super::*;
use operandi_core::item_list::AddItemKind;

/// A `side` × `side` quad grid in XZ, `size` wide, centred on `at`.
fn grid(side: u32, size: f64, at: DVec3) -> operandi_core::Mesh {
    let row = side + 1;
    let step = size / f64::from(side);
    let mut mesh = operandi_core::Mesh {
        vertices: (0..row * row)
            .map(|i| {
                at + DVec3::new(
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

/// The shot: a group locator keyed to travel and turn over 100 frames,
/// `meshes` meshes of `side`² polygons parented under it, and a keyed
/// camera. Returns the app with the viewport drawn once.
pub(super) fn shot(meshes: usize, side: u32) -> (Bench, Operandi) {
    let state = super::performance_tests::headless_gpu().expect("graphics device");
    let ctx = Context::default();
    theme::install(&ctx);
    let b = Bench {
        ctx,
        device: state.device.clone(),
        size: vec2(1440., 900.),
    };
    let mut a = Operandi::empty();
    a.viewport_gpu = Some(gpu::ViewportGpu::new(state));
    a.camera.view = crate::camera::View::Perspective;
    a.add_item(AddItemKind::GroupLocator);
    let ship = a.editor.scene.items.last().unwrap().id;
    for k in 0..meshes {
        a.editor.scene.new_mesh().unwrap();
        let item = a.editor.scene.items.last_mut().unwrap();
        let at = DVec3::new((k % 16) as f64 * 1.2, (k / 16) as f64 * 0.3, 0.);
        item.mesh = grid(side, 1., at).into();
        item.parent = Some(ship);
    }
    for (f, x, r) in [(0, 0., 0.), (50, 6., 20.), (100, 12., 0.)] {
        a.execute(&format!(
            "operandi.key.add channel:{ship}:pos.X frame:{f} value:{x}"
        ));
        a.execute(&format!(
            "operandi.key.add channel:{ship}:rot.Y frame:{f} value:{r}"
        ));
    }
    // Relationship links, as an import makes them: 37 meshes turn with
    // the locator's travel.
    let meshes_ids: Vec<u64> = a
        .editor
        .scene
        .items
        .iter()
        .filter(|i| i.parent == Some(ship))
        .map(|i| i.id)
        .take(37)
        .collect();
    for m in meshes_ids {
        a.execute(&format!(
            "operandi.rig.linkAdd driven:{m}:rot.Z driver:{ship}:pos.X"
        ));
    }
    a.add_item(AddItemKind::Camera);
    let camera = a.editor.scene.items.last().unwrap().id;
    for (f, z) in [(0, 20.), (100, 30.)] {
        a.execute(&format!(
            "operandi.key.add channel:{camera}:pos.Z frame:{f} value:{z}"
        ));
    }
    a.run(Command::Mode(Mode::Item));
    a.editor.scene.select_item(None);
    a.execute("operandi.time.set frame:0");
    a.frame(false);
    for _ in 0..3 {
        b.frame(&mut a, vec![]);
    }
    (b, a)
}

#[test]
#[ignore = "opt-in whole-frame timings; run in release with --nocapture"]
fn animated_scene_frames() {
    let (b, mut a) = shot(191, 64); // 191 × 4,096 = 782k polygons
    let mut still: Vec<f64> = (0..10).map(|_| b.frame(&mut a, vec![])).collect();
    report("shot191/still_frame", &mut still);
    // Scrubbing: a new time each frame.
    let before = a.rebuild_totals();
    // Each scrub step split: setting the time (keys, posing) and the frame
    // that draws it (evaluation, geometry, upload, GPU drawing).
    let mut pose = Vec::new();
    let mut scrub: Vec<f64> = (1..=20)
        .map(|f| {
            let t = std::time::Instant::now();
            a.execute(&format!("operandi.time.set frame:{}", f * 2));
            pose.push(super::perf_drags_tests::ms(t));
            b.frame(&mut a, vec![])
        })
        .collect();
    report("shot191/scrub_set_time", &mut pose);
    println!(
        "PERF   scrub before={before:?} after={:?}",
        a.rebuild_totals()
    );
    report("shot191/scrub", &mut scrub);
    // Playback, one frame per drawn frame.
    a.execute("operandi.time.settings realTime:false");
    a.execute("operandi.time.set frame:0");
    a.execute("operandi.time.play");
    let mut play: Vec<f64> = (0..20).map(|_| b.frame(&mut a, vec![])).collect();
    report("shot191/playback", &mut play);
    println!("PERF   before stop {:?}", a.rebuild_totals());
    a.execute("operandi.time.play stop:true");
    // After it stops, the moved meshes are rebuilt at their places a few
    // at a time.
    // (The first frame after Stop still counts as moving time.)
    let mut settle = vec![b.frame(&mut a, vec![]), b.frame(&mut a, vec![])];
    while a.resident_cache.deferred() && settle.len() < 200 {
        settle.push(b.frame(&mut a, vec![]));
    }
    super::perf_drags_tests::one("shot191/settle_frames", settle.len() as f64);
    println!("PERF   settle totals {:?}", a.rebuild_totals());
    println!("PERF   settle times {settle:?}");
    report("shot191/settling", &mut settle);
    // OPERANDI_HEAVY_SETTLE_LOOPS: scrub and settle again and again, for a
    // profiler.
    let loops: usize = std::env::var("OPERANDI_HEAVY_SETTLE_LOOPS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    for k in 0..loops {
        a.execute(&format!("operandi.time.set frame:{}", k % 90 + 3));
        for _ in 0..20 {
            b.frame(&mut a, vec![]);
        }
    }
}

/// Playback and scrubbing move items without rebuilding their GPU
/// geometry; once time stops they settle, and each is then exactly what a
/// full build at its place gives.
#[test]
fn moving_items_reuse_their_geometry_then_settle_at_their_places() {
    let Some(state) = super::performance_tests::headless_gpu() else {
        return;
    };
    drop(state);
    // 6 × 52,900 polygons: more than one frame's settle budget.
    let (b, mut a) = shot(6, 230);
    let builds = a.rebuild_totals().gpu_meshes;
    a.execute("operandi.time.settings realTime:false");
    a.execute("operandi.time.play");
    for _ in 0..5 {
        b.frame(&mut a, vec![]);
    }
    for f in [10, 20, 30] {
        a.execute(&format!("operandi.time.set frame:{f}"));
        b.frame(&mut a, vec![]);
    }
    assert_eq!(
        a.rebuild_totals().gpu_meshes,
        builds,
        "no rebuilds while time moves"
    );
    a.execute("operandi.time.play stop:true");
    let mut frames = 1;
    b.frame(&mut a, vec![]);
    b.frame(&mut a, vec![]);
    while a.resident_cache.deferred() {
        b.frame(&mut a, vec![]);
        frames += 1;
        assert!(frames < 50, "settles");
    }
    assert!(frames > 1, "settled over several frames, not one");
    let shown = a
        .editor
        .shown_display()
        .unwrap_or_else(|| a.editor.scene.clone());
    for item in shown.items.iter().filter(|i| i.is_mesh()) {
        let (cached, _) = a.resident_cache.cached(item.id).expect("drawn");
        let matrix = shown.world_matrix(item.id).unwrap();
        let fresh = crate::gpu_scene::Resident::default()
            .item(item, matrix, None, None, &shown.smoothing)
            .0;
        assert!(
            cached.fill == fresh.fill,
            "item {} settled at its place",
            item.id
        );
        assert!(cached.edges == fresh.edges);
    }
}
