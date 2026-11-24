//! BEHAVIOR-ACS-FEATURES, first milestone: a Biped rig's guides (ACS-30,
//! 31, 34), contexts (ACS-01..03), the bind skeleton built from the guides
//! (ACS-32, 40) and binding meshes to it (ACS-41), through the
//! `operandi.acs.*` commands.
use glam::DVec3;
use operandi_core::character_rig::{self, RigContext};
use operandi_core::*;
use serde_json::Value;

fn ok(e: &mut Editor, text: &str) {
    if let Err(err) = command_line::run_line(e, text).expect("headless") {
        panic!("{text}: {err}");
    }
}

fn err(e: &mut Editor, text: &str) -> String {
    command_line::run_line(e, text)
        .expect("headless")
        .expect_err("should be refused")
}

fn query(e: &mut Editor) -> Value {
    let text = command_line::run_line(e, "operandi.acs.query")
        .expect("headless")
        .unwrap()
        .expect("a query answers");
    serde_json::from_str(&text).unwrap()
}

fn pos(e: &Editor, id: u64) -> DVec3 {
    e.scene.world_matrix(id).unwrap().w_axis.truncate()
}

fn near(a: DVec3, b: DVec3) {
    assert!(a.distance(b) < 1e-6, "{a} vs {b}");
}

fn rig(e: &Editor) -> character_rig::CharacterRig {
    e.scene.character_rigs().last().expect("a rig").clone()
}

fn guide(e: &Editor, role: &str) -> u64 {
    rig(e)
        .guide(role)
        .unwrap_or_else(|| panic!("no guide {role}"))
}

fn joint(e: &Editor, role: &str) -> u64 {
    rig(e)
        .joint(role)
        .unwrap_or_else(|| panic!("no joint {role}"))
}

fn biped() -> Editor {
    let mut e = Editor::default();
    ok(&mut e, "operandi.acs.rigAdd");
    e
}

/// A box of `size` with its bottom centre at `base`.
fn body(e: &mut Editor, base: DVec3, size: DVec3) -> u64 {
    e.scene.new_mesh().unwrap();
    let mesh = e.scene.items.last().unwrap().id;
    e.scene.item_mut(mesh).unwrap().mesh =
        primitive::segmented_cube(base + DVec3::new(0., size.y / 2., 0.), size, [4, 16, 2])
            .unwrap()
            .into();
    mesh
}

#[test]
fn acs10_new_biped_rig_is_one_step_with_mirrored_guides() {
    let mut e = biped();
    let r = rig(&e);
    assert_eq!(r.preset, "biped");
    assert_eq!(r.context, RigContext::Guide);
    assert!(r.symmetry);
    // 5 centre guides and 10 a side.
    assert_eq!(r.guides.len(), 25);
    assert_eq!(e.scene.selection.item, Some(r.root));
    for p in &r.guides {
        let item = e.scene.item(p.item).unwrap();
        assert_eq!(
            item.locator,
            Some(locator::LocatorKind::Guide),
            "{}",
            p.role
        );
        let twin = r.guide(&character_rig::twin_role(&p.role)).unwrap();
        let (a, b) = (pos(&e, p.item), pos(&e, twin));
        near(a, DVec3::new(-b.x, b.y, b.z));
    }
    // Guides stand at the template's proportions of 1.8 m.
    near(pos(&e, guide(&e, "head.C.top")), DVec3::new(0., 1.8, 0.018));
    assert!(pos(&e, guide(&e, "arm.L.hand")).x > 0.8);
    // One undo step takes it all away.
    e.dispatch(Command::Undo).unwrap();
    assert!(e.scene.character_rigs().is_empty());
    assert!(e.scene.items.iter().all(|i| i.locator.is_none()));
    e.dispatch(Command::Redo).unwrap();
    assert_eq!(rig(&e).guides.len(), 25);
}

