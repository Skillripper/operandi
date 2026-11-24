//! Character rigs, first milestone (BEHAVIOR-ACS-FEATURES, ACS-01..44):
//! a Biped rig whose **guides** you fit to a character, the **bind
//! skeleton** built from them, and meshes bound to that skeleton.
//!
//! The ideas come from Auto Character System 3 by Lukasz Pazera (MIT,
//! github.com/lukpazera/AutoCharacterSystem); this is our own code, built
//! natively from the behaviour spec, with no ACS code copied.
//!
//! A rig is one root item (a Group Locator) holding a "Guides" group and,
//! once built, a "Skeleton" group (ACS-10). What the rig is made of lives
//! here, in `Rig::characters`: its preset, context, options and which
//! item plays which part ("role"). Guides and joints are ordinary items,
//! so moving, parenting, undo and files work on them unchanged.
//!
//! Roles are `module.side.piece` (ACS-20): `leg.L.knee`, `spine.C.hips`.
//! The Biped's modules and their insides are our own design for now
//! (Jason, 2026-11-23: builders design piece internals until spec v2).
//!
//! PROVISIONAL (ACSF-Q01..Q06 in docs/SPECIFICATION_QUESTIONS.md): which
//! items each context shows, which contexts are Setup mode, how guide
//! symmetry picks the side that leads, module options shared by both
//! sides, twist joints as in-line joints along the bone, and how Fit to
//! Mesh centres each guide.
use crate::locator::LocatorKind;
use crate::scene::{Scene, Transform};
use crate::{Result, rig::clean};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};

/// The name of the rig root's Guides group and Skeleton group.
pub const GUIDES_GROUP: &str = "Guides";
pub const SKELETON_GROUP: &str = "Skeleton";

/// ACS-01: what the rig shows. Saved with the scene, not an undo step
/// (ACS-02, UIC-225).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RigContext {
    Assembly,
    #[default]
    Guide,
    Meshes,
    Form,
    Animate,
}

impl RigContext {
    pub const ALL: [RigContext; 5] = [
        RigContext::Assembly,
        RigContext::Guide,
        RigContext::Meshes,
        RigContext::Form,
        RigContext::Animate,
    ];
    /// The name shown in menus (ACS-01 "Shown name").
    pub fn label(self) -> &'static str {
        match self {
            RigContext::Assembly => "Assemble",
            RigContext::Guide => "Guide",
            RigContext::Meshes => "Meshes",
            RigContext::Form => "Form",
            RigContext::Animate => "Animate",
        }
    }
    /// The script name.
    pub fn name(self) -> &'static str {
        match self {
            RigContext::Assembly => "assembly",
            RigContext::Guide => "guide",
            RigContext::Meshes => "meshes",
            RigContext::Form => "form",
            RigContext::Animate => "animate",
        }
    }
    pub fn parse(s: &str) -> Result<RigContext> {
        let s = s.trim().to_ascii_lowercase();
        RigContext::ALL
            .into_iter()
            .find(|c| c.name() == s || c.label().to_ascii_lowercase() == s)
            .ok_or_else(|| format!("context is assembly, guide, meshes, form or animate, not {s}"))
    }
    /// Guides show only in the Guide context (ACS-30).
    pub fn shows_guides(self) -> bool {
        self == RigContext::Guide
    }
    /// The bind skeleton shows everywhere but the Guide context.
    /// PROVISIONAL (ACS-Q1): Animate shows it too until controllers exist.
    pub fn shows_skeleton(self) -> bool {
        self != RigContext::Guide
    }
    /// ACS-02: contexts that put the scene in its setup (rest) state.
    /// PROVISIONAL (ACS-Q2): every context but Animate.
    pub fn is_setup(self) -> bool {
        self != RigContext::Animate
    }
}

/// ACS-22 options of the Biped's modules. PROVISIONAL (ACS-Q4): both
/// sides of a mirrored pair share one set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct BipedOptions {
    /// Spine: vertebrae between the hips and the chest (1..=8).
    pub spine: u32,
    /// Head: neck joints (1..=4).
    pub neck: u32,
    /// Biped Arm / Leg: twist joints per bone (0..=4, ACS-23).
    pub upper_arm_twist: u32,
    pub forearm_twist: u32,
    pub thigh_twist: u32,
    pub shin_twist: u32,
}

impl Default for BipedOptions {
    fn default() -> Self {
        BipedOptions {
            spine: 3,
            neck: 1,
            upper_arm_twist: 0,
            forearm_twist: 0,
            thigh_twist: 0,
            shin_twist: 0,
        }
    }
}

