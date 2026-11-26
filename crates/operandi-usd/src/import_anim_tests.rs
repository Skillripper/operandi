//! Animation import (import_anim.rs) on stages described by hand, without
//! OpenUSD. The round trip through real files is in anim_usd_tests.rs
//! (`--features usd`).
use crate::import::{Assets, build};
use crate::stage::{
    AttributeSamples, CameraData, LightData, MeshData, Prim, StageData, StageTiming, Trs,
    XformSamples,
};
use glam::{DMat4, DVec3};
use operandi_core::anim_curve::Slope;
use operandi_core::animation::{self, ChannelKey};
use operandi_core::interchange::{Imported, UsdImportOptions};
use operandi_core::time_tick::FrameRate;
use operandi_core::{Item, Result, Scene};
use std::path::Path;

struct NoAssets;
impl Assets for NoAssets {
    fn image_size(&self, _: &Path) -> Option<(u32, u32)> {
        None
    }
    fn copy_packaged(&self, _: &str, _: &Path) -> Result<()> {
        Err("no packages here".into())
    }
}

fn stage(fps: f64, codes: f64) -> StageData {
    StageData {
        up_axis: "Y".into(),
        meters_per_unit: 1.0,
        time: Some(StageTiming {
            time_codes_per_second: codes,
            frames_per_second: fps,
            start: 0.0,
            end: 0.0,
            range: false,
        }),
        ..Default::default()
    }
}

fn prim(s: &mut StageData, name: &str, ty: &str, local: DMat4) -> usize {
    s.prims.push(Prim {
        path: format!("/{name}"),
        name: name.into(),
        ty: ty.into(),
        parent: -1,
        visible: true,
        local: Some(local.to_cols_array()),
        material: Some(-1),
        ..Default::default()
    });
    s.prims.len() - 1
}

/// A one-quad Mesh prim (an import needs polygons).
fn quad(s: &mut StageData, name: &str) -> usize {
    let points = [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]];
    let data = MeshData {
        points: s.push_points(&points),
        counts: s.push_i32(&[4]),
        indices: s.push_i32(&[0, 1, 2, 3]),
        orientation: "rightHanded".into(),
        scheme: "none".into(),
        boundary: "edgeAndCorner".into(),
        fvli: "cornersPlus1".into(),
        ..Default::default()
    };
    s.meshes.push(data);
    let p = prim(s, name, "Mesh", DMat4::IDENTITY);
    s.prims[p].mesh = Some(s.meshes.len() - 1);
    p
}

fn trs(t: [f64; 3], r: [f64; 3]) -> Trs {
    Trs {
        t,
        r,
        s: [1.; 3],
        order: "XYZ".into(),
    }
}

fn matrix(t: &Trs) -> DMat4 {
    DMat4::from_translation(DVec3::from_array(t.t))
        * DMat4::from_rotation_z(t.r[2].to_radians())
        * DMat4::from_rotation_y(t.r[1].to_radians())
        * DMat4::from_rotation_x(t.r[0].to_radians())
}

/// Plain TRS samples at `times` for prim `p` (its own transform is the
/// first).
fn animate(s: &mut StageData, p: usize, times: &[f64], samples: &[Trs]) {
    s.prims[p].local = Some(matrix(&samples[0]).to_cols_array());
    s.prims[p].trs = Some(samples[0].clone());
    s.prims[p].xform_samples = Some(XformSamples {
        times: times.to_vec(),
        local: samples.iter().map(|t| matrix(t).to_cols_array()).collect(),
        trs: samples.iter().cloned().map(Some).collect(),
    });
}

fn run(s: &StageData) -> Imported {
    build(
        s,
        Path::new("/nowhere/scene.usdc"),
        &UsdImportOptions::default(),
        &NoAssets,
    )
    .unwrap()
}

fn item<'a>(scene: &'a Scene, name: &str) -> &'a Item {
    scene.items.iter().find(|i| i.name == name).unwrap()
}

fn value_at(scene: &Scene, id: u64, channel: &str, frame: f64) -> f64 {
    let posed = animation::evaluate_at(scene, animation::at_frames(scene.time.rate, frame));
    animation::read(&posed, &ChannelKey::new(id, channel)).unwrap()
}

/// A moving mesh: its changing channels get a key per sample, linear;
/// the rest get none. The scene takes the stage's rate and the samples'
/// range, and is posed at its current time.
#[test]
fn transform_samples_become_keys_on_the_channels_that_change() {
    let mut s = stage(30.0, 30.0);
    let p = quad(&mut s, "Ball");
    animate(
        &mut s,
        p,
        &[1.0, 11.0, 21.0],
        &[
            trs([0., 0., 0.], [0., 0., 0.]),
            trs([0., 2., 0.], [0., 45., 0.]),
            trs([0., 0., 0.], [0., 90., 0.]),
        ],
    );
    let imported = run(&s);
    let scene = &imported.scene;
    let ball = item(scene, "Ball").id;
    assert_eq!(scene.time.rate, FrameRate::FPS_30);
    assert_eq!(scene.time.start, animation::at_frames(scene.time.rate, 1.0));
    assert_eq!(scene.time.end, animation::at_frames(scene.time.rate, 21.0));
    let keyed: Vec<&str> = scene
        .animation
        .item_channels(ball)
        .map(|(k, _)| k.channel.as_str())
        .collect();
    assert_eq!(keyed, ["pos.Y", "rot.Y"]);
    let y = &scene.animation.channels[&ChannelKey::new(ball, "pos.Y")];
    assert_eq!(y.curve.keys.len(), 3);
    assert!(
        y.curve
            .keys
            .iter()
            .all(|k| k.slope_in == Slope::Linear && k.slope_out == Slope::Linear)
    );
    assert!((value_at(scene, ball, "pos.Y", 11.0) - 2.0).abs() < 1e-9);
    // Linear between samples, as USD interpolates.
    assert!((value_at(scene, ball, "pos.Y", 6.0) - 1.0).abs() < 1e-9);
    assert!((value_at(scene, ball, "rot.Y", 21.0) - 90f64.to_radians()).abs() < 1e-9);
    assert!(
        imported
            .notes
            .iter()
            .any(|n| n.starts_with("Animation: keys from time samples on 1 item"))
    );
}