#[test]
fn acs31_moving_a_guide_moves_its_twin_mirrored() {
    let mut e = biped();
    let knee_l = guide(&e, "leg.L.knee");
    let knee_r = guide(&e, "leg.R.knee");
    let ankle_l = guide(&e, "leg.L.ankle");
    let ankle_before = pos(&e, ankle_l);
    ok(
        &mut e,
        "operandi.acs.guideMove guide:leg.L.knee position:0.2,0.6,0.05",
    );
    near(pos(&e, knee_l), DVec3::new(0.2, 0.6, 0.05));
    near(pos(&e, knee_r), DVec3::new(-0.2, 0.6, 0.05));
    // The child guide moved with its parent, and its twin mirrored it.
    let ankle = pos(&e, ankle_l);
    assert!(ankle.distance(ankle_before) > 0.01);
    let ankle_r = pos(&e, guide(&e, "leg.R.ankle"));
    near(ankle_r, DVec3::new(-ankle.x, ankle.y, ankle.z));
    // The right side leads when it's the one moved.
    ok(
        &mut e,
        &format!("operandi.acs.guideMove guide:{knee_r} position:-0.3,0.6,0.05"),
    );
    near(pos(&e, knee_l), DVec3::new(0.3, 0.6, 0.05));
    // A centre guide stays on the centre plane.
    ok(
        &mut e,
        "operandi.acs.guideMove guide:spine.C.chest position:0.1,1.3,0.02",
    );
    near(
        pos(&e, guide(&e, "spine.C.chest")),
        DVec3::new(0., 1.3, 0.02),
    );
    // One step each.
    e.dispatch(Command::Undo).unwrap();
    e.dispatch(Command::Undo).unwrap();
    near(pos(&e, knee_l), DVec3::new(0.2, 0.6, 0.05));
}

#[test]
fn acs31_any_transform_edit_keeps_symmetry_and_off_lets_go() {
    let mut e = biped();
    let wrist_l = guide(&e, "arm.L.wrist");
    let wrist_r = guide(&e, "arm.R.wrist");
    // An ordinary channel edit, not the guide command.
    ok(
        &mut e,
        &format!("transform.channel name:pos.Y value:-0.2 item:{wrist_l}"),
    );
    let l = pos(&e, wrist_l);
    near(pos(&e, wrist_r), DVec3::new(-l.x, l.y, l.z));
    ok(&mut e, "operandi.acs.options symmetry:false");
    let r_before = pos(&e, wrist_r);
    ok(
        &mut e,
        "operandi.acs.guideMove guide:arm.L.wrist position:0.7,1.0,0",
    );
    near(pos(&e, wrist_r), r_before);
    // Mirror Guides copies one side over on demand.
    ok(&mut e, "operandi.acs.guideMirror from:left");
    near(pos(&e, wrist_r), DVec3::new(-0.7, 1.0, 0.));
}

#[test]
fn acs01_contexts_show_guides_or_skeleton_and_are_not_undo_steps() {
    let mut e = biped();
    ok(&mut e, "operandi.acs.build");
    let hips = joint(&e, "spine.C.hips");
    let knee = guide(&e, "leg.L.knee");
    // Guide: guides show, joints don't.
    assert!(e.scene.drawn_in_viewport(knee, true));
    assert!(!e.scene.drawn_in_viewport(hips, true));
    let steps = e.history_mark();
    ok(&mut e, "operandi.acs.context context:form");
    assert_eq!(e.history_mark(), steps, "a context switch is no undo step");
    assert!(!e.scene.drawn_in_viewport(knee, true));
    assert!(e.scene.drawn_in_viewport(hips, true));
    assert!(e.scene.animation.setup, "Form is Setup mode");
    ok(&mut e, "operandi.acs.context context:animate");
    assert!(
        !e.scene.animation.setup,
        "Animate shows the animated values"
    );
    // Undo of the build keeps the context the rig is in now.
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(rig(&e).context, RigContext::Animate);
    // Saved with the scene (ACS-02).
    let back = persistence::decode(&persistence::encode(&e.scene).unwrap()).unwrap();
    assert_eq!(back.character_rigs()[0].context, RigContext::Animate);
    assert!(
        err(&mut e, "operandi.acs.context context:pose").contains("context is"),
        "unknown contexts are refused"
    );
}

