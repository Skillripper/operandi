//! LXO import of animation, cameras, lights and rigs (spec
//! lxo-import-anim-rig-2026-11-24, LXA-01..90), checked against the eight
//! fixture scenes and the values in the spec's `cases.json`.
use glam::{DMat4, DVec3, DVec4};
use operandi_core::*;
use serde_json::Value;

fn dir() -> String {
    concat!(
        env!("OPERANDI_SPECS_DIR"),
        "/specifications/lxo-import-anim-rig-2026-11-24"
    )
    .to_owned()
}

fn bytes(file: &str) -> Vec<u8> {
    std::fs::read(format!("{}/fixtures/{file}", dir())).unwrap()
}

fn cases() -> Value {
    serde_json::from_str(&std::fs::read_to_string(format!("{}/cases.json", dir())).unwrap())
        .unwrap()
}

fn import(file: &str) -> lxo::Candidate {
    let mut progress = |_: &str| true;
    let mut c = lxo::Control::new(&mut progress);
    lxo::parse(&bytes(file), &mut c).unwrap_or_else(|e| panic!("{file}: {e}"))
}

fn editor(file: &str) -> Editor {
    Editor {
        scene: import(file).scene,
        ..Default::default()
    }
}

fn at_frame(e: &mut Editor, frame: i64) {
    let t = e.scene.time.rate.tick_of(frame);
    e.scene.time.current = t;
    animation::pose(&mut e.scene, t);
}

fn id(e: &Editor, name: &str) -> u64 {
    e.scene
        .items
        .iter()
        .find(|i| i.name == name || i.source_display_name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("no item {name}"))
        .id
}

/// The evaluated world of `name` now (keys, rig and IK applied).
fn world(e: &Editor, name: &str) -> DMat4 {
    let s = e.evaluated(evaluate::Purpose::Final);
    s.world_matrix(id(e, name)).unwrap()
}

/// `cases.json` matrices are rows, the last row the position.
fn matrix(rows: &Value) -> DMat4 {
    let r: Vec<DVec4> = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let v: Vec<f64> = row
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
                .collect();
            DVec4::new(v[0], v[1], v[2], v[3])
        })
        .collect();
    DMat4::from_cols(r[0], r[1], r[2], r[3])
}

fn close(a: DMat4, b: DMat4, tol: f64) -> bool {
    a.to_cols_array()
        .iter()
        .zip(b.to_cols_array())
        .all(|(x, y)| (x - y).abs() <= tol)
}

/// Every world matrix of a fixture's `world` table that doesn't match,
/// as text (empty when all do).
fn world_mismatches(file: &str, skip: &[&str]) -> Vec<String> {
    world_mismatches_pinned(file, skip, &[])
}

/// As `world_mismatches`, with the `pinned` items' X position kept at its
/// saved value: `cases.json` was read with the file's unkeyed edits in
/// force (LXA-03), which the import drops (Q-LXA-01).
fn world_mismatches_pinned(file: &str, skip: &[&str], pinned: &[&str]) -> Vec<String> {
    let all = cases();
    let table = &all[file]["expected"]["world"];
    let mut e = editor(file);
    for n in pinned {
        let key = animation::ChannelKey::new(id(&e, n), "pos.X");
        assert!(e.scene.animation.channels.remove(&key).is_some(), "{n}");
    }
    let mut out = Vec::new();
    for (name, frames) in table.as_object().unwrap() {
        if skip.contains(&name.as_str()) {
            continue;
        }
        for (frame, rows) in frames.as_object().unwrap() {
            at_frame(&mut e, frame.parse().unwrap());
            let want = matrix(rows);
            let got = world(&e, name);
            if !close(got, want, 2e-4) {
                out.push(format!(
                    "{name} @{frame}: got {:?}, want {:?}",
                    got.to_cols_array(),
                    want.to_cols_array()
                ));
            }
        }
    }
    out
}

// --- LXA-10..21: curves ----------------------------------------------------

