//! LXO import: user channels (custom sliders, spec
//! lxo-import-anim-rig-2026-11-24 LXA-100..102). Each `UCHN` sub-chunk of
//! an item becomes an Operandi user channel (RG-53) on that item, before
//! links and modifiers are imported, so that what reads or drives them
//! finds them (LXA-180). Their values and keys import like any channel's.
use super::phase1::{RawItem, Value, number};
use super::wire::Reader;
use super::{Control, Report, Result};
use crate::Scene;
use crate::rig::{UserChannel, UserType};

/// One `UCHN` record as stored (LXA-100).
#[derive(Clone, Debug, Default)]
pub struct UserDef {
    pub name: String,
    pub ty: String,
    /// 0 one channel, 1 XY, 2 XYZ, 3 RGB, 4 RGBA.
    pub vector: u32,
    pub default_int: i32,
    pub default_float: f64,
    /// `#` hint: the label.
    pub label: Option<String>,
    /// `%min` / `%max` hints as stored.
    pub min: Option<i32>,
    pub max: Option<i32>,
}

/// Reads one `UCHN` record.
pub(super) fn read(r: &mut Reader<'_>, c: &mut Control<'_>) -> Result<UserDef> {
    let (name, _) = r.text(c)?;
    let (ty, _) = r.text(c)?;
    let mut def = UserDef {
        name,
        ty,
        ..Default::default()
    };
    if r.done() {
        return Ok(def);
    }
    def.vector = r.u32()?;
    let _flags = r.u32()?;
    def.default_int = r.u32()? as i32;
    def.default_float = r.f32()?;
    let hints = r.u16()?;
    for _ in 0..hints {
        let (hint, _) = r.text(c)?;
        let value = r.u32()? as i32;
        if let Some(label) = hint.strip_prefix('#') {
            def.label = Some(label.to_owned());
        } else if hint == "%min" {
            def.min = Some(value);
        } else if hint == "%max" {
            def.max = Some(value);
        }
    }
    Ok(def)
}

/// LXA-102: a type name as Operandi's user channel type.
fn user_type(ty: &str) -> Option<UserType> {
    Some(match ty {
        "float" => UserType::Float,
        "distance" => UserType::Distance,
        "angle" => UserType::Angle,
        "percent" => UserType::Percentage,
        "time" => UserType::Time,
        "integer" => UserType::Integer,
        "boolean" => UserType::Boolean,
        "string" => UserType::String,
        _ => return None,
    })
}

fn mode(vector: u32) -> Option<&'static str> {
    Some(match vector {
        0 => "scalar",
        1 => "xy",
        2 => "xyz",
        3 => "rgb",
        4 => "rgba",
        _ => return None,
    })
}

/// The channel names a user channel makes (`name`, or `name.X`…).
pub fn names(def: &UserDef) -> Vec<String> {
    let comps = mode(def.vector)
        .and_then(crate::rig::mode_components)
        .unwrap_or(&[""]);
    comps
        .iter()
        .map(|c| {
            if c.is_empty() {
                def.name.clone()
            } else {
                format!("{}.{c}", def.name)
            }
        })
        .collect()
}

/// Creates every imported item's user channels (LXA-102).
pub fn create(
    scene: &mut Scene,
    items: &std::collections::BTreeMap<u32, RawItem>,
    report: &mut Report,
) {
    let mut made = 0usize;
    for item in items.values() {
        let id = u64::from(item.id) + 1;
        if item.user.is_empty() || scene.item(id).is_none() {
            continue;
        }
        for def in &item.user {
            let (Some(ty), Some(mode)) = (user_type(&def.ty), mode(def.vector)) else {
                report.omit(format!("user channels of type {}", def.ty));
                continue;
            };
            if scene.animation.rig.user_channel(id, &def.name).is_some() {
                continue;
            }
            let default = match ty {
                UserType::Integer | UserType::Boolean => f64::from(def.default_int),
                _ => def.default_float,
            };
            // Percent limits are stored in whole percent (LXA-100); other
            // types' scaling isn't known (Q-LXA-12), so they're left off.
            let limit = |v: Option<i32>| {
                (ty == UserType::Percentage)
                    .then_some(v)
                    .flatten()
                    .map(|v| f64::from(v) / 100.)
            };
            let mut channel = UserChannel {
                item: id,
                name: def.name.clone(),
                display: def.label.clone().unwrap_or_default(),
                ty,
                mode: mode.into(),
                default,
                min: limit(def.min),
                max: limit(def.max),
                choices: Vec::new(),
                values: Default::default(),
                text: String::new(),
            };
            if ty == UserType::String {
                if let Some(Value::Text(t)) = item.channels.get(&def.name) {
                    channel.text = t.clone();
                }
            } else {
                for n in names(def) {
                    if let Some(v) = number(item, &n) {
                        let _ = channel.set(&n, v);
                    }
                }
            }
            scene.animation.rig.user.push(channel);
            made += 1;
        }
    }
    if made > 0 {
        report.notes.push(format!("{made} user channels imported"));
    }
}