impl BipedOptions {
    /// (script name, range) of every option, in the order shown.
    pub const FIELDS: [(&'static str, u32, u32); 6] = [
        ("spine", 1, 8),
        ("neck", 1, 4),
        ("upperArmTwist", 0, 4),
        ("forearmTwist", 0, 4),
        ("thighTwist", 0, 4),
        ("shinTwist", 0, 4),
    ];
    pub fn get(&self, name: &str) -> Option<u32> {
        Some(match name {
            "spine" => self.spine,
            "neck" => self.neck,
            "upperArmTwist" => self.upper_arm_twist,
            "forearmTwist" => self.forearm_twist,
            "thighTwist" => self.thigh_twist,
            "shinTwist" => self.shin_twist,
            _ => return None,
        })
    }
    pub fn set(&mut self, name: &str, v: u32) -> Result<()> {
        let (_, lo, hi) = Self::FIELDS
            .iter()
            .find(|f| f.0 == name)
            .copied()
            .ok_or_else(|| format!("No option {name}"))?;
        if !(lo..=hi).contains(&v) {
            return Err(format!("{name} is {lo} to {hi}"));
        }
        let slot = match name {
            "spine" => &mut self.spine,
            "neck" => &mut self.neck,
            "upperArmTwist" => &mut self.upper_arm_twist,
            "forearmTwist" => &mut self.forearm_twist,
            "thighTwist" => &mut self.thigh_twist,
            _ => &mut self.shin_twist,
        };
        *slot = v;
        Ok(())
    }
}

/// Which item plays a role in the rig.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub role: String,
    pub item: u64,
}

/// One character rig (ACS-10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CharacterRig {
    /// The rig's root item (a Group Locator).
    pub root: u64,
    /// The preset it was made from (ACS-12): "biped".
    pub preset: String,
    #[serde(default)]
    pub context: RigContext,
    /// ACS-31 Guide symmetry: a guide's twin on the other side follows it.
    #[serde(default = "yes")]
    pub symmetry: bool,
    /// ACS-11 Reference size: the character's height, in metres.
    pub reference_size: f64,
    #[serde(default)]
    pub options: BipedOptions,
    /// The Guides group under the root.
    pub guides_group: u64,
    /// The Skeleton group, once the skeleton is built.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skeleton_group: Option<u64>,
    /// Guides, parents before children (ACS-30).
    pub guides: Vec<Part>,
    /// The bind skeleton's joints, parents before children (ACS-40).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joints: Vec<Part>,
}

fn yes() -> bool {
    true
}

impl CharacterRig {
    pub fn guide(&self, role: &str) -> Option<u64> {
        self.guides.iter().find(|p| p.role == role).map(|p| p.item)
    }
    pub fn joint(&self, role: &str) -> Option<u64> {
        self.joints.iter().find(|p| p.role == role).map(|p| p.item)
    }
    pub fn role_of(&self, item: u64) -> Option<&str> {
        self.guides
            .iter()
            .chain(&self.joints)
            .find(|p| p.item == item)
            .map(|p| p.role.as_str())
    }
    /// Whether `item` is one of this rig's items (root, groups, guides,
    /// joints).
    pub fn owns(&self, item: u64) -> bool {
        item == self.root
            || item == self.guides_group
            || self.skeleton_group == Some(item)
            || self.role_of(item).is_some()
    }
    /// The skeleton's root joint (the hips).
    pub fn root_joint(&self) -> Option<u64> {
        self.joints.first().map(|p| p.item)
    }
}

// ------------------------------------------------------------ the biped

/// A rig side (ACS-20).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Centre,
    Left,
    Right,
}

