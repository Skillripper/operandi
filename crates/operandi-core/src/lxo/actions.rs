//! LXO import: action layers (`ACTN`, spec lxo-import-anim-rig-2026-11-24
//! LXA-01..05). Each layer is a list of records: an `ITEM` sub-chunk names
//! the item (its reference ID), then that item's `CHAN` records follow
//! until the next `ITEM`. A `CHAN` is the channel-name index, the type
//! word, the envelope index (0 when not keyed) and the value.
use super::phase1::Value;
use super::wire::{Chunk, Reader};
use super::{Control, Result};
use std::collections::BTreeMap;

/// One channel record of a layer.
#[derive(Clone, Debug)]
pub struct Record {
    /// The CHAN type word (LXA-04).
    pub ty: u16,
    /// The envelope it names, for a keyed channel (type bit 0x10).
    pub envelope: Option<u32>,
    pub value: Option<Value>,
}

/// One layer: records by item reference ID, then channel name.
#[derive(Clone, Debug, Default)]
pub struct Layer {
    pub records: BTreeMap<u32, BTreeMap<String, Record>>,
    /// Gradient channels (`GRAD`): item → channel → envelope (LXA-110).
    pub grads: BTreeMap<u32, BTreeMap<String, u32>>,
}

/// The three default layers (LXA-01; extra user layers are Q-LXA-11).
#[derive(Clone, Debug, Default)]
pub struct Actions {
    pub setup: Layer,
    pub scene: Layer,
    pub edit: Layer,
    /// Layers of other kinds, not read (counted for the report).
    pub other: usize,
}

/// Reads one `ACTN` chunk into `actions`; returns how many of its text
/// records weren't UTF-8.
pub(super) fn read(
    ch: &Chunk<'_>,
    names: &[String],
    actions: &mut Actions,
    c: &mut Control<'_>,
) -> Result<usize> {
    let mut r = Reader::new(ch.data, ch.offset);
    let (_name, a) = r.text(c)?;
    let (kind, b) = r.text(c)?;
    let mut not_utf8 = usize::from(a || b);
    let _id = r.u32()?;
    let layer = match kind.as_str() {
        "setup" => &mut actions.setup,
        "scene" if actions.scene.records.is_empty() => &mut actions.scene,
        "edit" => &mut actions.edit,
        _ => {
            actions.other += 1;
            return Ok(not_utf8);
        }
    };
    let mut owner: Option<u32> = None;
    for sub in r.chunks(true, c)? {
        let mut s = Reader::new(sub.data, sub.offset);
        match &sub.id {
            b"ITEM" => owner = Some(s.u32()?),
            b"CHAN" => {
                let Some(item) = owner else {
                    continue;
                };
                c.charge(96)?;
                let index = s.vx()? as usize;
                let ty = s.u16()?;
                let envelope = s.vx()?;
                let value = if s.done() {
                    None
                } else {
                    match ty & 0xf {
                        1 => Some(Value::Number(s.u32()? as i32 as f64)),
                        2 => Some(Value::Number(s.f32()?)),
                        3 => {
                            let (text, bad) = s.text(c)?;
                            not_utf8 += usize::from(bad);
                            Some(Value::Text(text))
                        }
                        _ => None,
                    }
                };
                let Some(name) = names.get(index) else {
                    continue;
                };
                layer.records.entry(item).or_default().insert(
                    name.clone(),
                    Record {
                        ty,
                        envelope: (ty & 0x10 != 0).then_some(envelope),
                        value,
                    },
                );
            }
            b"GRAD" => {
                let Some(item) = owner else {
                    continue;
                };
                let index = s.vx()? as usize;
                let envelope = s.vx()?;
                if let Some(name) = names.get(index) {
                    layer
                        .grads
                        .entry(item)
                        .or_default()
                        .insert(name.clone(), envelope);
                }
            }
            // CHNS (strings), PRNT and others
            // carry nothing the import reads.
            _ => {}
        }
    }
    Ok(not_utf8)
}
