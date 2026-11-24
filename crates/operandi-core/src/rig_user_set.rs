//! Set User Channel (RG-53): a user channel's value from its slider, one
//! undo step. Keys like any channel edit (Auto Key, TK-30), so an imported
//! rig's control sliders can be driven and animated from the panel.
use crate::{Command, Editor};
use serde_json::{Map, Value};

type Args = Map<String, Value>;

pub fn run(e: &mut Editor, _name: &str, a: &Args) -> Result<Option<Command>, String> {
    let item = match a.get("item").and_then(Value::as_u64) {
        Some(i) => i,
        None => e.scene.selection.item.ok_or("Give item")?,
    };
    let name = a
        .get("name")
        .and_then(Value::as_str)
        .ok_or("Give name (the channel, or name.X for a component)")?
        .to_owned();
    let value = match a.get("value") {
        Some(Value::Bool(b)) => f64::from(u8::from(*b)),
        Some(v) => v
            .as_f64()
            .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
            .ok_or("value must be a number")?,
        None => return Err("Give value".into()),
    };
    let channel = e
        .scene
        .animation
        .rig
        .user_channel(item, &name)
        .ok_or_else(|| format!("No user channel {name} on item {item}"))?;
    if channel.get(&name) == Some(value) {
        return Ok(None);
    }
    let before = e.scene.clone();
    e.scene
        .animation
        .rig
        .user_channel_mut(item, &name)
        .expect("checked")
        .set(&name, value)?;
    e.record(format!("Set {name}"), before);
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rig::{UserChannel, UserType};

    fn editor() -> (Editor, u64) {
        let mut e = Editor::default();
        let id = e.scene.add_group_locator().unwrap();
        e.scene.animation.rig.user.push(UserChannel {
            item: id,
            name: "wings".into(),
            display: "Wings".into(),
            ty: UserType::Percentage,
            mode: "scalar".into(),
            default: 0.,
            min: Some(0.),
            max: Some(1.),
            choices: Vec::new(),
            values: Default::default(),
            text: String::new(),
        });
        (e, id)
    }

    fn args(id: u64, value: Value) -> Args {
        let mut a = Args::new();
        a.insert("item".into(), id.into());
        a.insert("name".into(), "wings".into());
        a.insert("value".into(), value);
        a
    }

    #[test]
    fn sets_clamps_and_undoes() {
        let (mut e, id) = editor();
        run(&mut e, "", &args(id, 0.4.into())).unwrap();
        let get = |e: &Editor| {
            e.scene
                .animation
                .rig
                .user_channel(id, "wings")
                .unwrap()
                .get("wings")
        };
        assert_eq!(get(&e), Some(0.4));
        run(&mut e, "", &args(id, 3.0.into())).unwrap();
        assert_eq!(get(&e), Some(1.), "clamped to its range");
        e.dispatch(crate::Command::Undo).unwrap();
        assert_eq!(get(&e), Some(0.4));
        assert!(run(&mut e, "", &args(id + 99, 0.2.into())).is_err());
    }
}
