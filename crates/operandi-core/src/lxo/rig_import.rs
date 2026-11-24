//! LXO import: joints, skins, constraints, dynamic parents, channel links,
//! modifiers and two-bone IK (spec lxo-import-anim-rig-2026-11-24,
//! LXA-60..80), as Operandi joints, skin pieces and rig pieces.
//!
//! Modo keeps each rig piece as an item of its own, wired with item links
//! (`LINK`) and channel links (`CLNK`, stored on the driving item). Each
//! is matched to the Operandi piece that does the same job; kinds Operandi
//! has no piece for are left out and named in the import report.
use super::anim_import::target;
use super::phase1::{ChannelLink, Extras, RawItem, number, text};
use super::{Report, Result};
use crate::Scene;
use crate::animation::ChannelKey;
use crate::rig::Piece;
use std::collections::{BTreeMap, BTreeSet};

/// The Operandi id of a Modo item (phase1 numbers items this way).
fn oid(id: u32) -> u64 {
    u64::from(id) + 1
}

/// Modo rig item types this module reads (the rest of `cm*` is reported).
pub fn handled(kind: &str) -> bool {
    matches!(
        kind,
        "genInfluence"
            | "deformGroup"
            | "cmTransformConstraint"
            | "cmDirectionConstraint"
            | "cmDynamicParent"
            | "cmMatrixCompose"
            | "cmMathBasic"
            | "Expression"
            | "cmIKDual2D"
            | "cmIK2Bar3Bar"
            | "cmChannelRelation"
            | "cmPathConstraint"
            | "morphDeform"
    ) || quiet(kind)
}

/// LXA-170: schematic layout items, skipped without a report line.
fn quiet(kind: &str) -> bool {
    matches!(kind, "schmNode" | "elbow" | "backdropSchm" | "noteSchm")
}

/// One skin to build (LXA-61).
#[derive(Debug, Default)]
pub struct SkinPlan {
    pub mesh: u32,
    /// (joint, its weight map's name after the rename), in link order.
    pub joints: Vec<(u32, String)>,
    pub enabled: Vec<bool>,
    pub interpolation: u32,
    pub world: bool,
    pub folder_off: bool,
}

/// One two-bone IK chain (LXA-80).
#[derive(Debug)]
pub struct IkPlan {
    pub solver: u32,
    pub root: u32,
    /// The middle joint of a two-bone chain; `None` for a chain IK
    /// (three-bar planar IK, LXA-131).
    pub mid: Option<u32>,
    pub end: u32,
    pub goal: u32,
}

/// What the scene build needs to know before it makes items.
#[derive(Debug, Default)]
pub struct Plan {
    /// Locators imported as joints (skin joints and IK chains).
    pub joints: BTreeSet<u32>,
    /// Locators imported as IK goals.
    pub goals: BTreeSet<u32>,
    /// Per mesh: weight-map renames (`__item_locator031` → `j0`).
    pub map_names: BTreeMap<u32, BTreeMap<String, String>>,
    pub skins: Vec<SkinPlan>,
    pub ik: Vec<IkPlan>,
    /// Influences left out, by reason.
    pub left_out: BTreeMap<String, usize>,
}

/// The CLNK into `to`'s channel `channel`, if any.
fn input<'a>(extras: &'a Extras, to: u32, channel: &str) -> Option<&'a ChannelLink> {
    extras
        .clinks
        .iter()
        .find(|l| l.to == to && l.to_channel == channel)
}