impl Side {
    pub fn token(self) -> &'static str {
        match self {
            Side::Centre => "C",
            Side::Left => "L",
            Side::Right => "R",
        }
    }
    fn suffix(self) -> &'static str {
        match self {
            Side::Centre => "",
            Side::Left => "_L",
            Side::Right => "_R",
        }
    }
    fn other(self) -> Side {
        match self {
            Side::Centre => Side::Centre,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

/// The side named in a role.
pub fn side_of(role: &str) -> Side {
    match role.split('.').nth(1) {
        Some("L") => Side::Left,
        Some("R") => Side::Right,
        _ => Side::Centre,
    }
}

/// The same role on the other side (a centre role is its own twin).
pub fn twin_role(role: &str) -> String {
    let mut parts: Vec<&str> = role.split('.').collect();
    if parts.len() == 3 {
        parts[1] = match parts[1] {
            "L" => "R",
            "R" => "L",
            s => s,
        };
    }
    parts.join(".")
}

/// One Biped guide: module, piece, parent guide (same side, else centre)
/// and where it sits as a fraction of the reference size, feet at the
/// root, facing +Z, the character's left on +X (a T pose).
struct GuideDef {
    module: &'static str,
    piece: &'static str,
    /// A centre guide when false; a left guide (mirrored to the right).
    sided: bool,
    parent: Option<&'static str>,
    at: [f64; 3],
}

const fn g(
    module: &'static str,
    piece: &'static str,
    sided: bool,
    parent: Option<&'static str>,
    at: [f64; 3],
) -> GuideDef {
    GuideDef {
        module,
        piece,
        sided,
        parent,
        at,
    }
}

/// The Biped's guides, parents first. Our own proportions (an adult of
/// average build), not taken from any preset.
const BIPED: [GuideDef; 15] = [
    g("spine", "hips", false, None, [0., 0.53, 0.]),
    g("spine", "chest", false, Some("spine.hips"), [0., 0.72, 0.]),
    g(
        "head",
        "neck",
        false,
        Some("spine.chest"),
        [0., 0.83, -0.005],
    ),
    g("head", "head", false, Some("head.neck"), [0., 0.875, 0.01]),
    g("head", "top", false, Some("head.head"), [0., 1., 0.01]),
    g(
        "arm",
        "clavicle",
        true,
        Some("spine.chest"),
        [0.02, 0.81, 0.015],
    ),
    g(
        "arm",
        "shoulder",
        true,
        Some("arm.clavicle"),
        [0.1, 0.815, 0.],
    ),
    g(
        "arm",
        "elbow",
        true,
        Some("arm.shoulder"),
        [0.25, 0.815, -0.012],
    ),
    g("arm", "wrist", true, Some("arm.elbow"), [0.39, 0.815, 0.]),
    g("arm", "hand", true, Some("arm.wrist"), [0.48, 0.815, 0.]),
    g("leg", "thigh", true, Some("spine.hips"), [0.055, 0.5, 0.]),
    g("leg", "knee", true, Some("leg.thigh"), [0.06, 0.28, 0.012]),
    g("leg", "ankle", true, Some("leg.knee"), [0.065, 0.05, -0.01]),
    g(
        "leg",
        "ball",
        true,
        Some("leg.ankle"),
        [0.065, 0.012, 0.075],
    ),
    g("leg", "toe", true, Some("leg.ball"), [0.065, 0.012, 0.13]),
];

/// The Biped's modules (ACS-20, ACS-22): kind, side and their options.
pub fn biped_modules() -> Vec<(&'static str, Side, &'static [&'static str])> {
    vec![
        ("spine", Side::Centre, &["spine"]),
        ("head", Side::Centre, &["neck"]),
        ("arm", Side::Left, &["upperArmTwist", "forearmTwist"]),
        ("arm", Side::Right, &["upperArmTwist", "forearmTwist"]),
        ("leg", Side::Left, &["thighTwist", "shinTwist"]),
        ("leg", Side::Right, &["thighTwist", "shinTwist"]),
    ]
}

fn role(module: &str, side: Side, piece: &str) -> String {
    format!("{module}.{}.{piece}", side.token())
}

fn title(piece: &str) -> String {
    let mut c = piece.chars();
    match c.next() {
        Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
        None => String::new(),
    }
}

/// Every Biped guide role with its parent role and template position
/// (fractions), parents first, each left guide followed by its right.
pub fn biped_guides() -> Vec<(String, Option<String>, DVec3)> {
    let mut out = Vec::new();
    for d in &BIPED {
        let sides: &[Side] = if d.sided {
            &[Side::Left, Side::Right]
        } else {
            &[Side::Centre]
        };
        for &side in sides {
            let parent = d.parent.map(|p| {
                let (m, piece) = p.split_once('.').expect("module.piece");
                let parent_sided = BIPED
                    .iter()
                    .any(|e| e.module == m && e.piece == piece && e.sided);
                role(m, if parent_sided { side } else { Side::Centre }, piece)
            });
            let mut at = DVec3::from_array(d.at);
            if side == Side::Right {
                at.x = -at.x;
            }
            out.push((role(d.module, side, d.piece), parent, at));
        }
    }
    out
}

/// A guide item's name.
pub fn guide_name(role: &str) -> String {
    let mut parts = role.split('.');
    let (_, side, piece) = (parts.next(), parts.next(), parts.next().unwrap_or(""));
    let side = match side {
        Some("L") => Side::Left,
        Some("R") => Side::Right,
        _ => Side::Centre,
    };
    format!("Guide {}{}", title(piece), side.suffix())
}

/// One joint of the bind skeleton a build makes.
#[derive(Clone, Debug, PartialEq)]
pub struct JointPlan {
    pub role: String,
    pub name: String,
    pub parent: Option<String>,
    pub at: DVec3,
}

/// The bind skeleton for these options from guide positions (`at` gives
/// a guide role's world position). Parents first. Serial pieces (ACS-25)
/// sit evenly along the line between two guides; twist joints are joints
/// in line along their bone (PROVISIONAL, ACS-Q5).
pub fn biped_joints(
    o: &BipedOptions,
    at: &dyn Fn(&str) -> Result<DVec3>,
) -> Result<Vec<JointPlan>> {
    let mut out: Vec<JointPlan> = Vec::new();
    let mut push = |role: String, name: String, parent: Option<String>, p: DVec3| {
        out.push(JointPlan {
            role,
            name,
            parent,
            at: p,
        });
        out.last().expect("pushed").role.clone()
    };
    let c = Side::Centre;
    let hips = push(
        role("spine", c, "hips"),
        "Hips".into(),
        None,
        at("spine.C.hips")?,
    );
    // Vertebrae between the hips and the chest.
    let (h, ch) = (at("spine.C.hips")?, at("spine.C.chest")?);
    let mut last = hips.clone();
    for i in 1..=o.spine {
        let t = i as f64 / (o.spine + 1) as f64;
        last = push(
            role("spine", c, &format!("spine{i}")),
            format!("Spine{i}"),
            Some(last),
            h.lerp(ch, t),
        );
    }
    let chest = push(role("spine", c, "chest"), "Chest".into(), Some(last), ch);
    // Neck joints from the neck guide towards the head.
    let (n, hd) = (at("head.C.neck")?, at("head.C.head")?);
    let mut last = chest.clone();
    for i in 1..=o.neck {
        let t = (i - 1) as f64 / o.neck as f64;
        let name = if o.neck == 1 {
            "Neck".to_string()
        } else {
            format!("Neck{i}")
        };
        last = push(
            role("head", c, &format!("neck{i}")),
            name,
            Some(last),
            n.lerp(hd, t),
        );
    }
    let head = push(role("head", c, "head"), "Head".into(), Some(last), hd);
    push(
        role("head", c, "top"),
        "HeadEnd".into(),
        Some(head),
        at("head.C.top")?,
    );
    for side in [Side::Left, Side::Right] {
        let s = side.suffix();
        let g = |piece: &str| at(&role("arm", side, piece));
        let clav = push(
            role("arm", side, "clavicle"),
            format!("Clavicle{s}"),
            Some(chest.clone()),
            g("clavicle")?,
        );
        let mut chain = |from: String,
                         bones: &[(&str, &str, u32, &str)],
                         module: &str,
                         end: (&str, &str)|
         -> Result<()> {
            // Each bone: (piece, joint name, twist count, next piece).
            let mut last = from;
            for &(piece, name, twist, next) in bones {
                let a = at(&role(module, side, piece))?;
                let b = at(&role(module, side, next))?;
                last = push(
                    role(module, side, piece),
                    format!("{name}{s}"),
                    Some(last),
                    a,
                );
                for i in 1..=twist {
                    let t = i as f64 / (twist + 1) as f64;
                    last = push(
                        role(module, side, &format!("{piece}Twist{i}")),
                        format!("{name}Twist{i}{s}"),
                        Some(last),
                        a.lerp(b, t),
                    );
                }
            }
            let (piece, name) = end;
            push(
                role(module, side, piece),
                format!("{name}{s}"),
                Some(last),
                at(&role(module, side, piece))?,
            );
            Ok(())
        };
        chain(
            clav,
            &[
                ("shoulder", "UpperArm", o.upper_arm_twist, "elbow"),
                ("elbow", "Forearm", o.forearm_twist, "wrist"),
                ("wrist", "Hand", 0, "hand"),
            ],
            "arm",
            ("hand", "HandEnd"),
        )?;
        chain(
            hips.clone(),
            &[
                ("thigh", "Thigh", o.thigh_twist, "knee"),
                ("knee", "Shin", o.shin_twist, "ankle"),
                ("ankle", "Foot", 0, "ball"),
                ("ball", "Toe", 0, "toe"),
            ],
            "leg",
            ("toe", "ToeEnd"),
        )?;
    }
    Ok(out)
}

// ------------------------------------------------------------ scene

impl Scene {
    pub fn character_rigs(&self) -> &[CharacterRig] {
        &self.animation.rig.characters
    }
    pub fn character_rig(&self, root: u64) -> Option<&CharacterRig> {
        self.animation
            .rig
            .characters
            .iter()
            .find(|c| c.root == root)
    }
    pub fn character_rig_mut(&mut self, root: u64) -> Option<&mut CharacterRig> {
        self.animation
            .rig
            .characters
            .iter_mut()
            .find(|c| c.root == root)
    }
    /// The rig `item` belongs to.
    pub fn character_rig_of(&self, item: u64) -> Option<&CharacterRig> {
        self.animation.rig.characters.iter().find(|c| c.owns(item))
    }
}

/// The world position of item `id`.
pub fn world_pos(scene: &Scene, id: u64) -> DVec3 {
    scene
        .world_matrix(id)
        .map(|m| m.w_axis.truncate())
        .unwrap_or(DVec3::ZERO)
}

fn parent_world(scene: &Scene, id: u64) -> DMat4 {
    scene
        .item(id)
        .and_then(|i| i.parent)
        .and_then(|p| scene.world_matrix(p).ok())
        .unwrap_or(DMat4::IDENTITY)
}

/// Moves item `id` so its world position is `p`, keeping its rotation and
/// scale; its children move with it.
pub fn set_world_pos(scene: &mut Scene, id: u64, p: DVec3) -> Result<()> {
    let local = parent_world(scene, id).inverse().transform_point3(p);
    let item = scene.item_mut(id).ok_or_else(|| format!("No item {id}"))?;
    item.transform.translation = local.map(clean);
    Ok(())
}

/// `p` mirrored across the rig root's YZ plane (its own X = 0).
pub fn mirror(scene: &Scene, rig: &CharacterRig, p: DVec3) -> DVec3 {
    let root = scene.world_matrix(rig.root).unwrap_or(DMat4::IDENTITY);
    let mut local = root.inverse().transform_point3(p);
    local.x = -local.x;
    root.transform_point3(local)
}

/// `p` put on the rig's centre plane.
fn on_centre(scene: &Scene, rig: &CharacterRig, p: DVec3) -> DVec3 {
    let root = scene.world_matrix(rig.root).unwrap_or(DMat4::IDENTITY);
    let mut local = root.inverse().transform_point3(p);
    local.x = 0.;
    root.transform_point3(local)
}

/// Adds a new item named `name` of locator `kind` under `parent` at world
/// position `at`. Returns its id.
pub fn add_locator(
    scene: &mut Scene,
    kind: LocatorKind,
    name: &str,
    parent: Option<u64>,
    at: DVec3,
) -> Result<u64> {
    let id = scene.add_group_locator_named(Some(name.into()))?;
    let local = match parent {
        Some(p) => scene.world_matrix(p)?.inverse().transform_point3(at),
        None => at,
    };
    let item = scene.item_mut(id).expect("made");
    item.locator = Some(kind);
    item.parent = parent;
    item.transform = Transform {
        translation: local.map(clean),
        ..Default::default()
    };
    Ok(id)
}

/// Puts every guide at its template place for the rig's reference size
/// (relative to the root), parents first.
pub fn place_template(scene: &mut Scene, root: u64) -> Result<()> {
    let rig = scene.character_rig(root).ok_or("No such rig")?.clone();
    let world = scene.world_matrix(root)?;
    for (role, _, at) in biped_guides() {
        if let Some(id) = rig.guide(&role)
            && scene.item(id).is_some()
        {
            let p = world.transform_point3(at * rig.reference_size);
            set_world_pos(scene, id, p)?;
        }
    }
    Ok(())
}

/// Makes a Biped rig (ACS-12) with its guides under `parent`-less root
/// `name` at `position`, `height` tall, in the Guide context. Returns the
/// root's id.
pub fn add_biped(scene: &mut Scene, name: &str, position: DVec3, height: f64) -> Result<u64> {
    if !(height.is_finite() && height > 0.) {
        return Err("height must be above 0".into());
    }
    let root = add_locator(scene, LocatorKind::Group, name, None, position)?;
    let guides_group = add_locator(
        scene,
        LocatorKind::Group,
        GUIDES_GROUP,
        Some(root),
        position,
    )?;
    let mut guides: Vec<Part> = Vec::new();
    for (role, parent, at) in biped_guides() {
        let parent_item = match &parent {
            Some(p) => guides
                .iter()
                .find(|g| &g.role == p)
                .map(|g| g.item)
                .ok_or("guide order")?,
            None => guides_group,
        };
        let id = add_locator(
            scene,
            LocatorKind::Guide,
            &guide_name(&role),
            Some(parent_item),
            position + at * height,
        )?;
        guides.push(Part { role, item: id });
    }
    scene.animation.rig.characters.push(CharacterRig {
        root,
        preset: "biped".into(),
        context: RigContext::Guide,
        symmetry: true,
        reference_size: height,
        options: BipedOptions::default(),
        guides_group,
        skeleton_group: None,
        guides,
        joints: Vec::new(),
    });
    Ok(root)
}

/// ACS-31: with symmetry on, each right guide is its left twin mirrored
/// and centre guides sit on the centre plane. `lead` is the side whose
/// guides lead; PROVISIONAL (ACS-Q3).
pub fn symmetrize(scene: &mut Scene, root: u64, lead: Side) -> Result<()> {
    let rig = scene.character_rig(root).ok_or("No such rig")?.clone();
    let follow = lead.other();
    for p in &rig.guides {
        if scene.item(p.item).is_none() {
            continue;
        }
        let side = side_of(&p.role);
        if side == Side::Centre {
            let now = world_pos(scene, p.item);
            let want = on_centre(scene, &rig, now);
            if now.distance(want) > 1e-12 {
                set_world_pos(scene, p.item, want)?;
            }
        } else if side == follow
            && let Some(twin) = rig.guide(&twin_role(&p.role))
            && scene.item(twin).is_some()
        {
            let want = mirror(scene, &rig, world_pos(scene, twin));
            if world_pos(scene, p.item).distance(want) > 1e-12 {
                set_world_pos(scene, p.item, want)?;
            }
        }
    }
    Ok(())
}

/// After an edit or during a drag (`before` the scene it started from):
/// keeps guide symmetry (ACS-31) on rigs whose guides moved, and drops
/// rig records whose root was deleted and parts whose items were.
pub fn sync(scene: &mut Scene, before: &Scene) {
    if scene.animation.rig.characters.is_empty() {
        return;
    }
    // Deleted items leave the rig (a deleted root takes the rig).
    let exists = |s: &Scene, id: u64| s.item(id).is_some();
    let mut characters = std::mem::take(&mut scene.animation.rig.characters);
    characters.retain(|c| exists(scene, c.root));
    for c in &mut characters {
        c.guides.retain(|p| exists(scene, p.item));
        c.joints.retain(|p| exists(scene, p.item));
        if c.skeleton_group.is_some_and(|g| !exists(scene, g)) {
            c.skeleton_group = None;
        }
    }
    scene.animation.rig.characters = characters;
    let roots: Vec<u64> = scene
        .animation
        .rig
        .characters
        .iter()
        .filter(|c| c.symmetry)
        .map(|c| c.root)
        .collect();
    for root in roots {
        let rig = scene.character_rig(root).expect("listed").clone();
        let moved = |side: Side| {
            rig.guides.iter().any(|p| {
                side_of(&p.role) == side
                    && before.item(p.item).is_some()
                    && world_pos(before, p.item).distance(world_pos(scene, p.item)) > 1e-12
            })
        };
        let (left, right, centre) = (moved(Side::Left), moved(Side::Right), moved(Side::Centre));
        if !(left || right || centre) {
            continue;
        }
        let lead = if right && !left {
            Side::Right
        } else {
            Side::Left
        };
        let _ = symmetrize(scene, root, lead);
    }
}

/// ACS-02/03: after undo or redo, each rig keeps the context it had in
/// the scene being left (contexts aren't undo steps).
pub fn keep_contexts(left: &Scene, restored: &mut Scene) {
    for c in &mut restored.animation.rig.characters {
        if let Some(l) = left.character_rig(c.root) {
            c.context = l.context;
        }
    }
}

/// ACS-01: whether a rig's context hides item `id` in the views. Only the
/// rig's own guides and joints are affected; meshes always show.
pub fn hidden_by_context(scene: &Scene, id: u64) -> bool {
    scene.animation.rig.characters.iter().any(|c| {
        (!c.context.shows_guides() && c.guides.iter().any(|p| p.item == id))
            || (!c.context.shows_skeleton() && c.joints.iter().any(|p| p.item == id))
    })
}

// ------------------------------------------------------------ fit

/// The first hits of a ray on mesh item `mesh`, nearest first (distances
/// along the unit `dir`).
fn hits(scene: &Scene, mesh: u64, origin: DVec3, dir: DVec3) -> Vec<f64> {
    let Some(item) = scene.item(mesh) else {
        return Vec::new();
    };
    let Ok(w) = scene.world_matrix(mesh) else {
        return Vec::new();
    };
    let m = item.mesh.open();
    let p: Vec<DVec3> = m.vertices.iter().map(|v| w.transform_point3(*v)).collect();
    let mut out = Vec::new();
    for f in m.faces.iter() {
        if f.vertices.len() < 3 {
            continue;
        }
        let a = p[f.vertices[0] as usize];
        for k in 1..f.vertices.len() - 1 {
            let (b, c) = (p[f.vertices[k] as usize], p[f.vertices[k + 1] as usize]);
            if let Some(t) = ray_triangle(origin, dir, a, b, c) {
                out.push(t);
            }
        }
    }
    out.sort_by(|a, b| a.total_cmp(b));
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    out
}

fn ray_triangle(o: DVec3, d: DVec3, a: DVec3, b: DVec3, c: DVec3) -> Option<f64> {
    let e1 = b - a;
    let e2 = c - a;
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-14 {
        return None;
    }
    let inv = 1. / det;
    let s = o - a;
    let u = s.dot(p) * inv;
    if !(0. ..=1.).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0. || u + v > 1. {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 1e-9).then_some(t)
}

/// World bounds of mesh item `mesh`.
pub fn mesh_bounds(scene: &Scene, mesh: u64) -> Result<(DVec3, DVec3)> {
    let item = scene.item(mesh).ok_or_else(|| format!("No item {mesh}"))?;
    if !item.is_mesh() || item.mesh.vertices.is_empty() {
        return Err(format!("{} has no geometry to fit to", item.name));
    }
    let w = scene.world_matrix(mesh)?;
    let mut lo = DVec3::splat(f64::INFINITY);
    let mut hi = DVec3::splat(f64::NEG_INFINITY);
    for v in item.mesh.open().vertices.iter() {
        let p = w.transform_point3(*v);
        lo = lo.min(p);
        hi = hi.max(p);
    }
    Ok((lo, hi))
}

/// How Fit to Mesh embeds a guide (ACS-34). PROVISIONAL (ACS-Q6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Embed {
    /// Centred front to back (Z): spine and head.
    Depth,
    /// Centred up and down (Y) and front to back: arms in a T pose.
    Arm,
    /// Centred across (X, entering from the outside) and front to back.
    Leg,
    /// Centred across only (the ball of the foot stays near the floor).
    Across,
    /// The top of the head.
    Top,
    /// The fingertips: as far out as the mesh reaches.
    Reach,
    /// The toe tip: as far forward as the mesh reaches.
    Front,
}

fn embed_of(role: &str) -> Embed {
    match role.rsplit('.').next().unwrap_or("") {
        "top" => Embed::Top,
        "hand" => Embed::Reach,
        "toe" => Embed::Front,
        "ball" => Embed::Across,
        "clavicle" | "shoulder" | "elbow" | "wrist" => Embed::Arm,
        "thigh" | "knee" | "ankle" => Embed::Leg,
        _ => Embed::Depth,
    }
}

/// Where guide `role` at `p` goes inside mesh `mesh`, `size` the
/// character's height (rays start that far out). `None` if the rays miss.
fn embedded(scene: &Scene, mesh: u64, role: &str, p: DVec3, size: f64) -> Option<DVec3> {
    let far = size * 4.;
    let side = side_of(role);
    let outward = if side == Side::Right { -1. } else { 1. };
    let mid = |origin: DVec3, dir: DVec3| -> Option<f64> {
        let h = hits(scene, mesh, origin, dir);
        match h.as_slice() {
            [a, b, ..] => Some((a + b) / 2.),
            _ => None,
        }
    };
    let first = |origin: DVec3, dir: DVec3| hits(scene, mesh, origin, dir).first().copied();
    let mut q = p;
    let depth = |q: &mut DVec3| {
        if let Some(t) = mid(DVec3::new(q.x, q.y, q.z + far), -DVec3::Z) {
            q.z = q.z + far - t;
        }
    };
    match embed_of(role) {
        Embed::Depth => depth(&mut q),
        Embed::Arm => {
            if let Some(t) = mid(DVec3::new(q.x, q.y + far, q.z), -DVec3::Y) {
                q.y = q.y + far - t;
            }
            depth(&mut q);
        }
        Embed::Leg | Embed::Across => {
            let o = DVec3::new(q.x + outward * far, q.y, q.z);
            if let Some(t) = mid(o, DVec3::X * -outward) {
                q.x = o.x - outward * t;
            }
            if embed_of(role) == Embed::Leg {
                depth(&mut q);
            }
        }
        Embed::Top => {
            let t = first(DVec3::new(q.x, q.y + far, q.z), -DVec3::Y)?;
            q.y = q.y + far - t;
        }
        Embed::Reach => {
            let o = DVec3::new(q.x + outward * far, q.y, q.z);
            let t = first(o, DVec3::X * -outward)?;
            q.x = o.x - outward * t;
        }
        Embed::Front => {
            let t = first(DVec3::new(q.x, q.y, q.z + far), -DVec3::Z)?;
            q.z = q.z + far - t;
        }
    }
    (q != p).then_some(q)
}

/// ACS-34 Fit guides to mesh: the rig moves to stand on the mesh's floor
/// at its centre, takes its height as the reference size, puts the guides
/// at their template places and, with `embed`, moves each into the mesh.
/// Returns how many guides were embedded.
pub fn fit_to_mesh(scene: &mut Scene, root: u64, mesh: u64, embed: bool) -> Result<usize> {
    let (lo, hi) = mesh_bounds(scene, mesh)?;
    let height = hi.y - lo.y;
    if height <= 1e-9 {
        return Err("The mesh is flat; fit needs a standing character".into());
    }
    let base = DVec3::new((lo.x + hi.x) / 2., lo.y, (lo.z + hi.z) / 2.);
    let root_parent = parent_world(scene, root);
    {
        let local = root_parent.inverse().transform_point3(base);
        let item = scene.item_mut(root).ok_or("No rig root")?;
        item.transform.translation = local.map(clean);
    }
    scene
        .character_rig_mut(root)
        .ok_or("No such rig")?
        .reference_size = clean(height);
    place_template(scene, root)?;
    if !embed {
        return Ok(0);
    }
    let rig = scene.character_rig(root).expect("checked").clone();
    let mut count = 0;
    for p in &rig.guides {
        if scene.item(p.item).is_none() {
            continue;
        }
        if rig.symmetry && side_of(&p.role) == Side::Right {
            continue;
        }
        let now = world_pos(scene, p.item);
        if let Some(q) = embedded(scene, mesh, &p.role, now, height) {
            set_world_pos(scene, p.item, q)?;
            count += 1;
        }
    }
    if rig.symmetry {
        symmetrize(scene, root, Side::Left)?;
    }
    Ok(count)
}

// ------------------------------------------------------------ build

/// ACS-32 Apply guides: builds the bind skeleton from the guides, or
/// moves the existing one onto them. Joints keep their ids (so bound
/// meshes keep their weights); joints a smaller option count no longer
/// needs are deleted, unless a mesh is bound to them. Returns
/// (made, moved, removed).
pub fn build_skeleton(scene: &mut Scene, root: u64) -> Result<(usize, usize, usize)> {
    let rig = scene.character_rig(root).ok_or("No such rig")?.clone();
    let at = |role: &str| -> Result<DVec3> {
        let id = rig
            .guide(role)
            .filter(|id| scene.item(*id).is_some())
            .ok_or_else(|| format!("{} is missing; Reset Guides puts it back", guide_name(role)))?;
        Ok(world_pos(scene, id))
    };
    let plan = biped_joints(&rig.options, &at)?;
    // Joints no longer planned, and whether a skin uses them.
    let stale: Vec<u64> = rig
        .joints
        .iter()
        .filter(|p| !plan.iter().any(|j| j.role == p.role))
        .map(|p| p.item)
        .collect();
    for item in &scene.items {
        if let Ok((_, v)) = crate::skin_commands::skin_op(scene, item.id)
            && let Some(j) = v.joints.iter().find(|j| stale.contains(j))
        {
            let joint = scene.item(*j).map_or(String::new(), |i| i.name.clone());
            return Err(format!(
                "{} is bound to {joint}; unbind it before lowering the joint count",
                item.name
            ));
        }
    }
    let group = match rig.skeleton_group.filter(|g| scene.item(*g).is_some()) {
        Some(g) => g,
        None => {
            let p = world_pos(scene, root);
            add_locator(scene, LocatorKind::Group, SKELETON_GROUP, Some(root), p)?
        }
    };
    let (mut made, mut moved) = (0, 0);
    let mut joints: Vec<Part> = Vec::new();
    for j in &plan {
        let parent = match &j.parent {
            Some(p) => joints.iter().find(|q| &q.role == p).map(|q| q.item),
            None => Some(group),
        };
        let existing = rig
            .joint(&j.role)
            .filter(|id| scene.item(*id).is_some_and(|i| i.is_joint()));
        let id = match existing {
            Some(id) => {
                if scene.item(id).and_then(|i| i.parent) != parent {
                    scene.set_parent(id, parent, true)?;
                }
                // Square to the world first; alignment aims it below.
                let local = parent_world(scene, id).inverse() * DMat4::from_translation(j.at);
                let (s, q, t) = local.to_scale_rotation_translation();
                let item = scene.item_mut(id).expect("checked");
                item.transform.translation = t.map(clean);
                item.transform.rotation = item.transform.angles_for(q).map(clean);
                item.transform.scale = s.map(clean);
                if item.name != j.name {
                    item.name = j.name.clone();
                }
                moved += 1;
                id
            }
            None => {
                made += 1;
                crate::skeleton::add_joint(scene, parent, j.at, j.name.clone())?
            }
        };
        joints.push(Part {
            role: j.role.clone(),
            item: id,
        });
    }
    // Children are put in plan order under each parent, so a joint's first
    // child (its bone, SK-17) is the next joint of its chain.
    for j in &joints {
        if let Some(i) = scene.item_index(j.item) {
            let item = scene.items.remove(i);
            scene.items.push(item);
        }
    }
    let removed = stale.len();
    if !stale.is_empty() {
        scene.delete_items(&stale, crate::item_delete::ChildrenOnDelete::MoveToRoot)?;
    }
    // SK-16: each joint aims its Z down its bone, X across.
    for j in &joints {
        crate::skeleton::align(scene, j.item, 2, 0)?;
    }
    let c = scene.character_rig_mut(root).expect("checked");
    c.skeleton_group = Some(group);
    c.joints = joints;
    Ok((made, moved, removed))
}

/// Mesh items bound to the rig's skeleton (ACS-41): a skin that uses one
/// of its joints.
pub fn bound_meshes(scene: &Scene, rig: &CharacterRig) -> Vec<u64> {
    scene
        .items
        .iter()
        .filter(|i| {
            crate::skin_commands::skin_op(scene, i.id)
                .is_ok_and(|(_, v)| v.joints.iter().any(|j| rig.role_of(*j).is_some()))
        })
        .map(|i| i.id)
        .collect()
}
