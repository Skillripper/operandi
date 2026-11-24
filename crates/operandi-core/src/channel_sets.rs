//! Channel sets (BEHAVIOR-CHANNEL-PANEL CP-01..70): named, ordered lists of
//! channels picked from any items, shown as the channel panel over a 3D
//! view (UIC-188). A set holds channel references (item + channel), not
//! items, so one panel can mix a rig locator's user channels with other
//! items' rotations. Sets and the current set are scene data (saved,
//! native version 66); the panel's place is layout state (app side).
//!
//! Every change is one undo step (CP-60); choosing the current set is not
//! (view state, CP-60), but it is saved with the scene so a rigged file
//! reopens with its controls showing (CP-70, [OP]).
use crate::animation::ChannelKey;
use crate::{Command, Editor, Result, Scene};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;
use serde_json::{Map, Value};

/// One channel set (CP-02). `members` keep the order they were added in,
/// or the order rows were moved to (CP-23).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelSet {
    /// From the scene's item id counter, so renames keep links.
    pub id: u64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<ChannelKey>,
}

/// Longest set name, bytes.
pub const MAX_NAME: usize = 256;
/// Most channels in one set.
pub const MAX_MEMBERS: usize = 10_000;

impl Scene {
    pub fn channel_set(&self, id: u64) -> Option<&ChannelSet> {
        self.animation.channel_sets.iter().find(|s| s.id == id)
    }
    fn channel_set_mut(&mut self, id: u64) -> Result<&mut ChannelSet> {
        self.animation
            .channel_sets
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("No channel set {id}"))
    }
    /// The current set (CP-30), when there is one and it still exists.
    pub fn current_channel_set(&self) -> Option<&ChannelSet> {
        self.animation
            .current_set
            .and_then(|id| self.channel_set(id))
    }
    /// CP-24: drops members whose item or channel is gone; empty sets stay.
    pub fn prune_channel_sets(&mut self) {
        if self.animation.channel_sets.is_empty() {
            return;
        }
        let keep: Vec<Vec<bool>> = self
            .animation
            .channel_sets
            .iter()
            .map(|s| {
                s.members
                    .iter()
                    .map(|k| crate::animation::read(self, k).is_some())
                    .collect()
            })
            .collect();
        for (set, keep) in self.animation.channel_sets.iter_mut().zip(keep) {
            let mut i = 0;
            set.members.retain(|_| {
                i += 1;
                keep[i - 1]
            });
        }
        if self.animation.current_set.is_some() && self.current_channel_set().is_none() {
            self.animation.current_set = None;
        }
    }
    fn free_set_name(&self, base: &str) -> String {
        crate::user_defaults::free_name(base, |n| {
            self.animation.channel_sets.iter().any(|s| s.name == n)
        })
    }
}

fn check_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Give the channel set a name".into());
    }
    if name.len() > MAX_NAME {
        return Err(format!("A channel set name is at most {MAX_NAME} bytes"));
    }
    Ok(name.to_owned())
}

/// The channels a command names (`channels:"3:pos.X,5:uPct"`), else the
/// channel selection (UIC-119); each must be a channel the scene has.
fn channels(e: &Editor, a: &Map<String, Value>) -> Result<Vec<ChannelKey>> {
    let list: Vec<ChannelKey> = match a.get("channels") {
        Some(Value::String(s)) => s
            .split(',')
            .filter(|t| !t.trim().is_empty())
            .map(ChannelKey::parse)
            .collect::<Result<_>>()?,
        Some(Value::Array(v)) => v
            .iter()
            .map(|x| {
                x.as_str()
                    .ok_or_else(|| "channels are item:name texts".to_string())
                    .and_then(ChannelKey::parse)
            })
            .collect::<Result<_>>()?,
        _ => e.anim.selected_channels.iter().cloned().collect(),
    };
    for k in &list {
        if crate::animation::read(&e.scene, k).is_none() {
            return Err(format!("No channel {}", k.text()));
        }
    }
    Ok(list)
}

fn set_arg(e: &Editor, a: &Map<String, Value>) -> Result<u64> {
    match a.get("set").and_then(Value::as_u64) {
        Some(id) => e
            .scene
            .channel_set(id)
            .map(|s| s.id)
            .ok_or_else(|| format!("No channel set {id}")),
        None => e
            .scene
            .current_channel_set()
            .map(|s| s.id)
            .ok_or_else(|| "No current channel set (pick one, or give set:)".into()),
    }
}

