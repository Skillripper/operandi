//! USD import of animation: the other way from anim_usd.rs. An item's
//! transform time samples, and its camera's or light's attribute time
//! samples, become keys on the channels they move; values that don't
//! change get no keys. UsdSkel joint animation is import_skel.rs.
//!
//! A sample becomes a key with linear slopes (stepped for on/off and
//! choice channels), so the curve passes through the samples the way
//! USD interpolates between them. Time codes become frames by the
//! stage's framesPerSecond / timeCodesPerSecond, and the scene takes the
//! stage's frame rate and, when the file authors one, its time range.
//! Operandi's export writes a sample per frame, so a round trip keeps
//! the motion with a key per frame, not the original keys (the native
//! .opscene stays the lossless home).
use crate::stage::StageData;
use operandi_core::anim_curve::{Curve, Slope};
use operandi_core::animation::{self, AnimChannel, ChannelKey, TRANSFORM_CHANNELS};
use operandi_core::camera_light::{CameraSettings, LightSettings};
use operandi_core::scene::Transform;
use operandi_core::time_tick::{FrameRate, Tick};
use operandi_core::{Result, Scene};

/// An item's state at one time sample.
#[derive(Clone, Debug)]
pub(crate) enum Pose {
    Transform(Transform),
    Camera(CameraSettings),
    Light(LightSettings),
}

/// Item `id`'s poses at time codes `times` (all of one kind).
#[derive(Clone, Debug)]
pub(crate) struct Track {
    pub id: u64,
    pub times: Vec<f64>,
    pub poses: Vec<Pose>,
}

impl Track {
    pub(crate) fn new(id: u64) -> Self {
        Track {
            id,
            times: Vec::new(),
            poses: Vec::new(),
        }
    }
    pub(crate) fn push(&mut self, time: f64, pose: Pose) {
        if time.is_finite() {
            self.times.push(time);
            self.poses.push(pose);
        }
    }
}

/// Whether the stage moves anything: its tracks or a SkelAnimation.
fn animated(stage: &StageData, tracks: &[Track]) -> bool {
    tracks.iter().any(|t| t.times.len() > 1)
        || stage
            .skeletons
            .iter()
            .any(|s| s.anim.as_ref().is_some_and(|a| a.times.len() > 1))
}

/// The scene takes the stage's frame rate when the stage moves anything
/// (before the skeletons' keys are added, which read it).
pub(crate) fn set_rate(scene: &mut Scene, stage: &StageData, tracks: &[Track]) {
    let Some(t) = stage.time else {
        return;
    };
    if animated(stage, tracks) && t.frames_per_second.is_finite() && t.frames_per_second > 0.0 {
        let rate = FrameRate::from_f64(t.frames_per_second);
        if rate.is_valid() {
            scene.time.rate = rate;
        }
    }
}

/// Keys from `tracks`, the scene range, and the scene posed at its
/// current time. Returns report lines.
pub(crate) fn add(scene: &mut Scene, stage: &StageData, tracks: &[Track]) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    if !animated(stage, tracks) {
        return Ok(notes);
    }
    let rate = scene.time.rate;
    let per = stage.frames_per_code();
    let tick = |code: f64| animation::at_frames(rate, code * per);
    // The range the skeletons' keys set, if any (import_skel.rs).
    let skel = stage
        .skeletons
        .iter()
        .any(|s| s.anim.as_ref().is_some_and(|a| a.times.len() > 1));
    let mut range: Option<(Tick, Tick)> = skel.then_some((scene.time.start, scene.time.end));
    let mut items = std::collections::BTreeSet::new();
    for track in tracks.iter().filter(|t| t.times.len() > 1) {
        let keyed = add_track(scene, track, &tick)?;
        if keyed {
            items.insert(track.id);
        }
        for &code in &track.times {
            let t = tick(code);
            range = Some(range.map_or((t, t), |(a, b)| (a.min(t), b.max(t))));
        }
    }
    if let Some(t) = stage.time.filter(|t| t.range && t.end >= t.start) {
        range = Some((tick(t.start), tick(t.end)));
    }
    if let Some((first, last)) = range.filter(|(a, b)| a < b) {
        scene.time.start = first;
        scene.time.end = last;
        if scene.time.current < first || scene.time.current > last {
            scene.time.current = first;
        }
    }
    animation::pose(scene, scene.time.current);
    if !items.is_empty() {
        notes.push(format!(
            "Animation: keys from time samples on {} item(s), at {} fps",
            items.len(),
            rate.label()
        ));
    }
    Ok(notes)
}

