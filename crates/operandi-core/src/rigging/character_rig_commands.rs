//! The character rig's commands (ACS-00: every feature is a named command
//! the MCP bridge reaches, with a query, UIC-160..165). Edits are one undo
//! step each (UIC-224); switching context is a UI command (ACS-02,
//! UIC-225). See character_rig.rs.
use crate::character_rig::{
    self as cr, BipedOptions, CharacterRig, RigContext, Side, side_of, world_pos,
};
use crate::{Command, Editor, Result, Scene};
use glam::DVec3;
use serde_json::{Map, Value, json};

type Args = Map<String, Value>;

/// The refusal when there's no rig to work on.
pub const NO_RIG: &str = "Add a Biped rig first (Rig tab ▸ New Biped Rig)";

fn id_arg(a: &Args, name: &str) -> Result<Option<u64>> {
    match a.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n
            .as_u64()
            .map(Some)
            .ok_or_else(|| format!("{name} is an item id")),
        Some(Value::String(s)) => s
            .trim()
            .parse()
            .map(Some)
            .map_err(|_| format!("{name} is an item id")),
        Some(_) => Err(format!("{name} is an item id")),
    }
}

fn ids(a: &Args, name: &str) -> Result<Option<Vec<u64>>> {
    match a.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(Some(vec![
            n.as_u64().ok_or(format!("{name} are item ids"))?,
        ])),
        Some(Value::String(s)) => s
            .split(',')
            .map(|p| p.trim().parse().map_err(|_| format!("{name} are item ids")))
            .collect::<Result<Vec<u64>>>()
            .map(Some),
        Some(Value::Array(list)) => list
            .iter()
            .map(|v| v.as_u64().ok_or_else(|| format!("{name} are item ids")))
            .collect::<Result<Vec<u64>>>()
            .map(Some),
        Some(_) => Err(format!("{name} are item ids")),
    }
}

fn num(a: &Args, name: &str) -> Result<Option<f64>> {
    match a.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(n.as_f64()),
        Some(Value::String(s)) => s
            .trim()
            .parse()
            .map(Some)
            .map_err(|_| format!("{name} is a number")),
        Some(_) => Err(format!("{name} is a number")),
    }
}

fn flag(a: &Args, name: &str) -> Result<Option<bool>> {
    match a.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(Value::String(s)) => crate::command_args::boolean(name, s).map(Some),
        Some(_) => Err(format!("{name} must be true or false")),
    }
}

fn text<'a>(a: &'a Args, name: &str) -> Option<&'a str> {
    a.get(name).and_then(Value::as_str)
}

fn vec3(a: &Args, name: &str) -> Result<Option<DVec3>> {
    let Some(v) = a.get(name) else {
        return Ok(None);
    };
    let parts: Vec<f64> = match v {
        Value::Null => return Ok(None),
        Value::String(s) => s
            .split(',')
            .map(|p| {
                p.trim()
                    .parse::<f64>()
                    .map_err(|_| format!("{name} is x,y,z"))
            })
            .collect::<Result<_>>()?,
        Value::Array(list) => list
            .iter()
            .map(|p| p.as_f64().ok_or_else(|| format!("{name} is x,y,z")))
            .collect::<Result<_>>()?,
        _ => return Err(format!("{name} is x,y,z")),
    };
    match parts.as_slice() {
        [x, y, z] if parts.iter().all(|v| v.is_finite()) => Ok(Some(DVec3::new(*x, *y, *z))),
        _ => Err(format!("{name} is x,y,z")),
    }
}

/// The rig a command works on (ACS-10 edit rig): `rig` (its root id),
/// else the rig of the selected item, else the newest rig.
pub fn target_rig(scene: &Scene, a: &Args) -> Result<u64> {
    if let Some(r) = id_arg(a, "rig")? {
        return scene
            .character_rig(r)
            .map(|c| c.root)
            .ok_or_else(|| format!("{r} isn't a character rig's root"));
    }
    scene
        .selected_items()
        .into_iter()
        .find_map(|i| scene.character_rig_of(i).map(|c| c.root))
        .or_else(|| scene.character_rigs().last().map(|c| c.root))
        .ok_or_else(|| NO_RIG.into())
}