/// LXA-60/61/80: joints, skins and IK chains, from the links alone.
pub fn plan(items: &BTreeMap<u32, RawItem>, extras: &Extras) -> Plan {
    let mut plan = Plan::default();
    let mut by_mesh: BTreeMap<u32, Vec<(u32, u32, &RawItem)>> = BTreeMap::new();
    for influence in items.values().filter(|i| i.kind == "genInfluence") {
        let Some((_, mesh, order)) = influence
            .links
            .iter()
            .find(|(g, t, _)| g == "deformers" && items.get(t).is_some_and(|m| m.kind == "mesh"))
        else {
            // LXA-170: it has no effect; nothing to report.
            continue;
        };
        if text(influence, "type").is_some_and(|t| t != "mapWeight") {
            *plan
                .left_out
                .entry(format!(
                    "joint influences of type {}",
                    text(influence, "type").unwrap_or_default()
                ))
                .or_default() += 1;
            continue;
        }
        let Some(joint) = items.values().find(|j| {
            j.kind == "locator"
                && j.links
                    .iter()
                    .any(|(g, t, _)| g == "deformers" && *t == influence.id)
        }) else {
            *plan
                .left_out
                .entry("joint influences with no joint".into())
                .or_default() += 1;
            continue;
        };
        by_mesh
            .entry(*mesh)
            .or_default()
            .push((*order, joint.id, influence));
    }
    for (mesh, mut list) in by_mesh {
        list.sort_by_key(|(order, _, _)| *order);
        let mut skin = SkinPlan {
            mesh,
            interpolation: crate::skin::LINEAR,
            world: true,
            ..Default::default()
        };
        let mut used: BTreeSet<String> = BTreeSet::new();
        let renames = plan.map_names.entry(mesh).or_default();
        for (_, joint, influence) in list {
            plan.joints.insert(joint);
            if skin.joints.iter().any(|(j, _)| *j == joint) {
                continue;
            }
            // LXA-61: the map takes the joint's name, made unique.
            let base = items[&joint].label();
            let base = if base.is_empty() {
                "Joint".to_owned()
            } else {
                base
            };
            let mut name = base.clone();
            let mut n = 2;
            while used.contains(&name) {
                name = format!("{base} ({n})");
                n += 1;
            }
            used.insert(name.clone());
            if let Some(old) = text(influence, "name") {
                renames.insert(old.to_owned(), name.clone());
            }
            skin.joints.push((joint, name));
            skin.enabled.push(number(influence, "enable") != Some(0.));
            // LXA-63: the Normalizing Folder's interpolation (`local` is
            // read as Linear, Q-LXA-06).
            if let Some(folder) = influence
                .link("deformTree")
                .and_then(|f| items.get(&f))
                .filter(|f| f.kind == "deformGroup")
            {
                if text(folder, "interpolation") == Some("dqs") {
                    skin.interpolation = crate::skin::DUAL_QUATERNION;
                }
                if number(folder, "world") == Some(0.) {
                    skin.world = false;
                }
                if number(folder, "enable") == Some(0.) {
                    skin.folder_off = true;
                }
            }
        }
        plan.skins.push(skin);
    }
    // LXA-80: two-bone IK.
    for solver in items.values().filter(|i| i.kind == "cmIKDual2D") {
        let root = input(extras, solver.id, "jointA").map(|l| l.from);
        let mid = input(extras, solver.id, "jointB").map(|l| l.from);
        let goal = input(extras, solver.id, "goalWPos")
            .map(|l| l.from)
            .or_else(|| solver.link("chanMods"));
        let end = mid.and_then(|m| {
            items
                .values()
                .find(|i| i.kind == "locator" && i.link("parent") == Some(m))
                .map(|i| i.id)
        });
        let chain = [root, mid, end, goal];
        let all = chain
            .iter()
            .all(|j| j.is_some_and(|j| items.get(&j).is_some_and(|i| i.kind == "locator")));
        if !all || items[&mid.unwrap()].link("parent") != root {
            *plan
                .left_out
                .entry("two-bone IK whose chain isn't three linked joints".into())
                .or_default() += 1;
            continue;
        }
        let [Some(root), Some(mid), Some(end), Some(goal)] = chain else {
            continue;
        };
        plan.joints.extend([root, mid, end]);
        plan.goals.insert(goal);
        plan.ik.push(IkPlan {
            solver: solver.id,
            root,
            mid: Some(mid),
            end,
            goal,
        });
    }
    // LXA-130/131: planar IK. Two bars are two-bone IK; three bars a
    // chain IK on the four joints.
    for solver in items.values().filter(|i| i.kind == "cmIK2Bar3Bar") {
        let joint = |c: &str| input(extras, solver.id, c).map(|l| l.from);
        let three = number(solver, "joints").unwrap_or(0.) >= 0.5;
        let (root, b, cj, end) = (
            joint("jointA"),
            joint("jointB"),
            joint("jointC"),
            joint("jointE"),
        );
        let goal = joint("goalWPos");
        let parent = |id: u32| items.get(&id).and_then(|i| i.link("parent"));
        let locator =
            |id: Option<u32>| id.filter(|j| items.get(j).is_some_and(|i| i.kind == "locator"));
        let chain: Option<Vec<u32>> = if three {
            [root, b, cj, end].iter().map(|j| locator(*j)).collect()
        } else {
            [root, b, end].iter().map(|j| locator(*j)).collect()
        };
        let linked = chain
            .as_ref()
            .is_some_and(|c| c.windows(2).all(|w| parent(w[1]) == Some(w[0])));
        let (Some(chain), Some(goal), true) = (chain, locator(goal), linked) else {
            *plan
                .left_out
                .entry("planar IK whose chain isn't linked joints".into())
                .or_default() += 1;
            continue;
        };
        plan.joints.extend(chain.iter().copied());
        plan.goals.insert(goal);
        plan.ik.push(IkPlan {
            solver: solver.id,
            root: chain[0],
            mid: (!three).then_some(chain[1]),
            end: *chain.last().unwrap(),
            goal,
        });
    }
    plan
}