#[test]
fn acs32_build_makes_the_bind_skeleton_on_the_guides() {
    let mut e = biped();
    ok(&mut e, "operandi.acs.build");
    let r = rig(&e);
    // Hips, 3 vertebrae, chest, neck, head, head end; 5 joints an arm and
    // 5 a leg each side.
    assert_eq!(r.joints.len(), 28);
    let group = r.skeleton_group.unwrap();
    assert_eq!(e.scene.item(group).unwrap().parent, Some(r.root));
    let hips = joint(&e, "spine.C.hips");
    assert_eq!(e.scene.item(hips).unwrap().parent, Some(group));
    assert_eq!(e.scene.item(hips).unwrap().name, "Hips");
    for (j, g) in [
        ("spine.C.hips", "spine.C.hips"),
        ("leg.L.knee", "leg.L.knee"),
        ("arm.R.wrist", "arm.R.wrist"),
        ("head.C.top", "head.C.top"),
    ] {
        near(pos(&e, joint(&e, j)), pos(&e, guide(&e, g)));
    }
    // Chains: thigh → shin → foot; each joint's first child is its bone.
    let thigh = joint(&e, "leg.L.thigh");
    let shin = joint(&e, "leg.L.knee");
    assert_eq!(e.scene.item(thigh).unwrap().name, "Thigh_L");
    assert_eq!(e.scene.item(shin).unwrap().name, "Shin_L");
    assert_eq!(skeleton::first_child(&e.scene, thigh), Some(shin));
    assert_eq!(e.scene.item(thigh).unwrap().parent, Some(hips));
    // SK-16: the thigh's Z points down its bone.
    let w = e.scene.world_matrix(thigh).unwrap();
    let down = (pos(&e, shin) - pos(&e, thigh)).normalize();
    assert!(w.z_axis.truncate().normalize().dot(down) > 0.999);
    // Rebuilding after a guide moves keeps every joint's id.
    let ids: Vec<u64> = r.joints.iter().map(|p| p.item).collect();
    ok(
        &mut e,
        "operandi.acs.guideMove guide:leg.L.knee position:0.15,0.45,0.05",
    );
    ok(&mut e, "operandi.acs.build");
    let again: Vec<u64> = rig(&e).joints.iter().map(|p| p.item).collect();
    assert_eq!(ids, again);
    near(pos(&e, shin), DVec3::new(0.15, 0.45, 0.05));
    near(
        pos(&e, joint(&e, "leg.R.knee")),
        DVec3::new(-0.15, 0.45, 0.05),
    );
    // A build with nothing to change is no step.
    let steps = e.history_mark();
    ok(&mut e, "operandi.acs.build");
    assert_eq!(e.history_mark(), steps);
}

#[test]
fn acs22_acs25_options_add_and_remove_serial_joints() {
    let mut e = biped();
    ok(&mut e, "operandi.acs.build");
    ok(&mut e, "operandi.acs.options spine:5 forearmTwist:2");
    let r = rig(&e);
    assert_eq!(r.joints.len(), 28 + 2 + 4);
    assert!(r.joint("spine.C.spine5").is_some());
    let t1 = joint(&e, "arm.L.elbowTwist1");
    assert_eq!(e.scene.item(t1).unwrap().name, "ForearmTwist1_L");
    // Twist joints sit in line between the elbow and the wrist.
    let (el, wr) = (
        pos(&e, joint(&e, "arm.L.elbow")),
        pos(&e, joint(&e, "arm.L.wrist")),
    );
    near(pos(&e, t1), el.lerp(wr, 1. / 3.));
    ok(&mut e, "operandi.acs.options spine:2 forearmTwist:0");
    assert_eq!(rig(&e).joints.len(), 27);
    assert!(e.scene.items.iter().all(|i| i.name != "Spine3"));
    assert!(
        err(&mut e, "operandi.acs.options spine:9").contains("spine is 1 to 8"),
        "out of range"
    );
    // The Rig tab's buttons step a count and toggle symmetry.
    ok(&mut e, "operandi.acs.options spine:more");
    assert_eq!(rig(&e).options.spine, 3);
    ok(&mut e, "operandi.acs.options spine:fewer");
    assert_eq!(rig(&e).options.spine, 2);
    ok(&mut e, "operandi.acs.options symmetry:toggle");
    assert!(!rig(&e).symmetry);
    ok(&mut e, "operandi.acs.options symmetry:toggle");
    assert!(rig(&e).symmetry);
}

#[test]
fn acs34_fit_guides_to_a_mesh() {
    let mut e = biped();
    let mesh = body(&mut e, DVec3::new(2., 0.1, 1.), DVec3::new(0.5, 1.6, 0.3));
    let root = rig(&e).root;
    ok(&mut e, &format!("operandi.acs.guideFit mesh:{mesh}"));
    let r = rig(&e);
    assert!((r.reference_size - 1.6).abs() < 1e-9);
    near(pos(&e, root), DVec3::new(2., 0.1, 1.));
    // Embedded: the spine is centred front to back, the head's top is on
    // the mesh's top.
    let hips = pos(&e, guide(&e, "spine.C.hips"));
    assert!((hips.z - 1.).abs() < 1e-6, "{hips}");
    assert!((hips.x - 2.).abs() < 1e-6, "{hips}");
    let top = pos(&e, guide(&e, "head.C.top"));
    assert!((top.y - 1.7).abs() < 1e-6, "{top}");
    // Symmetry holds about the rig's centre.
    let l = pos(&e, guide(&e, "leg.L.knee"));
    let rk = pos(&e, guide(&e, "leg.R.knee"));
    near(rk, DVec3::new(4. - l.x, l.y, l.z));
    // Without embed the guides keep their template places.
    ok(
        &mut e,
        &format!("operandi.acs.guideFit mesh:{mesh} embed:false"),
    );
    let hips = pos(&e, guide(&e, "spine.C.hips"));
    near(hips, DVec3::new(2., 0.1 + 0.53 * 1.6, 1.));
    // Nothing selected and no mesh: refused, scene unchanged.
    e.scene.select_item(None);
    assert!(err(&mut e, "operandi.acs.guideFit").contains("Select the character's mesh"));
}