/// A guide by id or by role (`leg.L.knee`) in rig `root`.
fn guide_arg(scene: &Scene, root: u64, a: &Args) -> Result<u64> {
    let rig = scene.character_rig(root).ok_or(NO_RIG)?;
    let v = a
        .get("guide")
        .ok_or("Give guide (an id or a role such as leg.L.knee)")?;
    if let Some(role) = v.as_str().filter(|s| s.contains('.')) {
        return rig
            .guide(role)
            .ok_or_else(|| format!("No guide {role} in this rig"));
    }
    let id = id_arg(a, "guide")?.ok_or("Give guide")?;
    rig.guides
        .iter()
        .any(|p| p.item == id)
        .then_some(id)
        .ok_or_else(|| format!("{id} isn't one of this rig's guides"))
}

/// Runs `edit` as one undo step in Setup mode (ACS-02: guides and the
/// bind skeleton are the rig's setup state), then puts Setup back as the
/// rig's context wants it.
fn setup_step(
    e: &mut Editor,
    root: u64,
    label: &str,
    edit: impl FnOnce(&mut Editor) -> Result<bool>,
) -> Result<()> {
    let was = e.scene.animation.setup;
    set_setup(e, true);
    let result = e.anim_step(label, edit);
    let want = e
        .scene
        .character_rig(root)
        .map_or(was, |c| c.context.is_setup());
    set_setup(e, want);
    result
}

fn set_setup(e: &mut Editor, on: bool) {
    if e.scene.animation.setup != on {
        e.scene.animation.setup = on;
        let now = e.scene.time.current;
        crate::animation::pose(&mut e.scene, now);
    }
}

