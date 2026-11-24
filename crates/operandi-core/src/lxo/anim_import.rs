//! LXO import: keyed channels and scene time (spec
//! lxo-import-anim-rig-2026-11-24, LXA-01..05, LXA-30..33, LXA-40).
//!
//! A channel is keyed when its record in the `scene` action layer names an
//! envelope (LXA-02); its curve becomes an Operandi key curve on the
//! matching Operandi channel, with the channel's rest value from the
//! `setup` layer (else the scene-layer record's value). Keyed transforms
//! sit on the transform items phase 1 reads and are mapped onto the item
//! they move (LXA-30). Edit-layer values on keyed channels are dropped
//! (LXA-03, Q-LXA-01). The `scene` item's frame rate, ranges and current
//! time become Operandi's (LXA-40).
use super::actions::{Layer, Record};
use super::envelope::{self, Interpolation};
use super::phase1::{Extras, RawItem, Value, number, scene_kind, text};
use super::rig_import::Pieces;
use super::{Report, Result};
use crate::Scene;
use crate::animation::{self, AnimChannel, ChannelKey, TimeDisplay};
use crate::time_tick::{FrameRate, Tick};
use std::collections::BTreeMap;

fn oid(id: u32) -> u64 {
    u64::from(id) + 1
}

/// A rig piece's Operandi channel for one of its Modo item's channels.
fn piece_channel(kind: &str, modo: &str) -> Option<String> {
    if modo == "enable" {
        return Some("enable".into());
    }
    let split = |prefix: &str| modo.strip_prefix(prefix).map(str::to_owned);
    match kind {
        "dynamicParent" => {
            if modo == "parent" {
                return Some("parent".into());
            }
            for (from, to) in [
                ("offsetPos.", "offset.pos."),
                ("offsetRot.", "offset.rot."),
                ("offsetScl.", "offset.scl."),
            ] {
                if let Some(a) = split(from) {
                    return Some(format!("{to}{a}"));
                }
            }
            None
        }
        "position" | "rotation" | "scale" | "direction" => {
            if let Some(a) = split("offset.") {
                let group = match kind {
                    "position" => "pos",
                    "scale" => "scl",
                    _ => "rot",
                };
                return Some(format!("offset.{group}.{a}"));
            }
            match (kind, modo) {
                ("direction", "roll") => Some("roll".into()),
                ("direction", "axisNeg") => Some("negative".into()),
                ("direction", m) if m.starts_with("up.") => Some(m.into()),
                _ => None,
            }
        }
        crate::ik::TWO_BONE | crate::ik::CHAIN => (modo == "blend").then(|| "blend".into()),
        "path" | "pathPosition" | "pathNormal" => match modo {
            "percent" => Some("percent".into()),
            "offset" => Some("pathOffset".into()),
            "roll" => Some("roll".into()),
            _ => None,
        },
        _ => None,
    }
}

/// The Operandi channel a Modo item's channel maps to (LXA-30/32): a core
/// transform item's `pos.X` onto the item it moves, a camera's, light's or
/// light material's own channels, or a rig piece's settings. `None` when
/// there is no equivalent (pivot-type transforms included, Q-LXA-03).
pub fn target(
    items: &BTreeMap<u32, RawItem>,
    pieces: Option<&Pieces>,
    modo: u32,
    channel: &str,
) -> Option<ChannelKey> {
    if let Some((id, kind)) = pieces.and_then(|p| p.get(&modo)) {
        return piece_channel(kind, channel).map(|c| ChannelKey::new(*id, &c));
    }
    let item = items.get(&modo)?;
    match item.kind.as_str() {
        "translation" | "rotation" | "scale" => {
            if text(item, "type").is_some_and(|t| t != "core") {
                return None;
            }
            let owner = item.link("xfrmCore")?;
            if !items.get(&owner).is_some_and(|o| scene_kind(&o.kind)) {
                return None;
            }
            animation::transform_part(channel)?;
            Some(ChannelKey::new(oid(owner), channel))
        }
        "lightMaterial" => {
            let light = item.link("parent")?;
            let c = super::cameras_lights::channel("lightMaterial", channel)?;
            Some(ChannelKey::new(oid(light), &c))
        }
        // User channels (LXA-101) key and link under their own names.
        _ if item
            .user
            .iter()
            .any(|u| super::user_channels::names(u).iter().any(|n| n == channel)) =>
        {
            scene_kind(&item.kind).then(|| ChannelKey::new(oid(modo), channel))
        }
        kind if scene_kind(kind) => {
            let c = super::cameras_lights::channel(kind, channel)?;
            Some(ChannelKey::new(oid(modo), &c))
        }
        _ => None,
    }
}

fn layer_number(layer: &Layer, item: u32, channel: &str) -> Option<f64> {
    match layer.records.get(&item)?.get(channel)?.value.as_ref()? {
        Value::Number(n) => Some(*n),
        Value::Text(_) => None,
    }
}

fn layer_text<'a>(layer: &'a Layer, item: u32, channel: &str) -> Option<&'a str> {
    match layer.records.get(&item)?.get(channel)?.value.as_ref()? {
        Value::Text(t) => Some(t),
        Value::Number(_) => None,
    }
}

