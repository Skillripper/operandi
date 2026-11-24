//! Loading a scene into another (File > Import, item and assembly
//! presets): the arriving items' animation and rig come with them. Key
//! curves, rig pieces, user channels and pivots are renumbered onto the
//! new item ids, skins follow their joints, and a scene that had no keys
//! yet takes the arriving scene's frame rate and time range so the
//! animation plays as it was made.
use crate::animation::{AnimChannel, ChannelKey};
use crate::mesh_stack::OpKind;
use crate::{Result, Scene};
use std::collections::BTreeMap;

/// `text` ("item:channel") with its item renumbered by `map`.
fn channel_text(text: &str, map: &BTreeMap<u64, u64>) -> Option<String> {
    let k = ChannelKey::parse(text).ok()?;
    map.get(&k.item)
        .map(|&id| ChannelKey::new(id, &k.channel).text())
}

/// Carries `source`'s animation for the items `ids` renumbered (old id →
/// new id) into `scene`. `next` is the next free id.
pub(crate) fn carry(
    scene: &mut Scene,
    next: &mut u64,
    source: &Scene,
    ids: &BTreeMap<u64, u64>,
) -> Result<()> {
    let from = &source.animation;
    if from.channels.is_empty() && from.rig.is_default() {
        return Ok(());
    }
    let had_keys = !scene.animation.channels.is_empty();
    // Pieces whose item came along get new ids from the item counter.
    let mut map = ids.clone();
    for p in from.rig.pieces.iter().filter(|p| ids.contains_key(&p.item)) {
        let start = (*next).max(scene.next_id);
        map.insert(p.id, start);
        *next = start + 1;
        scene.next_id = *next;
    }
    for p in &from.rig.pieces {
        let Some(&id) = map.get(&p.id) else {
            continue;
        };
        let mut copy = p.clone();
        copy.id = id;
        copy.item = map[&p.item];
        copy.targets = p
            .targets
            .iter()
            .filter_map(|t| map.get(t).copied())
            .collect();
        copy.refs = p
            .refs
            .iter()
            .filter_map(|(k, v)| map.get(v).map(|&n| (k.clone(), n)))
            .collect();
        for (key, value) in copy.names.iter_mut() {
            if key.starts_with("in.") {
                if let Some(t) = channel_text(value, &map) {
                    *value = t;
                }
            } else if matches!(key.as_str(), "mid" | "end")
                && let Some(&n) = value.parse::<u64>().ok().and_then(|o| map.get(&o))
            {
                *value = n.to_string();
            }
        }
        scene.animation.rig.pieces.push(copy);
    }
    for (k, c) in &from.channels {
        if let Some(&id) = map.get(&k.item) {
            scene.animation.channels.insert(
                ChannelKey::new(id, &k.channel),
                AnimChannel {
                    curve: c.curve.clone(),
                    rest: c.rest,
                },
            );
        }
    }
    for u in &from.rig.user {
        if let Some(&id) = map.get(&u.item) {
            let mut copy = u.clone();
            copy.item = id;
            scene.animation.rig.user.push(copy);
        }
    }
    for (item, pivot) in &from.rig.pivots {
        if let Some(&id) = map.get(item) {
            scene.animation.rig.pivots.insert(id, *pivot);
        }
    }
    // Skins name their joints by id; joints' weight maps by name.
    for &new in ids.values() {
        let Some(stack) = scene.item_mut(new).and_then(|i| i.mesh_stack.as_mut()) else {
            continue;
        };
        for op in &mut stack.ops {
            if let OpKind::Op { type_id, values } = &mut op.kind
                && type_id == crate::skin::SKIN_OP
                && let Some(joints) = values.get_mut("joints").and_then(|j| j.as_array_mut())
            {
                for j in joints.iter_mut() {
                    if let Some(&n) = j.as_u64().and_then(|o| map.get(&o)) {
                        *j = n.into();
                    }
                }
            }
        }
    }
    for item in source.items.iter().filter(|i| i.is_joint()) {
        if let Some(&id) = ids.get(&item.id) {
            crate::skin_weights::follow_rename(scene, id, &item.name);
        }
    }
    if !had_keys && !from.channels.is_empty() {
        scene.time = source.time;
        scene.animation.settings.display = from.settings.display;
        scene.animation.settings.current_range = from.settings.current_range;
    }
    crate::animation::pose(scene, scene.time.current);
    Ok(())
}