/// The handler for every `operandi.acs.*` edit.
pub fn run(e: &mut Editor, name: &str, a: &Args) -> Result<Option<Command>> {
    match name {
        "operandi.acs.rigAdd" => {
            let preset = text(a, "preset").unwrap_or("biped");
            if !preset.eq_ignore_ascii_case("biped") {
                return Err(format!(
                    "preset is biped (more presets come later), not {preset}"
                ));
            }
            let height = num(a, "height")?.unwrap_or(1.8);
            let at = vec3(a, "position")?.unwrap_or(DVec3::ZERO);
            let label = text(a, "name").unwrap_or("Biped").to_string();
            if label.trim().is_empty() {
                return Err("name can't be empty".into());
            }
            let mut made = 0;
            e.anim_step("New Biped Rig", |e| {
                made = cr::add_biped(&mut e.scene, &label, at, height)?;
                e.scene.select_item(Some(made));
                Ok(true)
            })?;
            Ok(None)
        }
        "operandi.acs.options" => {
            let root = target_rig(&e.scene, a)?;
            let mut options = e.scene.character_rig(root).ok_or(NO_RIG)?.options;
            for (field, _, _) in BipedOptions::FIELDS {
                // "more" / "fewer" step the count (the Rig tab's buttons).
                let now = f64::from(options.get(field).unwrap_or(0));
                let v = match text(a, field).map(str::trim) {
                    Some("more") => Some(now + 1.),
                    Some("fewer") => Some(now - 1.),
                    _ => num(a, field)?,
                };
                if let Some(v) = v {
                    if v.fract() != 0. || v < 0. {
                        return Err(format!("{field} is a whole number"));
                    }
                    options.set(field, v as u32)?;
                }
            }
            let symmetry = match text(a, "symmetry").map(str::trim) {
                Some("toggle") => Some(!e.scene.character_rig(root).is_some_and(|c| c.symmetry)),
                _ => flag(a, "symmetry")?,
            };
            let size = num(a, "referenceSize")?;
            if size.is_some_and(|s| !(s.is_finite() && s > 0.)) {
                return Err("referenceSize must be above 0".into());
            }
            setup_step(e, root, "Rig Options", |e| {
                let rig = e.scene.character_rig(root).expect("checked").clone();
                let mut changed = options != rig.options;
                let c = e.scene.character_rig_mut(root).expect("checked");
                c.options = options;
                if let Some(s) = symmetry
                    && s != c.symmetry
                {
                    c.symmetry = s;
                    changed = true;
                }
                if let Some(s) = size
                    && (s - rig.reference_size).abs() > 1e-12
                {
                    // ACS-11: the guides scale about the root.
                    c.reference_size = s;
                    let k = s / rig.reference_size;
                    let base = world_pos(&e.scene, root);
                    for p in &rig.guides {
                        if e.scene.item(p.item).is_some() {
                            let at = world_pos(&e.scene, p.item);
                            cr::set_world_pos(&mut e.scene, p.item, base + (at - base) * k)?;
                        }
                    }
                    changed = true;
                }
                if symmetry == Some(true) {
                    cr::symmetrize(&mut e.scene, root, Side::Left)?;
                }
                // ACS-22: changing an option rebuilds that part.
                if options != rig.options && !rig.joints.is_empty() {
                    cr::build_skeleton(&mut e.scene, root)?;
                }
                Ok(changed)
            })?;
            Ok(None)
        }
        "operandi.acs.context" => {
            // ACS-02: a UI command; no undo step.
            let root = target_rig(&e.scene, a)?;
            let context = RigContext::parse(text(a, "context").ok_or("Give context")?)?;
            let tool = e.settle_tool();
            if let Some(c) = e.scene.character_rig_mut(root) {
                c.context = context;
            }
            set_setup(e, context.is_setup());
            // A selected item the context hides is let go.
            if let Some(sel) = e.scene.selection.item
                && cr::hidden_by_context(&e.scene, sel)
            {
                e.scene.select_item(None);
            }
            e.resume_tool(tool);
            Ok(None)
        }
        "operandi.acs.guideMove" => {
            let root = target_rig(&e.scene, a)?;
            let guide = guide_arg(&e.scene, root, a)?;
            let to = vec3(a, "position")?.ok_or("Give position x,y,z")?;
            setup_step(e, root, "Move Guide", |e| {
                cr::set_world_pos(&mut e.scene, guide, to)?;
                if e.scene.character_rig(root).is_some_and(|c| c.symmetry) {
                    let side = side_of(
                        e.scene
                            .character_rig(root)
                            .and_then(|c| c.role_of(guide))
                            .unwrap_or(""),
                    );
                    let lead = if side == Side::Right {
                        Side::Right
                    } else {
                        Side::Left
                    };
                    cr::symmetrize(&mut e.scene, root, lead)?;
                }
                e.scene.select_item(Some(guide));
                Ok(true)
            })?;
            Ok(None)
        }
        "operandi.acs.guideMirror" => {
            let root = target_rig(&e.scene, a)?;
            let from = match text(a, "from")
                .unwrap_or("left")
                .to_ascii_lowercase()
                .as_str()
            {
                "left" | "l" => Side::Left,
                "right" | "r" => Side::Right,
                other => return Err(format!("from is left or right, not {other}")),
            };
            setup_step(e, root, "Mirror Guides", |e| {
                let before = e.scene.clone();
                cr::symmetrize(&mut e.scene, root, from)?;
                Ok(before.items != e.scene.items)
            })?;
            Ok(None)
        }
        "operandi.acs.guideReset" => {
            let root = target_rig(&e.scene, a)?;
            setup_step(e, root, "Reset Guides", |e| {
                let before = e.scene.clone();
                restore_missing_guides(&mut e.scene, root)?;
                cr::place_template(&mut e.scene, root)?;
                Ok(before != e.scene)
            })?;
            Ok(None)
        }
        "operandi.acs.guideFit" => {
            let root = target_rig(&e.scene, a)?;
            let mesh = match id_arg(a, "mesh")? {
                Some(m) => m,
                None => e
                    .scene
                    .selected_items()
                    .into_iter()
                    .find(|&i| e.scene.item(i).is_some_and(|i| i.is_mesh()))
                    .ok_or("Select the character's mesh to fit the guides to")?,
            };
            if !e.scene.item(mesh).is_some_and(|i| i.is_mesh()) {
                return Err(format!("{mesh} isn't a mesh item"));
            }
            let embed = flag(a, "embed")?.unwrap_or(true);
            setup_step(e, root, "Fit Guides to Mesh", |e| {
                cr::fit_to_mesh(&mut e.scene, root, mesh, embed)?;
                Ok(true)
            })?;
            Ok(None)
        }
        "operandi.acs.build" => {
            let root = target_rig(&e.scene, a)?;
            setup_step(e, root, "Build Skeleton", |e| {
                let before = e.scene.clone();
                cr::build_skeleton(&mut e.scene, root)?;
                Ok(before != e.scene)
            })?;
            Ok(None)
        }
        "operandi.acs.bind" => {
            // ACS-41: Form context (Setup mode), then the shared Bind.
            let root = target_rig(&e.scene, a)?;
            let rig = e.scene.character_rig(root).ok_or(NO_RIG)?.clone();
            let joint = rig
                .root_joint()
                .filter(|j| e.scene.item(*j).is_some())
                .ok_or("Build the skeleton first (Rig tab ▸ Build Skeleton)")?;
            let meshes: Vec<u64> = match ids(a, "items")? {
                Some(m) => m,
                None => e
                    .scene
                    .selected_items()
                    .into_iter()
                    .filter(|&i| e.scene.item(i).is_some_and(|i| i.is_mesh()))
                    .collect(),
            };
            if meshes.is_empty() {
                return Err("Select the meshes to bind".into());
            }
            let mut args = a.clone();
            args.remove("rig");
            args.insert(
                "items".into(),
                Value::String(
                    meshes
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ),
            );
            args.insert("root".into(), json!(joint));
            let was = e.scene.character_rig(root).map(|c| c.context);
            if let Some(c) = e.scene.character_rig_mut(root) {
                c.context = RigContext::Form;
            }
            set_setup(e, true);
            let result = crate::skin_bind::run(e, "operandi.skin.bind", &args);
            if result.is_err()
                && let Some(c) = e.scene.character_rig_mut(root)
                && let Some(w) = was
            {
                c.context = w;
                set_setup(e, w.is_setup());
            }
            result
        }
        _ => Err(format!("Unknown character rig command {name}")),
    }
}