/// Modo rig items made into pieces: piece id and Operandi kind, by the
/// Modo item's reference ID (anim_import keys their channels).
pub type Pieces = BTreeMap<u32, (u64, &'static str)>;

fn new_id(scene: &mut Scene) -> Result<u64> {
    if scene.next_id >= u64::MAX - 1 {
        return Err(super::Error::new(
            "resource_limit",
            0,
            "Scene item/identity budget exhausted",
        ));
    }
    let id = scene.next_id;
    scene.next_id += 1;
    Ok(id)
}

fn xyz(item: &RawItem, base: &str, default: f64) -> [f64; 3] {
    ["X", "Y", "Z"].map(|a| number(item, &format!("{base}.{a}")).unwrap_or(default))
}

fn set_group(p: &mut Piece, group: &str, v: [f64; 3]) {
    for (i, a) in ["X", "Y", "Z"].iter().enumerate() {
        let _ = p.set(&format!("{group}{a}"), v[i]);
    }
}

/// A modifier's input `channel`: the channel wired into it, or its own
/// constant value.
enum Input {
    Wired(ChannelKey),
    Constant(f64),
}

fn modifier_input(
    items: &BTreeMap<u32, RawItem>,
    extras: &Extras,
    m: &RawItem,
    channel: &str,
) -> Input {
    input(extras, m.id, channel)
        .and_then(|l| target(items, None, l.from, &l.from_channel))
        .map_or_else(
            || Input::Constant(number(m, channel).unwrap_or(0.)),
            Input::Wired,
        )
}

/// Replaces each whole-word input name in `expr` that has a constant
/// value with that number (Operandi's unwired inputs read as nothing).
fn substitute(expr: &str, constants: &BTreeMap<&str, f64>) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        match constants.get(word.as_str()) {
            Some(v) => out.push_str(&format!("({v})")),
            None => out.push_str(word),
        }
        word.clear();
    };
    for ch in expr.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            flush(&mut word, &mut out);
            out.push(ch);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// Every channel a modifier's `output` drives.
fn outputs(items: &BTreeMap<u32, RawItem>, extras: &Extras, m: u32) -> Vec<ChannelKey> {
    extras
        .clinks
        .iter()
        .filter(|l| l.from == m && l.from_channel == "output")
        .filter_map(|l| target(items, None, l.to, &l.to_channel))
        .collect()
}