#[test]
fn lxa16_every_curve_matches_modo_at_every_frame() {
    let all = cases();
    let mut failures = Vec::new();
    let mut checked = 0;
    for case in all["curves"].as_array().unwrap() {
        let file = case["file"].as_str().unwrap();
        let item = case["item"].as_str().unwrap();
        let channel = case["channel"].as_str().unwrap();
        let curves = lxo::keyed_curves(&bytes(file)).unwrap();
        let Some((_, _, curve)) = curves.iter().find(|(i, c, _)| i == item && c == channel) else {
            failures.push(format!("{file} {item} {channel}: no curve"));
            continue;
        };
        let rate = time_tick::FrameRate::FPS_30;
        for s in case["samples"].as_array().unwrap() {
            let frame = s[0].as_f64().unwrap();
            let want = s[1].as_f64().unwrap();
            let got = curve.value_at(rate.tick_of(frame as i64));
            checked += 1;
            if (got - want).abs() > 1e-4 {
                failures.push(format!("{file} {item} {channel} f{frame}: {got} vs {want}"));
            }
        }
    }
    assert!(checked > 3000, "only {checked} samples");
    assert!(
        failures.is_empty(),
        "{} of {checked} samples differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn lxa40_frame_rate_range_and_current_time() {
    let s = import("constraints.lxo").scene;
    assert_eq!(s.time.rate, time_tick::FrameRate::FPS_30);
    assert_eq!(s.time.start, time_tick::Tick::ZERO);
    assert_eq!(s.time.end, s.time.rate.tick_of(150));
    assert_eq!(s.time.current, time_tick::Tick::ZERO);
}

// --- LXA-30..33, 50..54: keyed transforms, cameras and lights --------------

#[test]
fn lxa30_keyed_transforms_cameras_and_lights() {
    let bad = world_mismatches("xfrm-camera-lights.lxo", &[]);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
    let all = cases();
    let mut e = editor("xfrm-camera-lights.lxo");
    for (frame, focal) in all["xfrm-camera-lights.lxo"]["expected"]["focal"]
        .as_object()
        .unwrap()
    {
        at_frame(&mut e, frame.parse().unwrap());
        let shot = e.scene.item(id(&e, "Shot")).unwrap();
        let got = shot.camera.as_ref().unwrap().focal_length;
        assert!(
            (got - focal.as_f64().unwrap()).abs() < 1e-6,
            "f{frame}: {got}"
        );
    }
}

#[test]
fn lxa31_keyed_meshes_import() {
    let s = import("xfrm-camera-lights.lxo").scene;
    let b = s.items.iter().find(|i| i.name == "Box").unwrap();
    // The fixture's Box is an empty mesh item whose rotation and scale
    // are keyed.
    assert!(b.camera.is_none() && b.light.is_none() && b.locator.is_none());
    assert!(s.animation.item_is_animated(b.id));
}

#[test]
fn lxa50_every_light_type_and_its_material() {
    use camera_light::LightKind as K;
    let mut e = editor("xfrm-camera-lights.lxo");
    for (name, kind) in [
        ("L_sunLight", K::Directional),
        ("L_pointLight", K::Point),
        ("L_spotLight", K::Spot),
        ("L_areaLight", K::Area),
        ("L_cylinderLight", K::Cylinder),
        ("L_domeLight", K::Dome),
        ("L_photometryLight", K::Photometric),
        ("L_portal", K::Portal),
    ] {
        let l = e.scene.item(id(&e, name)).unwrap();
        assert_eq!(l.light.as_ref().map(|l| l.kind), Some(kind), "{name}");
    }
    // The point light's intensity is keyed 2 → 4 over frames 10–20 and its
    // light material's red 1 → 0; the spot's intensity 1 → 5, cone 0.5 → 0.8.
    let light = |e: &Editor, n: &str| e.scene.item(id(e, n)).unwrap().light.clone().unwrap();
    at_frame(&mut e, 10);
    assert!((light(&e, "L_pointLight").intensity - 2.).abs() < 1e-6);
    at_frame(&mut e, 20);
    assert!((light(&e, "L_pointLight").intensity - 4.).abs() < 1e-6);
    at_frame(&mut e, 30);
    let s = light(&e, "L_spotLight");
    assert!(
        (s.intensity - 5.).abs() < 1e-6 && (s.cone_angle - 0.8).abs() < 1e-6,
        "{s:?}"
    );
    let keyed_red = e
        .scene
        .animation
        .channels
        .keys()
        .any(|k| k.item == id(&e, "L_pointLight") && k.channel == "color.r");
    assert!(
        keyed_red,
        "the light material's colour key lands on the light"
    );
}

// --- LXA-60..64: joints and skins -------------------------------------------

#[test]
fn lxa61_skin_deforms_like_modo_at_frame_30() {
    let all = cases();
    let want = &all["skin.lxo"]["expected"]["mesh_t30"]["points"];
    let mut e = editor("skin.lxo");
    for j in ["j0", "j1", "j2"] {
        assert!(
            e.scene.item(id(&e, j)).unwrap().is_joint(),
            "{j} is a joint"
        );
    }
    let arm = id(&e, "Arm");
    let maps: Vec<String> = e
        .scene
        .item(arm)
        .unwrap()
        .mesh
        .attributes
        .point_weights
        .keys()
        .cloned()
        .collect();
    assert!(
        ["j0", "j1", "j2"]
            .iter()
            .all(|j| maps.iter().any(|m| m == j)),
        "weight maps renamed to the joints: {maps:?}"
    );
    at_frame(&mut e, 30);
    let s = e.evaluated(evaluate::Purpose::Final);
    let mesh = &s.item(arm).unwrap().mesh;
    for (i, p) in want.as_array().unwrap().iter().enumerate() {
        let w = DVec3::new(
            p[0].as_f64().unwrap(),
            p[1].as_f64().unwrap(),
            p[2].as_f64().unwrap(),
        );
        let got = s
            .world_matrix(arm)
            .unwrap()
            .transform_point3(mesh.vertices[i]);
        assert!((got - w).length() < 1e-4, "point {i}: {got} vs {w}");
    }
}

// --- LXA-70..74: constraints, links, modifiers ------------------------------

#[test]
fn lxa72_constraints_and_dynamic_parent() {
    // Dst_expr's world row in cases.json stays at the origin while the
    // same file's modifier results give it Y 1, 4, 7 (LXO-AR-Q02); the
    // modifier results are checked in the next test.
    let targets = ["T_pos", "T_rot", "T_scl", "T_posoff", "T_dir", "T_par"];
    let bad = world_mismatches_pinned("constraints.lxo", &["Dst_expr"], &targets);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn lxa74_math_expression_and_link_results() {
    let all = cases();
    let mods = &all["constraints.lxo"]["expected"]["modifiers_eval"];
    let mut e = editor("constraints.lxo");
    for (key, item) in [
        ("math", "Dst_math"),
        ("expr", "Dst_expr"),
        ("link", "Dst_link"),
    ] {
        for (frame, v) in mods[key].as_object().unwrap() {
            at_frame(&mut e, frame.parse().unwrap());
            let w = world(&e, item).w_axis.truncate();
            let want = DVec3::new(
                v[0].as_f64().unwrap(),
                v[1].as_f64().unwrap(),
                v[2].as_f64().unwrap(),
            );
            assert!((w - want).length() < 1e-4, "{key} f{frame}: {w} vs {want}");
        }
    }
}

// --- LXA-80: two-bone IK and stacked constraints ----------------------------

/// Q-LXA-08: Operandi's solver keeps the knee on the side the setup pose
/// bends it (+Z here); Modo's file bends it to −Z. The chain is compared
/// by what both agree on: the end joint reaches the goal, the bones keep
/// their lengths and the chain stays in its plane.
#[test]
fn lxa80_two_bone_ik_and_stacked_constraints() {
    let bad = world_mismatches("ik-and-stacked-constraints.lxo", &["ik0", "ik1", "ik2"]);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
    let all = cases();
    let table = &all["ik-and-stacked-constraints.lxo"]["expected"]["world"];
    let mut e = editor("ik-and-stacked-constraints.lxo");
    for frame in [0, 15, 30] {
        at_frame(&mut e, frame);
        let p = |n: &str| world(&e, n).w_axis.truncate();
        let want = matrix(&table["ik2"][frame.to_string()]).w_axis.truncate();
        assert!(
            (p("ik2") - want).length() < 1e-4,
            "f{frame}: {} vs {want}",
            p("ik2")
        );
        let (a, b, c) = (p("ik0"), p("ik1"), p("ik2"));
        assert!(((b - a).length() - 1.00499).abs() < 1e-4, "f{frame}");
        assert!(((c - b).length() - 1.00499).abs() < 1e-4, "f{frame}");
        assert!((b.x - 3.).abs() < 1e-4, "f{frame}: knee {b}");
    }
}

// --- LXA-90: morph maps -----------------------------------------------------

#[test]
fn lxa90_morph_maps_become_relative_morphs() {
    let s = import("morph.lxo").scene;
    let mesh = s
        .items
        .iter()
        .find(|i| i.is_mesh() && i.mesh.attributes.relative_morphs.contains_key("Lift"))
        .expect("a mesh with the Lift morph");
    let lift = &mesh.mesh.attributes.relative_morphs["Lift"];
    assert_eq!(lift.len(), 2);
    for d in lift.values() {
        assert!((*d - DVec3::new(0., 0.5, 0.)).length() < 1e-6, "{d}");
    }
}

// --- the import as a whole ---------------------------------------------------

#[test]
fn every_fixture_imports_and_survives_save_and_reopen() {
    for file in [
        "constraints.lxo",
        "curves-1.lxo",
        "curves-2.lxo",
        "curves-3.lxo",
        "ik-and-stacked-constraints.lxo",
        "morph.lxo",
        "skin.lxo",
        "xfrm-camera-lights.lxo",
    ] {
        let c = import(file);
        let bytes = persistence::encode(&c.scene).unwrap();
        let back = persistence::decode(&bytes).unwrap();
        assert_eq!(back.animation, c.scene.animation, "{file}");
        if file != "morph.lxo" {
            assert!(c.report.summary().contains("key curves imported"), "{file}");
        }
    }
}

#[test]
fn the_report_names_what_was_left_out() {
    let r = import("constraints.lxo").report.summary();
    assert!(r.contains("Unkeyed edits on"), "{r}");
    let r = import("morph.lxo").report.summary();
    assert!(r.contains("morph influences"), "{r}");
}

/// File > Import (a scene's items joining the current one) keeps the
/// arriving animation: curves on the new ids, rig pieces, skins, and the
/// file's frame rate when this scene has no keys yet.
#[test]
fn importing_into_a_scene_keeps_the_animation_and_rig() {
    use preset::{Preset, PresetHeader, PresetKind};
    for file in ["skin.lxo", "constraints.lxo", "xfrm-camera-lights.lxo"] {
        let source = import(file).scene;
        let mut e = Editor::default();
        e.dispatch(Command::LoadPreset {
            preset: Box::new(Preset {
                header: PresetHeader {
                    kind: PresetKind::Assembly,
                    thumbnail: None,
                    source: None,
                },
                scene: source.clone(),
            }),
            group: None,
        })
        .unwrap();
        let a = &e.scene.animation;
        assert_eq!(a.channels.len(), source.animation.channels.len(), "{file}");
        assert_eq!(
            a.rig.pieces.len(),
            source.animation.rig.pieces.len(),
            "{file}"
        );
        assert_eq!(e.scene.time.rate, source.time.rate, "{file}");
        assert_eq!(e.scene.time.end, source.time.end, "{file}");
        for k in a.channels.keys() {
            assert!(
                e.scene.item(k.item).is_some() || a.rig.piece(k.item).is_some(),
                "{file}: {k:?}"
            );
        }
    }
    // The skin still bends the arm at frame 30.
    let all = cases();
    let want = &all["skin.lxo"]["expected"]["mesh_t30"]["points"][16];
    let mut e = Editor::default();
    e.dispatch(Command::LoadPreset {
        preset: Box::new(Preset {
            header: PresetHeader {
                kind: PresetKind::Assembly,
                thumbnail: None,
                source: None,
            },
            scene: import("skin.lxo").scene,
        }),
        group: None,
    })
    .unwrap();
    at_frame(&mut e, 30);
    let arm = id(&e, "Arm");
    let s = e.evaluated(evaluate::Purpose::Final);
    let p = s
        .world_matrix(arm)
        .unwrap()
        .transform_point3(s.item(arm).unwrap().mesh.vertices[16]);
    let w = DVec3::new(
        want[0].as_f64().unwrap(),
        want[1].as_f64().unwrap(),
        want[2].as_f64().unwrap(),
    );
    assert!((p - w).length() < 1e-4, "{p} vs {w}");
}

// --- spec v2: LXA-100..180 ---------------------------------------------------

#[test]
fn lxa102_user_channels_with_labels_types_and_values() {
    use rig::UserType as T;
    let s = import("user-channels-relation-math.lxo").scene;
    let ctrl = s.items.iter().find(|i| i.name == "Ctrl").unwrap().id;
    let user = |n: &str| {
        s.animation
            .rig
            .user_channel(ctrl, n)
            .unwrap_or_else(|| panic!("{n}"))
    };
    for (n, ty) in [
        ("uFloat", T::Float),
        ("uDist", T::Distance),
        ("uAngle", T::Angle),
        ("uPct", T::Percentage),
        ("uInt", T::Integer),
        ("uBool", T::Boolean),
        ("uStr", T::String),
        ("uTime", T::Time),
    ] {
        assert_eq!(user(n).ty, ty, "{n}");
        assert_eq!(user(n).display, format!("{n} label"));
    }
    assert_eq!(user("uFloat").get("uFloat"), Some(0.5));
    assert_eq!(user("uDist").get("uDist"), Some(1.5));
    assert_eq!(user("uInt").get("uInt"), Some(3.));
    assert_eq!((user("uPct").min, user("uPct").max), (Some(0.), Some(1.)));
    // The keyed sliders carry their curves.
    assert!(
        s.animation
            .channels
            .contains_key(&animation::ChannelKey::new(ctrl, "uPct"))
    );
    assert!(
        s.animation
            .channels
            .contains_key(&animation::ChannelKey::new(ctrl, "uAngle"))
    );
}

#[test]
fn lxa111_channel_relationship_drives_the_door_like_modo() {
    let all = cases();
    let mut e = editor("user-channels-relation-math.lxo");
    for row in all["user-channels-relation-math.lxo"]["expected"]["driven"]
        .as_array()
        .unwrap()
    {
        let frame = row[0].as_i64().unwrap();
        let want = row[2].as_f64().unwrap();
        at_frame(&mut e, frame);
        let w = world(&e, "Door");
        let got = w.x_axis.y.atan2(w.x_axis.x);
        assert!((got - want).abs() < 1e-4, "f{frame}: {got} vs {want}");
    }
}

#[test]
fn lxa120_math_operations_without_an_operandi_twin_are_reported() {
    let r = import("user-channels-relation-math.lxo").report.summary();
    assert!(r.contains("math modifiers with operation mod"), "{r}");
    assert!(r.contains("math modifiers with operation avg"), "{r}");
}

/// LXA-131 / Q-LXA-14: both planar chains reach their goals; the three-bar
/// chain is Operandi's chain IK (an iterative solve, within a few mm),
/// so only the ends are compared.
#[test]
fn lxa130_planar_ik_chains_reach_their_goals() {
    let all = cases();
    let table = &all["planar-ik.lxo"]["expected"]["world"];
    let mut e = editor("planar-ik.lxo");
    let kinds: Vec<&str> = e
        .scene
        .animation
        .rig
        .pieces
        .iter()
        .map(|p| p.kind.as_str())
        .collect();
    assert!(
        kinds.contains(&ik::TWO_BONE) && kinds.contains(&ik::CHAIN),
        "{kinds:?}"
    );
    for frame in [0, 15, 30] {
        at_frame(&mut e, frame);
        for (end, tol) in [("b2", 1e-4), ("c3", 5e-3)] {
            let want = matrix(&table[end][frame.to_string()]).w_axis.truncate();
            let got = world(&e, end).w_axis.truncate();
            assert!(
                (got - want).length() < tol,
                "{end} f{frame}: {got} vs {want}"
            );
        }
    }
}

#[test]
fn lxa140_path_constraint_rides_the_curve() {
    let all = cases();
    let mut e = editor("path-blend-shader.lxo");
    for (frame, p) in all["path-blend-shader.lxo"]["expected"]["rider"]
        .as_object()
        .unwrap()
    {
        at_frame(&mut e, frame.parse().unwrap());
        let got = world(&e, "Rider").w_axis.truncate();
        let want = DVec3::new(
            p[0].as_f64().unwrap(),
            p[1].as_f64().unwrap(),
            p[2].as_f64().unwrap(),
        );
        // Operandi measures percent along the curve's length; the file's
        // curve is a little uneven along its straight run (2.0025 at 50 %).
        assert!((got - want).length() < 5e-3, "f{frame}: {got} vs {want}");
    }
    let r = import("path-blend-shader.lxo").report.summary();
    assert!(r.contains("matrix blends"), "{r}");
}