/// The channels a pose kind moves: transforms, or the item's other
/// keyable channels (camera lens, light values).
fn channels(scene: &Scene, id: u64, pose: &Pose) -> Vec<String> {
    match pose {
        Pose::Transform(_) => TRANSFORM_CHANNELS.iter().map(|s| s.to_string()).collect(),
        Pose::Camera(_) | Pose::Light(_) => animation::keyable_channels(scene, id)
            .into_iter()
            .filter(|n| ChannelKey::new(id, n).transform_part().is_none())
            .collect(),
    }
}

/// Puts `pose` into item `id` (no undo; the item is restored after).
fn apply(scene: &mut Scene, id: u64, pose: &Pose) {
    let Some(item) = scene.item_mut(id) else {
        return;
    };
    match pose {
        Pose::Transform(t) => item.transform = t.clone(),
        Pose::Camera(c) => item.camera = Some(c.clone()),
        Pose::Light(l) => item.light = Some(l.clone()),
    }
}

/// Keys item `track.id`'s changing channels; true when any were keyed.
fn add_track(scene: &mut Scene, track: &Track, tick: &dyn Fn(f64) -> Tick) -> Result<bool> {
    let id = track.id;
    let Some(item) = scene.item(id).cloned() else {
        return Ok(false);
    };
    let names = channels(scene, id, &track.poses[0]);
    let rest: Vec<Option<f64>> = names
        .iter()
        .map(|n| animation::read(scene, &ChannelKey::new(id, n)))
        .collect();
    // values[sample][channel]
    let mut values: Vec<Vec<Option<f64>>> = Vec::with_capacity(track.poses.len());
    let mut prev: Option<glam::DVec3> = None;
    for pose in &track.poses {
        let pose = match pose {
            Pose::Transform(t) => {
                let mut t = t.clone();
                if let Some(p) = prev {
                    t.rotation = nearest_angles(&t, p);
                }
                prev = Some(t.rotation);
                Pose::Transform(t)
            }
            other => other.clone(),
        };
        apply(scene, id, &pose);
        values.push(
            names
                .iter()
                .map(|n| animation::read(scene, &ChannelKey::new(id, n)))
                .collect(),
        );
    }
    if let Some(slot) = scene.item_mut(id) {
        *slot = item;
    }
    let mut keyed = false;
    for (c, name) in names.iter().enumerate() {
        let mut column: Vec<(Tick, f64)> = track
            .times
            .iter()
            .zip(&values)
            .filter_map(|(&code, v)| Some((tick(code), v[c]?)))
            .filter(|(_, v)| v.is_finite())
            .collect();
        column.dedup_by_key(|(t, _)| *t);
        if column.len() < 2 {
            continue;
        }
        let v0 = column[0].1;
        if column.iter().all(|(_, v)| !differs(*v, v0)) {
            continue;
        }
        let key = ChannelKey::new(id, name);
        let stepped = animation::is_stepped(scene, &key);
        let mut curve = Curve {
            stepped,
            ..Curve::default()
        };
        for &(t, v) in &column {
            let mut k = animation::new_key(t, v);
            k.slope_in = if stepped { Slope::Flat } else { Slope::Linear };
            k.slope_out = if stepped {
                Slope::Stepped
            } else {
                Slope::Linear
            };
            curve.keys.push(k);
        }
        let rest = rest[c].unwrap_or(v0);
        scene
            .animation
            .channels
            .insert(key, AnimChannel { curve, rest });
        keyed = true;
    }
    Ok(keyed)
}

/// Angles decomposed from matrices can jump between samples: by whole
/// turns, or to the other angle triple that makes the same rotation
/// (first and last axes half a turn on, the middle one mirrored). Of
/// those, the one nearest the sample before (`prev`, radians).
fn nearest_angles(t: &Transform, prev: glam::DVec3) -> glam::DVec3 {
    use std::f64::consts::{PI, TAU};
    let near = |v: f64, p: f64| v + ((p - v) / TAU).round() * TAU;
    let [a, b, c] = t.rotation_order().axes();
    let r = t.rotation;
    let mut other = r;
    other[a] += PI;
    other[b] = PI - r[b];
    other[c] += PI;
    [r, other]
        .into_iter()
        .map(|mut v| {
            for i in 0..3 {
                v[i] = near(v[i], prev[i]);
            }
            v
        })
        .min_by(|x, y| {
            x.distance_squared(prev)
                .total_cmp(&y.distance_squared(prev))
        })
        .unwrap_or(r)
}

/// Values a key would tell apart (a hair above float noise).
fn differs(a: f64, b: f64) -> bool {
    (a - b).abs() > 1e-9 * a.abs().max(b.abs()).max(1.0)
}