/// `kind` as the static name of a rig piece kind.
fn kind_name(kind: &str) -> &'static str {
    crate::rig::KINDS
        .iter()
        .find(|k| **k == kind)
        .copied()
        .unwrap_or("position")
}

/// Makes the rig pieces (LXA-70..80) and the skins (LXA-61).
pub fn pieces(
    scene: &mut Scene,
    items: &BTreeMap<u32, RawItem>,
    extras: &Extras,
    plan: &Plan,
    report: &mut Report,
) -> Result<Pieces> {
    let mut map = Pieces::new();
    let exists = |scene: &Scene, id: u32| scene.item(oid(id)).is_some();
    let mut stats: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut left: BTreeMap<String, usize> = plan.left_out.clone();
    let mut leave = |what: String| *left.entry(what).or_default() += 1;

    // Skins (LXA-61): one skin piece per mesh.
    for skin in &plan.skins {
        if !exists(scene, skin.mesh) || skin.joints.iter().any(|(j, _)| !exists(scene, *j)) {
            leave("skins whose mesh or joints aren't imported".into());
            continue;
        }
        let mut values = crate::skin::SkinValues {
            interpolation: skin.interpolation,
            world: skin.world,
            ..Default::default()
        };
        for (k, (joint, name)) in skin.joints.iter().enumerate() {
            values.add(oid(*joint), name);
            values.enabled[k] = skin.enabled[k];
        }
        crate::skin_bind::insert_skin(scene, oid(skin.mesh), &values)
            .map_err(|e| super::Error::new("invalid_candidate", 0, &e))?;
        *stats.entry("skins").or_default() += 1;
        if skin.folder_off {
            report
                .notes
                .push("A disabled Normalizing Folder was imported on (Operandi always normalizes skin weights)".into());
        }
    }

    // Constraints and dynamic parents (LXA-72/73), per constrained item in
    // the file's order.
    let mut per_item: BTreeMap<u32, Vec<(u32, Piece, u32)>> = BTreeMap::new();
    for c in items.values() {
        let kind = match (c.kind.as_str(), text(c, "operation")) {
            ("cmTransformConstraint", Some("rot")) => "rotation",
            ("cmTransformConstraint", Some("scl")) => "scale",
            ("cmTransformConstraint", _) => "position",
            ("cmDirectionConstraint", _) => "direction",
            ("cmDynamicParent", _) => "dynamicParent",
            _ => continue,
        };
        let Some((_, item, order)) = c
            .links
            .iter()
            .find(|(g, _, _)| g == "chanMods")
            .map(|(g, t, o)| (g, *t, *o))
        else {
            leave(format!("{kind} constraints with nothing to constrain"));
            continue;
        };
        if !exists(scene, item) {
            leave(format!("{kind} constraints on items not imported"));
            continue;
        }
        let targets: Vec<u32> = if kind == "dynamicParent" {
            // LXA-73: the parent is the item the matrix compose reads.
            input(extras, c.id, "matrixInput")
                .and_then(|l| items.get(&l.from))
                .filter(|m| m.kind == "cmMatrixCompose")
                .and_then(|m| m.link("chanMods"))
                .into_iter()
                .collect()
        } else {
            let mut t: Vec<&ChannelLink> = extras
                .clinks
                .iter()
                .filter(|l| {
                    l.to == c.id && l.to_channel.starts_with("matrixInput") && l.from != item
                })
                .collect();
            t.sort_by(|a, b| a.to_channel.cmp(&b.to_channel));
            let mut ids: Vec<u32> = t.iter().map(|l| l.from).collect();
            ids.dedup();
            ids
        };
        if targets.is_empty() || targets.iter().any(|t| !exists(scene, *t)) {
            leave(format!("{kind} constraints whose targets aren't imported"));
            continue;
        }
        let mut p = Piece::new(
            0,
            kind,
            oid(item),
            targets.iter().map(|t| oid(*t)).collect(),
        );
        let _ = p.set("enable", number(c, "enable").unwrap_or(1.));
        // Per-target weights (`weight.<UNIQ>`, wired from each target).
        for (i, t) in targets.iter().enumerate() {
            if let Some(w) = extras
                .clinks
                .iter()
                .find(|l| l.graph == "weightLinks" && l.from == *t && l.to == c.id)
                .and_then(|l| number(c, &l.to_channel))
            {
                let _ = p.set(&format!("weight.{i}"), w);
            }
        }
        let offset = xyz(c, "offset", 0.);
        match kind {
            "position" => set_group(&mut p, "offset.pos.", offset),
            "rotation" => set_group(&mut p, "offset.rot.", offset),
            // A scale constraint stores no offset as 0 (provisional: an
            // offset adds to a scale of 1, LXO-AR-Q01).
            "scale" => set_group(&mut p, "offset.scl.", offset.map(|v| 1. + v)),
            "direction" => {
                set_group(&mut p, "offset.rot.", offset);
                let axis = match text(c, "axis") {
                    Some("x") => 0.,
                    Some("y") => 1.,
                    _ => 2.,
                };
                let _ = p.set("axis", axis);
                let _ = p.set("negative", number(c, "axisNeg").unwrap_or(0.));
                let _ = p.set("roll", number(c, "roll").unwrap_or(0.));
                set_group(&mut p, "up.", xyz(c, "up", 0.));
                if xyz(c, "up", 0.) == [0.; 3] {
                    let _ = p.set("up.Y", 1.);
                }
            }
            _ => {
                let _ = p.set(
                    "parent",
                    number(c, "parent").unwrap_or(1.).round().clamp(0., 1.),
                );
                set_group(&mut p, "offset.pos.", xyz(c, "offsetPos", 0.));
                set_group(&mut p, "offset.rot.", xyz(c, "offsetRot", 0.));
                set_group(&mut p, "offset.scl.", xyz(c, "offsetScl", 1.));
            }
        }
        per_item.entry(item).or_default().push((order, p, c.id));
    }
    for (_, mut list) in per_item {
        list.sort_by_key(|(order, _, _)| *order);
        for (_, mut p, modo) in list {
            p.id = new_id(scene)?;
            let kind: &'static str = crate::rig::KINDS
                .iter()
                .find(|k| **k == p.kind)
                .copied()
                .unwrap_or("position");
            map.insert(modo, (p.id, kind));
            *stats
                .entry(if kind == "dynamicParent" {
                    "dynamic parents"
                } else {
                    "constraints"
                })
                .or_default() += 1;
            scene.animation.rig.pieces.push(p);
        }
    }

    // Modifiers (LXA-74).
    for m in items.values() {
        let (kind, expr, inputs): (&str, String, Vec<(&str, &str)>) = match m.kind.as_str() {
            "cmMathBasic" => {
                let op = text(m, "operation").unwrap_or("add");
                let symbol = match op {
                    "add" => "A + B",
                    "sub" => "A - B",
                    "mul" => "A * B",
                    "div" => "A / B",
                    "min" => "min(A, B)",
                    "max" => "max(A, B)",
                    other => {
                        leave(format!("math modifiers with operation {other}"));
                        continue;
                    }
                };
                (
                    "mod.math",
                    symbol.into(),
                    vec![("A", "input1"), ("B", "input2")],
                )
            }
            "Expression" => (
                "mod.expression",
                text(m, "expression").unwrap_or_default().to_owned(),
                vec![
                    ("A", "inputA"),
                    ("B", "inputB"),
                    ("C", "inputC"),
                    ("D", "inputD"),
                ],
            ),
            _ => continue,
        };
        let outs = outputs(items, extras, m.id);
        if outs.is_empty() {
            leave("modifiers that drive nothing".into());
            continue;
        }
        let mut wired: Vec<(&str, ChannelKey)> = Vec::new();
        let mut constants: BTreeMap<&str, f64> = BTreeMap::new();
        for (name, channel) in &inputs {
            match modifier_input(items, extras, m, channel) {
                Input::Wired(k) => wired.push((name, k)),
                Input::Constant(v) => {
                    constants.insert(name, v);
                }
            }
        }
        // Operandi's Math reads two wired inputs; with a constant input it
        // becomes the same sum as an Expression.
        let (kind, expr) = if kind == "mod.math" && constants.is_empty() {
            ("mod.math", None)
        } else {
            ("mod.expression", Some(substitute(&expr, &constants)))
        };
        if let Some(e) = &expr
            && crate::rig_expression::Expr::parse(e).is_err()
        {
            leave("expressions Operandi can't read".into());
            continue;
        }
        for out in outs {
            if scene.item(out.item).is_none() {
                leave("modifiers driving items not imported".into());
                continue;
            }
            let mut p = Piece::new(new_id(scene)?, kind, out.item, Vec::new());
            let _ = p.set("enable", number(m, "enable").unwrap_or(1.));
            p.names.insert("driven".into(), out.channel.clone());
            for (name, k) in &wired {
                p.names.insert(format!("in.{name}"), k.text());
            }
            match &expr {
                Some(e) => {
                    p.names.insert("expr".into(), e.clone());
                }
                None => {
                    let op = match text(m, "operation") {
                        Some("sub") => "subtract",
                        Some("mul") => "multiply",
                        Some("div") => "divide",
                        Some("min") => "min",
                        Some("max") => "max",
                        _ => "add",
                    };
                    p.names.insert("op".into(), op.into());
                }
            }
            map.insert(
                m.id,
                (
                    p.id,
                    if kind == "mod.math" {
                        "mod.math"
                    } else {
                        "mod.expression"
                    },
                ),
            );
            scene.animation.rig.pieces.push(p);
            *stats.entry("modifiers").or_default() += 1;
        }
    }

    // Direct channel links (LXA-71): chanLinks between two ordinary
    // channels.
    let rig_item = |id: u32| {
        items
            .get(&id)
            .is_some_and(|i| handled(&i.kind) || i.kind.starts_with("cm"))
    };
    for l in extras.clinks.iter().filter(|l| l.graph == "chanLinks") {
        if rig_item(l.from) || rig_item(l.to) {
            continue;
        }
        if l.from_channel.contains("Matrix") || l.to_channel.contains("Matrix") {
            continue;
        }
        let (Some(from), Some(to)) = (
            target(items, None, l.from, &l.from_channel),
            target(items, None, l.to, &l.to_channel),
        ) else {
            leave("channel links between channels Operandi doesn't have".into());
            continue;
        };
        if scene.item(from.item).is_none() || scene.item(to.item).is_none() {
            leave("channel links on items not imported".into());
            continue;
        }
        // A link carries the value as Modo shows it: a distance of 1.5
        // drives a rotation of 1.5° (observed in constraints.lxo,
        // provisional LXO-AR-Q03), so a link between a rotation and
        // another channel becomes an Expression that converts.
        let angle = |c: &str| c.starts_with("rot.");
        let factor = match (angle(&from.channel), angle(&to.channel)) {
            (false, true) => Some(std::f64::consts::PI / 180.),
            (true, false) => Some(180. / std::f64::consts::PI),
            _ => None,
        };
        let mut p = match factor {
            None => {
                let mut p = Piece::new(new_id(scene)?, "link", to.item, vec![from.item]);
                p.names.insert("driver".into(), from.channel);
                p.names.insert("mode".into(), "direct".into());
                p
            }
            Some(f) => {
                let mut p = Piece::new(new_id(scene)?, "mod.expression", to.item, Vec::new());
                p.names.insert("in.A".into(), from.text());
                p.names.insert("expr".into(), format!("A * {f}"));
                p
            }
        };
        p.names.insert("driven".into(), to.channel);
        scene.animation.rig.pieces.push(p);
        *stats.entry("channel links").or_default() += 1;
    }

    // Two-bone IK (LXA-80); the bend hint needs the setup pose (`finish`).
    for ik in &plan.ik {
        if [ik.root, ik.end, ik.goal]
            .iter()
            .chain(ik.mid.iter())
            .any(|j| !exists(scene, *j))
        {
            leave("IK on items not imported".into());
            continue;
        }
        let kind = if ik.mid.is_some() {
            crate::ik::TWO_BONE
        } else {
            crate::ik::CHAIN
        };
        let mut p = Piece::new(new_id(scene)?, kind, oid(ik.root), vec![oid(ik.goal)]);
        if let Some(mid) = ik.mid {
            p.names.insert("mid".into(), oid(mid).to_string());
        }
        p.names.insert("end".into(), oid(ik.end).to_string());
        if let Some(b) = items.get(&ik.solver).and_then(|s| number(s, "blend")) {
            let _ = p.set("blend", b);
        }
        map.insert(ik.solver, (p.id, kind));
        scene.animation.rig.pieces.push(p);
        if kind == crate::ik::CHAIN {
            // LXA-131: Operandi's chain solver isn't the planar three-bar
            // one, and the bend share (weightA) has no setting.
            *stats
                .entry("three-bar planar IK (as chain IK)")
                .or_default() += 1;
        } else {
            *stats.entry("two-bone IK").or_default() += 1;
        }
    }

    // Channel Relationships (LXA-110..112): links in relationship mode.
    for r in items.values().filter(|i| i.kind == "cmChannelRelation") {
        let driver =
            input(extras, r.id, "input").and_then(|l| target(items, None, l.from, &l.from_channel));
        let env = extras
            .actions
            .setup
            .grads
            .get(&r.id)
            .and_then(|g| g.get("value"))
            .or_else(|| r.grads.get("value"))
            .and_then(|e| extras.envelopes.get(e));
        let (Some(driver), Some(env)) = (driver, env) else {
            leave("channel relationships with no driver or curve".into());
            continue;
        };
        let (mut curve, _) = super::envelope::to_value_curve(env);
        let offset = number(r, "offset").unwrap_or(0.);
        for k in &mut curve.keys {
            k.value += offset;
        }
        let outs = outputs(items, extras, r.id);
        if outs.is_empty() || scene.item(driver.item).is_none() {
            leave("channel relationships that drive nothing".into());
            continue;
        }
        for out in outs {
            if scene.item(out.item).is_none() {
                continue;
            }
            let mut p = Piece::new(new_id(scene)?, "link", out.item, vec![driver.item]);
            let _ = p.set("enable", number(r, "enable").unwrap_or(1.));
            p.names.insert("driven".into(), out.channel.clone());
            p.names.insert("driver".into(), driver.channel.clone());
            p.names.insert("mode".into(), "relationship".into());
            p.curve = Some(curve.clone());
            map.insert(r.id, (p.id, "link"));
            scene.animation.rig.pieces.push(p);
            *stats.entry("channel relationships").or_default() += 1;
        }
    }

    // Path constraints (LXA-140).
    for c in items.values().filter(|i| i.kind == "cmPathConstraint") {
        let kind = match text(c, "operation") {
            Some("curveNormal") => "pathNormal",
            Some("curvePath") => "path",
            _ => "pathPosition",
        };
        let item = c.link("chanMods");
        let curve = input(extras, c.id, "mesh").map(|l| l.from);
        let (Some(item), Some(curve)) = (item, curve) else {
            leave("path constraints with no item or curve".into());
            continue;
        };
        if !exists(scene, item) || !exists(scene, curve) {
            leave("path constraints on items not imported".into());
            continue;
        }
        let mut p = Piece::new(new_id(scene)?, kind, oid(item), vec![oid(curve)]);
        let _ = p.set("enable", number(c, "enable").unwrap_or(1.));
        for (from, to) in [
            ("percent", "percent"),
            ("offset", "pathOffset"),
            ("wrap", "wrap"),
        ] {
            if let Some(v) = number(c, from) {
                let _ = p.set(to, v);
            }
        }
        if kind != "pathPosition" {
            let axis = match text(c, "axis") {
                Some("x") => 0.,
                Some("y") => 1.,
                _ => 2.,
            };
            let _ = p.set("axis", axis);
            let _ = p.set("negative", number(c, "axisNeg").unwrap_or(0.));
            let _ = p.set("roll", number(c, "roll").unwrap_or(0.));
            let _ = p.set("localUp", number(c, "local").unwrap_or(0.));
            set_group(&mut p, "up.", xyz(c, "up", 0.));
            set_group(&mut p, "offset.rot.", xyz(c, "offsetRot", 0.));
        }
        set_group(&mut p, "offset.pos.", xyz(c, "offsetPos", 0.));
        map.insert(c.id, (p.id, kind_name(kind)));
        scene.animation.rig.pieces.push(p);
        *stats.entry("constraints").or_default() += 1;
    }

    // Everything else of Modo's rig library (GAPS 2, 3, 8).
    for i in items.values() {
        if i.kind == "morphDeform" {
            leave("morph influences (animated morph amounts)".into());
        } else if i.kind == "cmMatrixBlend" {
            leave("matrix blends".into());
        } else if i.kind.starts_with("cm") && !handled(&i.kind) {
            leave(format!("{} rig items", i.kind));
        }
    }
    if crate::rig_eval::would_loop(scene).is_some() {
        report.notes.push(
            "Some imported rig pieces form a loop; they evaluate as off until it is broken".into(),
        );
    }
    if !stats.is_empty() {
        report.notes.push(format!(
            "Rig imported: {}",
            stats
                .iter()
                .map(|(k, v)| format!("{v} {k}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for (what, n) in left {
        report.omissions.insert(format!("{what} not imported"), n);
    }
    Ok(map)
}

/// What needs the imported setup pose: each joint keeps its setup values
/// (SK-23), and each two-bone IK piece takes its bend hint from the setup
/// chain (LXA-80).
pub fn finish(scene: &mut Scene, plan: &Plan, report: &mut Report) {
    for j in &plan.joints {
        crate::skin::ensure_setup_channels(scene, oid(*j));
    }
    for ik in &plan.ik {
        let Some(mid) = ik.mid else {
            // Chain IK: the goal's setup rotation is its twist reference.
            let twist = crate::skin::setup_world(scene, oid(ik.goal))
                .map(|m| {
                    m.to_scale_rotation_translation()
                        .1
                        .to_euler(glam::EulerRot::XYZ)
                })
                .unwrap_or_default();
            if let Some(p) = scene
                .animation
                .rig
                .pieces
                .iter_mut()
                .find(|p| p.kind == crate::ik::CHAIN && p.item == oid(ik.root))
            {
                for (axis, v) in ["X", "Y", "Z"].iter().zip([twist.0, twist.1, twist.2]) {
                    p.values
                        .insert(format!("twist.{axis}"), crate::rig::clean(v));
                }
            }
            continue;
        };
        let pos = |s: &Scene, id: u32| {
            crate::skin::setup_world(s, oid(id))
                .map(|m| m.w_axis.truncate())
                .unwrap_or_default()
        };
        let parent = scene
            .item(oid(ik.root))
            .and_then(|i| i.parent)
            .and_then(|p| crate::skin::setup_world(scene, p).ok())
            .unwrap_or(glam::DMat4::IDENTITY);
        let bend = crate::ik::rest_bend(
            pos(scene, ik.root),
            pos(scene, mid),
            pos(scene, ik.end),
            parent,
        );
        let Some(p) = scene
            .animation
            .rig
            .pieces
            .iter_mut()
            .find(|p| p.kind == crate::ik::TWO_BONE && p.item == oid(ik.root))
        else {
            continue;
        };
        match bend {
            Some(b) => {
                for (k, axis) in ["X", "Y", "Z"].iter().enumerate() {
                    p.values
                        .insert(format!("bend.{axis}"), crate::rig::clean(b[k]));
                }
            }
            None => report.notes.push(
                "A two-bone IK chain is straight in its setup pose; Operandi picks its bend".into(),
            ),
        }
    }
}