/// LXA-40: the scene item's time channels onto Operandi's time.
fn scene_time(scene: &mut Scene, items: &BTreeMap<u32, RawItem>, extras: &Extras) {
    let Some(s) = items.values().find(|i| i.kind == "scene") else {
        return;
    };
    let a = &extras.actions;
    let num = |k: &str| {
        number(s, k)
            .or_else(|| layer_number(&a.setup, s.id, k))
            .or_else(|| layer_number(&a.scene, s.id, k))
    };
    let mut time = scene.time;
    if let Some(fps) = num("fps").filter(|f| crate::scene_time::FPS_RANGE.contains(f)) {
        time.rate = FrameRate::from_f64(fps);
    }
    let at = |k: &str| num(k).map(|v| envelope::key_time(v, time.rate));
    if let Some(end) = at("sceneE") {
        time.start = at("sceneS").unwrap_or(Tick::ZERO);
        time.end = end;
    }
    if let Some(t) = at("time") {
        time.current = t;
    }
    if time.validate().is_ok() {
        scene.time = time;
    }
    if let (Some(start), Some(end)) = (at("currentS"), at("currentE"))
        && (start, end) != (scene.time.start, scene.time.end)
        && start <= end
    {
        scene.animation.settings.current_range = Some((start, end));
    }
    let display = text(s, "timeSys").or_else(|| layer_text(&a.setup, s.id, "timeSys"));
    if let Some(d) = display {
        scene.animation.settings.display = match d {
            "seconds" => TimeDisplay::Seconds,
            "smpte" => TimeDisplay::Smpte,
            "filmcode" => TimeDisplay::FilmCode,
            _ => TimeDisplay::Frames,
        };
    }
}

/// Imports every keyed channel and the scene's time.
pub fn apply(
    scene: &mut Scene,
    items: &BTreeMap<u32, RawItem>,
    extras: &Extras,
    pieces: &Pieces,
    report: &mut Report,
) -> Result<()> {
    scene_time(scene, items, extras);
    let rate = scene.time.rate;
    let a = &extras.actions;
    // Keyed records: the scene layer's, then any keyed record of the edit
    // layer on a channel the scene layer doesn't key (Modo keys a dynamic
    // parent's channels there).
    let mut keyed: Vec<(u32, &str, &Record)> = Vec::new();
    for (item, records) in &a.scene.records {
        for (name, r) in records {
            if r.envelope.is_some() {
                keyed.push((*item, name, r));
            }
        }
    }
    for (item, records) in &a.edit.records {
        for (name, r) in records {
            let in_scene = a
                .scene
                .records
                .get(item)
                .and_then(|m| m.get(name))
                .is_some_and(|s| s.envelope.is_some());
            if r.envelope.is_some() && !in_scene {
                keyed.push((*item, name, r));
            }
        }
    }
    let mut curves = 0usize;
    let mut manual = 0usize;
    let mut breaks = 0usize;
    let mut dropped_edits = 0usize;
    let mut no_equivalent: BTreeMap<String, usize> = BTreeMap::new();
    for (item, name, record) in keyed {
        let Some(env) = record.envelope.and_then(|e| extras.envelopes.get(&e)) else {
            *no_equivalent
                .entry("keyed channels whose curve is missing".into())
                .or_default() += 1;
            continue;
        };
        let modo_kind = items.get(&item).map_or("", |i| i.kind.as_str());
        let pivot = matches!(modo_kind, "translation" | "rotation" | "scale")
            && items
                .get(&item)
                .and_then(|i| text(i, "type"))
                .is_some_and(|t| t != "core");
        let Some(key) = target(items, Some(pieces), item, name) else {
            let what = if pivot {
                "keys on pivot transforms".to_owned()
            } else {
                format!("keyed {modo_kind} channels with no Operandi equivalent")
            };
            *no_equivalent.entry(what).or_default() += 1;
            continue;
        };
        if animation::read(scene, &key).is_none() || !keyable(scene, &key) {
            *no_equivalent
                .entry(format!(
                    "keyed {modo_kind} channels with no Operandi equivalent"
                ))
                .or_default() += 1;
            continue;
        }
        let (curve, stats) = envelope::to_curve(env, Interpolation::of_type(record.ty), rate);
        manual += stats.manual_sides;
        breaks += stats.value_breaks;
        // LXA-02: the rest value.
        let rest = layer_number(&a.setup, item, name)
            .or(match &record.value {
                Some(Value::Number(n)) => Some(*n),
                _ => None,
            })
            .or_else(|| animation::read(scene, &key))
            .unwrap_or(0.);
        if a.edit
            .records
            .get(&item)
            .and_then(|m| m.get(name))
            .is_some_and(|r| r.envelope.is_none())
        {
            dropped_edits += 1;
        }
        scene
            .animation
            .channels
            .insert(key, AnimChannel { curve, rest });
        curves += 1;
    }
    // The fields show the curves at the saved time.
    animation::pose(scene, scene.time.current);
    if curves > 0 {
        report.notes.push(format!(
            "Animation: {curves} key curves imported at {} fps",
            scene.time.rate.label()
        ));
    }
    if manual > 0 {
        report.notes.push(format!(
            "{manual} curve handles imported as Manual so the curves keep their exact shape"
        ));
    }
    if breaks > 0 {
        report.notes.push(format!(
            "{breaks} keys with separate in and out values imported as a pair of keys one tick apart"
        ));
    }
    if dropped_edits > 0 {
        report.notes.push(format!(
            "Unkeyed edits on {dropped_edits} keyed channels were not kept"
        ));
    }
    for (what, n) in no_equivalent {
        report.omissions.insert(format!("{what} not imported"), n);
    }
    Ok(())
}

/// Whether Operandi can key `key` (transforms, the item tables, pieces).
fn keyable(scene: &Scene, key: &ChannelKey) -> bool {
    animation::transform_part(&key.channel).is_some()
        || animation::keyable(scene, key.item, &key.channel)
}