/// Guides deleted by hand come back (at the origin; placing follows).
fn restore_missing_guides(scene: &mut Scene, root: u64) -> Result<()> {
    let rig = scene.character_rig(root).ok_or(NO_RIG)?.clone();
    let mut guides = rig.guides.clone();
    for (role, parent, _) in cr::biped_guides() {
        if guides.iter().any(|p| p.role == role) {
            continue;
        }
        let parent_item = parent
            .and_then(|p| guides.iter().find(|g| g.role == p).map(|g| g.item))
            .unwrap_or(rig.guides_group);
        let id = cr::add_locator(
            scene,
            crate::locator::LocatorKind::Guide,
            &cr::guide_name(&role),
            Some(parent_item),
            DVec3::ZERO,
        )?;
        guides.push(crate::character_rig::Part { role, item: id });
    }
    // Parents first, in template order.
    let order: Vec<String> = cr::biped_guides().into_iter().map(|g| g.0).collect();
    guides.sort_by_key(|p| order.iter().position(|r| *r == p.role));
    scene.character_rig_mut(root).expect("checked").guides = guides;
    Ok(())
}

fn rig_json(scene: &Scene, c: &CharacterRig) -> Value {
    let part = |p: &crate::character_rig::Part| {
        let item = scene.item(p.item);
        json!({
            "role": p.role,
            "id": p.item,
            "name": item.map(|i| i.name.clone()),
            "side": side_of(&p.role).token(),
            "parent": item.and_then(|i| i.parent),
            "position": world_pos(scene, p.item).to_array().map(crate::rig::clean),
        })
    };
    let mut options = Map::new();
    for (field, _, _) in BipedOptions::FIELDS {
        options.insert(field.into(), json!(c.options.get(field)));
    }
    let modules: Vec<Value> = cr::biped_modules()
        .into_iter()
        .map(|(kind, side, opts)| {
            json!({
                "module": kind,
                "side": side.token(),
                "options": opts,
                "guides": c.guides.iter()
                    .filter(|p| p.role.starts_with(&format!("{kind}.{}.", side.token())))
                    .map(|p| p.item).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "root": c.root,
        "name": scene.item(c.root).map(|i| i.name.clone()),
        "preset": c.preset,
        "context": c.context.name(),
        "symmetry": c.symmetry,
        "referenceSize": c.reference_size,
        "options": options,
        "modules": modules,
        "guidesGroup": c.guides_group,
        "skeletonGroup": c.skeleton_group,
        "guides": c.guides.iter().map(part).collect::<Vec<_>>(),
        "joints": c.joints.iter().map(part).collect::<Vec<_>>(),
        "boundMeshes": cr::bound_meshes(scene, c),
    })
}

/// `operandi.acs.query`: the scene's rigs (or `rig`'s) as JSON.
pub fn answer(e: &Editor, name: &str, a: &Args) -> Option<Result<String>> {
    if name != "operandi.acs.query" {
        return None;
    }
    Some((|| {
        let rigs: Vec<Value> = match id_arg(a, "rig")? {
            Some(r) => vec![rig_json(
                &e.scene,
                e.scene
                    .character_rig(r)
                    .ok_or_else(|| format!("{r} isn't a character rig's root"))?,
            )],
            None => e
                .scene
                .character_rigs()
                .iter()
                .map(|c| rig_json(&e.scene, c))
                .collect(),
        };
        Ok(json!({ "rigs": rigs }).to_string())
    })())
}
