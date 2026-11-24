//! LXO import: envelopes (`ENVL`) and how they become Operandi key curves
//! (spec lxo-import-anim-rig-2026-11-24, LXA-10..21).
//!
//! An envelope is a float curve (kind 0: per key `KEY`, `TANI`, `TANO`,
//! `FLAG`) or an integer curve (kind 1: per key `IKEY`), with its
//! behaviour before the first key and after the last (`PRE`, `POST`).
//! Between two keys a float curve is a cubic Bezier in (seconds, value)
//! whose handles come from the stored slope and handle length of each side
//! (LXA-16). Operandi's curves use the same shape: the stored slopes are
//! imported with their slope types, and a side whose type Operandi would
//! compute differently is imported as Manual with the stored slope
//! (LXA-19), so the imported curve matches the file's exactly.
use super::wire::{Chunk, Reader};
use super::{Control, Result};
use crate::anim_curve::{Behavior, Curve, Key, Slope};
use crate::time_tick::{FrameRate, Tick};

/// One side of a key as stored (LXA-11).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Side {
    /// LXA-13 slope type bits.
    pub slope_type: u16,
    /// LXA-14: 0 automatic, 256 manual.
    pub weight_type: u16,
    /// Value per second.
    pub slope: f64,
    /// Handle length in the (seconds, value) plane.
    pub weight: f64,
}

/// One key as stored.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EnvKey {
    /// Seconds.
    pub time: f64,
    pub value: f64,
    pub incoming: Side,
    pub outgoing: Side,
    /// LXA-15 broken flags: 1 value, 2 slope, 4 weight.
    pub broken: u32,
    /// The outgoing value (used when the value is broken).
    pub out_value: f64,
}

/// One `ENVL` chunk.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Envelope {
    pub index: u32,
    /// Kind 1: an integer curve (LXA-12).
    pub integer: bool,
    pub pre: u16,
    pub post: u16,
    pub keys: Vec<EnvKey>,
}

/// LXA-04: how a keyed channel interpolates between keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Interpolation {
    #[default]
    Curve,
    Linear,
    Stepped,
}

impl Interpolation {
    /// From a channel's type word (bits 0x200 and 0x400).
    pub fn of_type(ty: u16) -> Interpolation {
        if ty & 0x400 != 0 {
            Interpolation::Stepped
        } else if ty & 0x200 != 0 {
            Interpolation::Linear
        } else {
            Interpolation::Curve
        }
    }
}

/// LXA-13 slope type values.
const MANUAL: u16 = 0;
const AUTO: u16 = 1;
const LINEAR_IN: u16 = 2;
const LINEAR_OUT: u16 = 4;
const FLAT: u16 = 8;
const AUTO_FLAT: u16 = 16;
const STEPPED: u16 = 32;
const SMOOTH_FLAT: u16 = 64;

fn side(r: &mut Reader<'_>) -> Result<Side> {
    Ok(Side {
        slope_type: r.u16()?,
        weight_type: r.u16()?,
        slope: r.f32()?,
        weight: r.f32()?,
    })
}

/// Reads one `ENVL` chunk (LXA-10..12).
pub(super) fn read(ch: &Chunk<'_>, c: &mut Control<'_>) -> Result<Envelope> {
    let mut r = Reader::new(ch.data, ch.offset);
    let index = r.vx()?;
    let kind = r.u32()?;
    let mut env = Envelope {
        index,
        integer: kind == 1,
        pre: 1,
        post: 1,
        keys: Vec::new(),
    };
    for sub in r.chunks(true, c)? {
        let mut s = Reader::new(sub.data, sub.offset);
        match &sub.id {
            b"PRE " => env.pre = s.u16()?,
            b"POST" => env.post = s.u16()?,
            b"KEY " => {
                c.charge(96)?;
                let time = s.f32()?;
                let value = s.f32()?;
                env.keys.push(EnvKey {
                    time,
                    value,
                    ..Default::default()
                });
            }
            b"IKEY" => {
                c.charge(96)?;
                let time = s.f32()?;
                let value = s.u32()? as i32 as f64;
                env.keys.push(EnvKey {
                    time,
                    value,
                    ..Default::default()
                });
            }
            // Fields in the order the files hold them (LXA-11: slope
            // before weight, unlike the public page).
            b"TANI" => {
                if let Some(k) = env.keys.last_mut() {
                    k.incoming = side(&mut s)?;
                }
            }
            b"TANO" => {
                if let Some(k) = env.keys.last_mut() {
                    k.broken = s.u32()?;
                    k.outgoing = side(&mut s)?;
                    k.out_value = s.f32()?;
                }
            }
            // FLAG (0x0F00, meaning unknown, Q-LXA-10) and anything newer.
            _ => {}
        }
    }
    env.keys.sort_by(|a, b| a.time.total_cmp(&b.time));
    env.keys.dedup_by(|b, a| a.time == b.time);
    Ok(env)
}