/// One undoable change, recorded under `label`.
fn step(e: &mut Editor, label: &str, edit: impl FnOnce(&mut Scene) -> Result<()>) -> Result<()> {
    let before = e.scene.clone();
    if let Err(err) = edit(&mut e.scene) {
        e.scene = before;
        return Err(err);
    }
    if e.scene == before {
        return Ok(());
    }
    e.record(label.into(), before);
    Ok(())
}

pub fn run(e: &mut Editor, name: &str, a: &Map<String, Value>) -> Result<Option<Command>> {
    match name {
        // CP-10/11: create (from the named or selected channels), current.
        "operandi.channelSet.create" => {
            let list = channels(e, a)?;
            let base = check_name(
                a.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("Channel Set"),
            )?;
            if e.scene.next_id >= u64::MAX - 1 {
                return Err("Scene item/identity budget exhausted".into());
            }
            step(e, "Create Channel Set", |s| {
                let id = s.next_id;
                s.next_id += 1;
                let name = s.free_set_name(&base);
                let mut members = Vec::new();
                for k in list {
                    if !members.contains(&k) {
                        members.push(k);
                    }
                }
                s.animation
                    .channel_sets
                    .push(ChannelSet { id, name, members });
                s.animation.current_set = Some(id);
                Ok(())
            })?;
        }
        // CP-20: add to the current (or named) set; ones already in it stay.
        "operandi.channelSet.add" => {
            let id = set_arg(e, a)?;
            let list = channels(e, a)?;
            if list.is_empty() {
                return Err("Select channels to add (click a row's label)".into());
            }
            step(e, "Add to Channel Set", |s| {
                let set = s.channel_set_mut(id)?;
                for k in list {
                    if !set.members.contains(&k) {
                        set.members.push(k);
                    }
                }
                if set.members.len() > MAX_MEMBERS {
                    return Err(format!(
                        "A channel set holds at most {MAX_MEMBERS} channels"
                    ));
                }
                Ok(())
            })?;
        }
        // CP-22.
        "operandi.channelSet.remove" => {
            let id = set_arg(e, a)?;
            let all = a.get("all").and_then(Value::as_bool).unwrap_or(false);
            let list = if all { Vec::new() } else { channels(e, a)? };
            step(e, "Remove From Channel Set", |s| {
                let set = s.channel_set_mut(id)?;
                if all {
                    set.members.clear();
                } else {
                    set.members.retain(|k| !list.contains(k));
                }
                Ok(())
            })?;
        }
        // CP-23: move one row to `to` (0 = top), or up/down by `by`.
        "operandi.channelSet.move" => {
            let id = set_arg(e, a)?;
            let k = a
                .get("channel")
                .and_then(Value::as_str)
                .ok_or("Give channel (item:name)")
                .map_err(String::from)
                .and_then(ChannelKey::parse)?;
            let to = a.get("to").and_then(Value::as_i64);
            let by = a.get("by").and_then(Value::as_i64);
            step(e, "Move Channel", |s| {
                let set = s.channel_set_mut(id)?;
                let from = set
                    .members
                    .iter()
                    .position(|m| *m == k)
                    .ok_or_else(|| format!("{} isn't in {}", k.text(), set.name))?;
                let last = set.members.len() as i64 - 1;
                let to = match (to, by) {
                    (Some(t), _) => t,
                    (None, Some(b)) => from as i64 + b,
                    (None, None) => return Err("Give to: or by:".into()),
                }
                .clamp(0, last) as usize;
                let m = set.members.remove(from);
                set.members.insert(to, m);
                Ok(())
            })?;
        }
        "operandi.channelSet.rename" => {
            let id = set_arg(e, a)?;
            let name = check_name(a.get("name").and_then(Value::as_str).ok_or("Give name")?)?;
            step(e, "Rename Channel Set", |s| {
                if s.animation
                    .channel_sets
                    .iter()
                    .any(|x| x.id != id && x.name == name)
                {
                    return Err(format!("A channel set is already called {name}"));
                }
                s.channel_set_mut(id)?.name = name;
                Ok(())
            })?;
        }
        "operandi.channelSet.delete" => {
            let id = set_arg(e, a)?;
            step(e, "Delete Channel Set", |s| {
                s.animation.channel_sets.retain(|x| x.id != id);
                if s.animation.current_set == Some(id) {
                    s.animation.current_set = None;
                }
                Ok(())
            })?;
        }
        // CP-30/32: the current set (none closes the panel); view state,
        // not an undo step (CP-60).
        "operandi.channelSet.current" => {
            let id = match a.get("set") {
                None | Some(Value::Null) => None,
                Some(Value::String(s)) if s == "none" => None,
                Some(v) => {
                    let id = v
                        .as_u64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                        .ok_or("set is a channel set id or none")?;
                    e.scene
                        .channel_set(id)
                        .ok_or_else(|| format!("No channel set {id}"))?;
                    Some(id)
                }
            };
            e.scene.animation.current_set = id;
        }
        _ => return Err(format!("Unknown command {name}")),
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    fn editor() -> (Editor, u64, u64) {
        let mut e = Editor::default();
        let a = e.scene.add_group_locator().unwrap();
        let b = e.scene.add_group_locator().unwrap();
        (e, a, b)
    }

    #[test]
    fn create_add_move_remove_and_undo() {
        let (mut e, a, b) = editor();
        run(
            &mut e,
            "operandi.channelSet.create",
            &args(json!({"name": "Ship", "channels": format!("{a}:pos.X,{b}:rot.Z")})),
        )
        .unwrap();
        let set = e.scene.current_channel_set().unwrap().clone();
        assert_eq!(set.name, "Ship");
        assert_eq!(set.members.len(), 2);
        // Adding one already there does nothing (CP-20).
        run(
            &mut e,
            "operandi.channelSet.add",
            &args(json!({"channels": format!("{a}:pos.X,{a}:rot.Y")})),
        )
        .unwrap();
        let m = &e.scene.current_channel_set().unwrap().members;
        assert_eq!(m.len(), 3);
        run(
            &mut e,
            "operandi.channelSet.move",
            &args(json!({"channel": format!("{a}:rot.Y"), "to": 0})),
        )
        .unwrap();
        assert_eq!(
            e.scene.current_channel_set().unwrap().members[0],
            ChannelKey::new(a, "rot.Y")
        );
        run(
            &mut e,
            "operandi.channelSet.remove",
            &args(json!({"channels": format!("{b}:rot.Z")})),
        )
        .unwrap();
        assert_eq!(e.scene.current_channel_set().unwrap().members.len(), 2);
        e.dispatch(Command::Undo).unwrap();
        assert_eq!(e.scene.current_channel_set().unwrap().members.len(), 3);
        // Bad channels are refused and change nothing.
        assert!(
            run(
                &mut e,
                "operandi.channelSet.add",
                &args(json!({"channels": "999:pos.X"}))
            )
            .is_err()
        );
    }

    #[test]
    fn current_set_is_not_an_undo_step_and_deleted_items_drop_out() {
        let (mut e, a, b) = editor();
        run(
            &mut e,
            "operandi.channelSet.create",
            &args(json!({"channels": format!("{a}:pos.X,{b}:pos.Y")})),
        )
        .unwrap();
        let id = e.scene.animation.current_set.unwrap();
        run(
            &mut e,
            "operandi.channelSet.current",
            &args(json!({"set": "none"})),
        )
        .unwrap();
        assert_eq!(e.scene.animation.current_set, None);
        e.dispatch(Command::Undo).unwrap();
        // Undo takes back the create, not the current-set change.
        assert!(e.scene.channel_set(id).is_none());
        e.dispatch(Command::Redo).unwrap();
        e.scene.items.retain(|i| i.id != b);
        e.scene.prune_channel_sets();
        assert_eq!(
            e.scene.channel_set(id).unwrap().members,
            vec![ChannelKey::new(a, "pos.X")]
        );
    }

    #[test]
    fn sets_save_and_reopen() {
        let (mut e, a, _) = editor();
        run(
            &mut e,
            "operandi.channelSet.create",
            &args(json!({"name": "Rig", "channels": format!("{a}:pos.X")})),
        )
        .unwrap();
        let bytes = crate::persistence::encode(&e.scene).unwrap();
        let back = crate::persistence::decode(&bytes).unwrap();
        assert_eq!(back.animation.channel_sets, e.scene.animation.channel_sets);
        assert_eq!(back.animation.current_set, e.scene.animation.current_set);
    }
}