#[test]
fn acs41_bind_meshes_to_the_skeleton() {
    let mut e = biped();
    let mesh = body(&mut e, DVec3::ZERO, DVec3::new(1.0, 1.8, 0.3));
    e.scene.select_item(Some(mesh));
    assert!(
        err(&mut e, "operandi.acs.bind type:distance").contains("Build the skeleton"),
        "no skeleton yet"
    );
    ok(&mut e, "operandi.acs.build");
    e.scene.select_item(Some(mesh));
    ok(
        &mut e,
        &format!("operandi.acs.bind items:{mesh} type:distance"),
    );
    let r = rig(&e);
    assert_eq!(r.context, RigContext::Form);
    assert_eq!(character_rig::bound_meshes(&e.scene, &r), vec![mesh]);
    let (_, skin) = skin_commands::skin_op(&e.scene, mesh).unwrap();
    assert_eq!(skin.joints.len(), 28);
    // Lowering a count would delete bound joints: refused.
    assert!(err(&mut e, "operandi.acs.options spine:1").contains("unbind it"));
    // Moving the guides and rebuilding keeps the bind; the rest shape stays.
    let before = e.scene.item(mesh).unwrap().mesh.vertices.clone();
    ok(
        &mut e,
        "operandi.acs.guideMove guide:arm.L.elbow position:0.5,1.4,0",
    );
    ok(&mut e, "operandi.acs.build");
    assert_eq!(e.scene.item(mesh).unwrap().mesh.vertices, before);
    assert_eq!(character_rig::bound_meshes(&e.scene, &rig(&e)), vec![mesh]);
    // Saves and reopens.
    let back = persistence::decode(&persistence::encode(&e.scene).unwrap()).unwrap();
    assert_eq!(back.character_rigs(), e.scene.character_rigs());
}

#[test]
fn deleting_parts_or_the_root_updates_the_rig() {
    let mut e = biped();
    ok(&mut e, "operandi.acs.build");
    let toe = guide(&e, "leg.L.toe");
    e.scene.select_item(Some(toe));
    ok(&mut e, &format!("item.delete item:{toe}"));
    assert!(rig(&e).guide("leg.L.toe").is_none());
    assert!(err(&mut e, "operandi.acs.build").contains("Guide Toe_L is missing"));
    ok(&mut e, "operandi.acs.guideReset");
    assert_eq!(rig(&e).guides.len(), 25);
    ok(&mut e, "operandi.acs.build");
    let root = rig(&e).root;
    e.scene.select_item(Some(root));
    ok(&mut e, "item.delete");
    assert!(e.scene.character_rigs().is_empty());
}

#[test]
fn acs00_query_reads_the_rig() {
    let mut e = biped();
    ok(&mut e, "operandi.acs.build");
    let q = query(&mut e);
    let r = &q["rigs"][0];
    assert_eq!(r["preset"], "biped");
    assert_eq!(r["context"], "guide");
    assert_eq!(r["options"]["spine"], 3);
    assert_eq!(r["guides"].as_array().unwrap().len(), 25);
    assert_eq!(r["joints"].as_array().unwrap().len(), 28);
    assert_eq!(r["modules"].as_array().unwrap().len(), 6);
    let knee = r["guides"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["role"] == "leg.L.knee")
        .unwrap();
    assert_eq!(knee["side"], "L");
    assert_eq!(knee["name"], "Guide Knee_L");
}

#[test]
fn acs_rig_saves_as_native_63_and_older_versions_refuse_it() {
    let e = biped();
    let bytes = persistence::encode(&e.scene).unwrap();
    let doc: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(doc["version"], 63);
    let back = persistence::decode(&bytes).unwrap();
    assert_eq!(back.character_rigs(), e.scene.character_rigs());
    let mut old = doc.clone();
    old["version"] = 62.into();
    let err = persistence::decode(&serde_json::to_vec(&old).unwrap()).unwrap_err();
    assert!(err.contains("version 63"), "{err}");
}