/// LXA-20: a PRE/POST value as Operandi's behaviour.
pub fn behavior(word: u16) -> Behavior {
    match word {
        0 => Behavior::Reset,
        2 => Behavior::Repeat,
        3 => Behavior::OscillateSmooth,
        4 => Behavior::OffsetRepeatSmooth,
        5 => Behavior::Linear,
        6 => Behavior::Constant,
        7 => Behavior::Oscillate,
        8 => Behavior::OffsetRepeat,
        _ => Behavior::ConstantSmooth,
    }
}

/// LXA-18: a key time in seconds as a tick, on the frame when it lies
/// within 0.001 frames of one.
pub fn key_time(seconds: f64, rate: FrameRate) -> Tick {
    let frames = seconds * rate.as_f64();
    let whole = frames.round();
    if (frames - whole).abs() <= 1e-3 && whole.abs() < 1e12 {
        rate.tick_of(whole as i64)
    } else {
        Tick::from_seconds(seconds)
    }
}

/// What converting a curve did, for the import report.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Conversion {
    /// LXA-19: sides imported as Manual because Operandi's own slope for
    /// their type differs from the stored one.
    pub manual_sides: usize,
    /// LXA-15: keys with a broken value, imported as two keys one tick
    /// apart (the second holding the outgoing value).
    pub value_breaks: usize,
}

fn slope_of(bits: u16) -> Slope {
    match bits {
        MANUAL => Slope::Manual,
        AUTO => Slope::Auto,
        LINEAR_IN | LINEAR_OUT => Slope::Linear,
        FLAT => Slope::Flat,
        AUTO_FLAT => Slope::AutoFlat,
        SMOOTH_FLAT => Slope::SmoothFlat,
        // Stepped is handled by the caller (LXA-16); an unknown word keeps
        // the stored slope.
        _ => Slope::Manual,
    }
}

/// LXA-17: a stored handle length as Operandi's relative weight (1 = a
/// handle one third of the segment long in time).
fn relative_weight(weight: f64, slope: f64, gap: Option<f64>) -> f64 {
    let Some(gap) = gap.filter(|g| *g > 0.) else {
        return 1.;
    };
    let w = (weight / (1. + slope * slope).sqrt()) / (gap / 3.);
    if !w.is_finite() || w <= 0. || (w - 1.).abs() < 1e-4 {
        1.
    } else {
        w
    }
}