/// Time codes are frames only when the stage says so: 48 codes a second
/// at 24 fps puts code 48 on frame 24. An authored range wins.
#[test]
fn time_codes_become_frames_by_the_stage_timing() {
    let mut s = stage(24.0, 48.0);
    s.time.as_mut().unwrap().range = true;
    s.time.as_mut().unwrap().start = 0.0;
    s.time.as_mut().unwrap().end = 96.0;
    let p = quad(&mut s, "Box");
    animate(
        &mut s,
        p,
        &[0.0, 48.0],
        &[trs([0.; 3], [0.; 3]), trs([5., 0., 0.], [0.; 3])],
    );
    let scene = run(&s).scene;
    let id = item(&scene, "Box").id;
    let x = &scene.animation.channels[&ChannelKey::new(id, "pos.X")];
    let rate = scene.time.rate;
    assert_eq!(rate, FrameRate::FPS_24);
    assert_eq!(x.curve.keys[1].time, animation::at_frames(rate, 24.0));
    assert_eq!(scene.time.end, animation::at_frames(rate, 48.0));
}

/// Angles decomposed from matrices take the turn nearest the sample
/// before, so a spin doesn't jump back a whole turn.
#[test]
fn a_matrix_spin_keeps_turning_the_same_way() {
    let mut s = stage(24.0, 24.0);
    let p = quad(&mut s, "Spin");
    let angles = [0.0, 120.0, 240.0, 360.0];
    let local: Vec<[f64; 16]> = angles
        .iter()
        .map(|a: &f64| DMat4::from_rotation_y(a.to_radians()).to_cols_array())
        .collect();
    s.prims[p].xform_samples = Some(XformSamples {
        times: vec![0., 1., 2., 3.],
        local,
        trs: vec![None; 4],
    });
    let scene = run(&s).scene;
    let id = item(&scene, "Spin").id;
    let ch = &scene.animation.channels[&ChannelKey::new(id, "rot.Y")];
    let got: Vec<f64> = ch.curve.keys.iter().map(|k| k.value.to_degrees()).collect();
    for w in got.windows(2) {
        assert!((w[1] - w[0] - 120.0).abs() < 1e-6, "{got:?}");
    }
}

/// A camera's focal length and a light's intensity over time become keys
/// on the item's own channels.
#[test]
fn camera_and_light_attribute_samples_become_keys() {
    let mut s = stage(24.0, 24.0);
    quad(&mut s, "Floor");
    let cam = prim(&mut s, "Cam", "Camera", DMat4::IDENTITY);
    let lens = |f: f64| CameraData {
        projection: Some("perspective".into()),
        focal_length: Some(f),
        ..Default::default()
    };
    s.prims[cam].camera = Some(lens(500.0));
    s.prims[cam].attribute_samples = Some(AttributeSamples {
        times: vec![0., 10.],
        values: vec![
            serde_json::json!({"projection": "perspective", "focalLength": 500.0}),
            serde_json::json!({"projection": "perspective", "focalLength": 1000.0}),
        ],
    });
    let lamp = prim(&mut s, "Lamp", "SphereLight", DMat4::IDENTITY);
    s.prims[lamp].light = Some(LightData {
        intensity: Some(1.0),
        ..Default::default()
    });
    s.prims[lamp].attribute_samples = Some(AttributeSamples {
        times: vec![0., 10.],
        values: vec![
            serde_json::json!({"inputs:intensity": 1.0}),
            serde_json::json!({"inputs:intensity": 4.0}),
        ],
    });
    let scene = run(&s).scene;
    let c = item(&scene, "Cam").id;
    let focal = &scene.animation.channels[&ChannelKey::new(c, "focalLength")];
    assert_eq!(focal.curve.keys.len(), 2);
    let (f0, f1) = (focal.curve.keys[0].value, focal.curve.keys[1].value);
    assert!((f1 / f0 - 2.0).abs() < 1e-9, "{f0} {f1}");
    let still = crate::import_lights::camera_settings(&lens(500.0), 1.0);
    assert!((f0 - still.focal_length).abs() < 1e-12);
    let l = item(&scene, "Lamp").id;
    let intensity = &scene.animation.channels[&ChannelKey::new(l, "intensity")];
    let (a, b) = (intensity.curve.keys[0].value, intensity.curve.keys[1].value);
    assert!((b / a - 4.0).abs() < 1e-9, "{a} {b}");
    // Nothing else of theirs is keyed.
    assert_eq!(scene.animation.item_channels(c).count(), 1);
    assert_eq!(scene.animation.item_channels(l).count(), 1);
}

/// A file with no time samples leaves the scene's time alone.
#[test]
fn a_still_file_keeps_the_default_time() {
    let mut s = stage(30.0, 30.0);
    quad(&mut s, "Still");
    let scene = run(&s).scene;
    assert!(scene.animation.channels.is_empty());
    assert_eq!(scene.time, operandi_core::scene_time::SceneTime::default());
}