/// Converts an envelope into an Operandi curve (LXA-12..21).
pub fn to_curve(
    env: &Envelope,
    interpolation: Interpolation,
    rate: FrameRate,
) -> (Curve, Conversion) {
    let mut stats = Conversion::default();
    let mut curve = Curve {
        pre: behavior(env.pre),
        post: behavior(env.post),
        ..Default::default()
    };
    if env.integer || interpolation == Interpolation::Stepped {
        // LXA-12 / LXA-16: hold each key's value until the next.
        curve.stepped = true;
        for k in &env.keys {
            curve.put(Key::new(key_time(k.time, rate), k.value));
        }
        return (curve, stats);
    }
    if interpolation == Interpolation::Linear {
        // LXA-16: straight lines between the keys.
        for k in &env.keys {
            let mut key = Key::new(key_time(k.time, rate), k.value);
            key.slope_in = Slope::Linear;
            key.slope_out = Slope::Linear;
            curve.put(key);
        }
        return (curve, stats);
    }
    // Stored slopes per key side, kept beside the curve's keys (same
    // order) for the LXA-19 comparison.
    let mut stored: Vec<(f64, f64)> = Vec::new();
    let n = env.keys.len();
    for (i, k) in env.keys.iter().enumerate() {
        let before = i.checked_sub(1).map(|j| k.time - env.keys[j].time);
        let after = env.keys.get(i + 1).map(|b| b.time - k.time);
        let mut key = Key::new(key_time(k.time, rate), k.value);
        key.slope_in = slope_of(k.incoming.slope_type);
        key.manual_in = k.incoming.slope;
        key.weight_in = relative_weight(k.incoming.weight, k.incoming.slope, before);
        key.slope_out = slope_of(k.outgoing.slope_type);
        key.manual_out = k.outgoing.slope;
        key.weight_out = relative_weight(k.outgoing.weight, k.outgoing.slope, after);
        // LXA-16: a Stepped *outgoing* type does nothing; an incoming one
        // holds the previous key's value up to this key, which Operandi
        // writes on the previous key's out side.
        if k.outgoing.slope_type == STEPPED {
            key.slope_out = Slope::Manual;
        }
        if k.incoming.slope_type == STEPPED {
            key.slope_in = Slope::Manual;
            if let Some(prev) = curve.keys.last_mut() {
                prev.slope_out = Slope::Stepped;
            }
        }
        key.broken = k.broken != 0
            || key.slope_in != key.slope_out
            || (key.manual_in - key.manual_out).abs() > 1e-9
            || (key.weight_in - key.weight_out).abs() > 1e-9;
        let value_break = k.broken & 1 != 0 && i + 1 < n && (k.out_value - k.value).abs() > 0.;
        if value_break {
            // The key holds its own value at its time; one tick later the
            // curve carries on from the outgoing value (LXA-16 P0).
            stats.value_breaks += 1;
            let mut out = key.clone();
            out.time = Tick(key.time.0 + 1);
            out.value = k.out_value;
            out.slope_in = Slope::Flat;
            out.manual_in = 0.;
            out.weight_in = 1.;
            out.broken = true;
            key.slope_out = Slope::Stepped;
            key.manual_out = 0.;
            key.weight_out = 1.;
            key.broken = true;
            curve.keys.push(key);
            stored.push((k.incoming.slope, 0.));
            curve.keys.push(out);
            stored.push((0., k.outgoing.slope));
        } else {
            curve.keys.push(key);
            stored.push((k.incoming.slope, k.outgoing.slope));
        }
    }
    curve.sort();
    stats.manual_sides = keep_stored_slopes(&mut curve, &stored);
    (curve, stats)
}

/// LXA-19: every side the curve uses whose computed slope differs from the
/// stored one by more than 1e-4 per second becomes Manual with the stored
/// slope. Repeated until nothing changes (a side turning Manual can change
/// its segment's no-overshoot limit). Returns how many sides changed.
fn keep_stored_slopes(curve: &mut Curve, stored: &[(f64, f64)]) -> usize {
    let n = curve.keys.len();
    if n < 2 || stored.len() != n {
        return 0;
    }
    let mut changed = 0;
    for _ in 0..4 {
        let mut flips: Vec<(usize, bool)> = Vec::new();
        for i in 0..n - 1 {
            if curve.keys[i].slope_out == Slope::Stepped {
                continue;
            }
            let (m0, m1) = curve.segment_slopes(i);
            if curve.keys[i].slope_out != Slope::Manual && (m0 - stored[i].1).abs() > 1e-4 {
                flips.push((i, false));
            }
            if curve.keys[i + 1].slope_in != Slope::Manual && (m1 - stored[i + 1].0).abs() > 1e-4 {
                flips.push((i + 1, true));
            }
        }
        if flips.is_empty() {
            break;
        }
        for (i, incoming) in flips {
            let k = &mut curve.keys[i];
            if incoming {
                k.slope_in = Slope::Manual;
                k.manual_in = stored[i].0;
            } else {
                k.slope_out = Slope::Manual;
                k.manual_out = stored[i].1;
            }
            k.broken = k.broken || k.slope_in != k.slope_out;
            changed += 1;
        }
    }
    changed
}
