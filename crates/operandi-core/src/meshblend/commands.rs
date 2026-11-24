//! MeshBlend commands (PLAN.md rule 3): every edit of a MeshBlend recipe
//! is one undoable command (SB-80), so the panel now and the toolbox, the
//! drag menu and MCP later all call the same ones.
//!
//! | Script | What it does |
//! |---|---|
//! | `meshblend.new` | A new MeshBlend Item from the selected meshes, all Primaries (SB-01, SB-60) |
//! | `meshblend.addSource` | Adds a mesh as a source with a role and, for a Trim, its Primaries (SB-11) |
//! | `meshblend.role` | Changes a source's role (SB-10); relationships are kept |
//! | `meshblend.trim` / `meshblend.untrim` | Adds or removes Trim ↔ Primary relationships (SB-11) |
//! | `meshblend.clearRole` | Takes a source out; it is an ordinary mesh again (SB-12) |
//! | `meshblend.source` | Per-source settings: enabled, Subdivision, Keep cage points |
//! | `meshblend.convert` | Freezes the result into an ordinary mesh, in place or as a copy (SB-53) |
//!
//! Each names the MeshBlend Item with `item:<id>` or works on the selected
//! one. Strip settings are set with `item.channel` on the op.
use super::op::MESHBLEND_OP;
use super::recipe::{BlendSource, Recipe, Role};
use crate::mesh_stack::{MeshOp, MeshStack, OpKind};
use crate::mesh_stack_editor::{OpRef, OpRow};
use crate::{Editor, Result};
use glam::DVec3;

/// One MeshBlend edit.
#[derive(Clone, Debug, PartialEq)]
pub enum MeshBlendCommand {
    New,
    AddSource {
        blend: Option<u64>,
        source: u64,
        role: Role,
        of: Vec<u64>,
    },
    Role {
        blend: Option<u64>,
        source: u64,
        role: Role,
    },
    Trim {
        blend: Option<u64>,
        trims: Vec<u64>,
        of: Vec<u64>,
        untrim: bool,
    },
    Clear {
        blend: Option<u64>,
        source: u64,
    },
    Source {
        blend: Option<u64>,
        source: u64,
        enabled: Option<bool>,
        subdivision: Option<u8>,
        keep_cage: Option<bool>,
    },
    Convert {
        blend: Option<u64>,
        duplicate: bool,
    },
}

/// The recipe of item `id`, if it is a MeshBlend Item (its stack starts
/// with the MeshBlend op).
pub fn recipe_of(scene: &crate::Scene, id: u64) -> Option<(u64, Recipe)> {
    let op = scene.item(id)?.mesh_stack.as_ref()?.ops.first()?;
    match &op.kind {
        OpKind::Op { type_id, values } if type_id == MESHBLEND_OP => {
            serde_json::from_value(values.clone())
                .ok()
                .map(|r| (op.id, r))
        }
        _ => None,
    }
}

impl Editor {
    pub(crate) fn meshblend(&mut self, cmd: MeshBlendCommand) -> Result<()> {
        self.require_idle()?;
        match cmd {
            MeshBlendCommand::New => self.meshblend_new(),
            MeshBlendCommand::AddSource {
                blend,
                source,
                role,
                of,
            } => self.meshblend_edit(blend, "Add MeshBlend Source", |e, id, r| {
                e.check_source(id, source)?;
                if r.source(source).is_some() {
                    return Err("That mesh is already a source of this MeshBlend".into());
                }
                let mut s = BlendSource::new(source, role);
                if role.is_trim() {
                    s.trims = primaries_or_all(r, &of)?;
                }
                r.sources.push(s);
                Ok(())
            }),
            MeshBlendCommand::Role {
                blend,
                source,
                role,
            } => self.meshblend_edit(blend, "Set MeshBlend Role", |_, _, r| {
                let all = primaries_or_all(r, &[])?;
                let s = r.source_mut(source).ok_or(NOT_A_SOURCE)?;
                s.role = role;
                if role == Role::Primary {
                    s.trims.clear();
                } else if s.trims.is_empty() {
                    s.trims = all.into_iter().filter(|&p| p != source).collect();
                }
                if role.is_trim() {
                    // A Trim can't be the Primary other Trims act on.
                    for t in &mut r.sources {
                        t.trims.retain(|&p| p != source);
                    }
                }
                Ok(())
            }),
            MeshBlendCommand::Trim {
                blend,
                trims,
                of,
                untrim,
            } => {
                let label = if untrim { "Untrim" } else { "Trim" };
                self.meshblend_edit(blend, label, |_, _, r| {
                    for &p in &of {
                        if r.source(p).map(|s| s.role) != Some(Role::Primary) {
                            return Err("Trim relates Trims to Primaries; pick a Primary".into());
                        }
                    }
                    for &t in &trims {
                        let s = r.source_mut(t).ok_or(NOT_A_SOURCE)?;
                        if !s.role.is_trim() {
                            return Err("Only a Subtractive or Intersect Trim can trim".into());
                        }
                        for &p in &of {
                            if untrim {
                                s.trims.retain(|&x| x != p);
                            } else if !s.trims.contains(&p) {
                                s.trims.push(p);
                            }
                        }
                    }
                    Ok(())
                })
            }
            MeshBlendCommand::Clear { blend, source } => {
                self.meshblend_edit(blend, "Clear MeshBlend Role", |_, _, r| {
                    if r.source(source).is_none() {
                        return Err(NOT_A_SOURCE.into());
                    }
                    r.sources.retain(|s| s.item != source);
                    for t in &mut r.sources {
                        t.trims.retain(|&p| p != source);
                    }
                    Ok(())
                })
            }
            MeshBlendCommand::Source {
                blend,
                source,
                enabled,
                subdivision,
                keep_cage,
            } => self.meshblend_edit(blend, "MeshBlend Source Settings", |_, _, r| {
                let s = r.source_mut(source).ok_or(NOT_A_SOURCE)?;
                if let Some(v) = enabled {
                    s.enabled = v;
                }
                if let Some(v) = subdivision {
                    if v > 3 {
                        return Err("Subdivision must be 0 to 3".into());
                    }
                    s.subdivision = v;
                }
                if let Some(v) = keep_cage {
                    s.keep_cage = v;
                }
                Ok(())
            }),
            MeshBlendCommand::Convert { blend, duplicate } => {
                let id = self.blend_item(blend)?;
                let top = self
                    .scene
                    .item(id)
                    .and_then(|i| i.mesh_stack.as_ref())
                    .and_then(|s| s.ops.last())
                    .map(|o| o.id)
                    .ok_or("This MeshBlend Item has no stack")?;
                self.meshop_freeze(OpRef { item: id, op: top }, duplicate)
            }
        }
    }

    /// The MeshBlend Item named by `blend`, or the selected one.
    fn blend_item(&self, blend: Option<u64>) -> Result<u64> {
        let id = match blend {
            Some(id) => id,
            None => self
                .scene
                .selection
                .item
                .ok_or("Select a MeshBlend Item first")?,
        };
        recipe_of(&self.scene, id)
            .map(|_| id)
            .ok_or_else(|| "That item isn't a MeshBlend Item".into())
    }

    /// A mesh item that can be a source of `blend`.
    fn check_source(&self, blend: u64, source: u64) -> Result<()> {
        if source == blend {
            return Err("A MeshBlend Item can't be its own source".into());
        }
        match self.scene.item(source) {
            Some(i) if i.is_mesh() => Ok(()),
            Some(_) => Err("Only mesh items can be MeshBlend sources".into()),
            None => Err("That item doesn't exist".into()),
        }
    }

    /// Edits the recipe of `blend` as one undo step named `label`,
    /// refused (scene unchanged) if `edit` fails or the result would read
    /// its own output (SAFE-07).
    fn meshblend_edit(
        &mut self,
        blend: Option<u64>,
        label: &str,
        edit: impl FnOnce(&Editor, u64, &mut Recipe) -> Result<()>,
    ) -> Result<()> {
        let id = self.blend_item(blend)?;
        let (op, mut recipe) = recipe_of(&self.scene, id).expect("checked");
        edit(self, id, &mut recipe)?;
        recipe.check()?;
        let before = self.scene.clone();
        set_recipe(&mut self.scene, id, op, &recipe);
        if let Some((item, _)) = crate::mesh_stack::find_loop(&self.scene) {
            let name = self
                .scene
                .item(item)
                .map_or_else(String::new, |i| i.name.clone());
            self.scene = before;
            return Err(format!(
                "That would make a dependency loop: {name} would end up reading its own result"
            ));
        }
        self.record(label.into(), before);
        Ok(())
    }

    /// `meshblend.new`: a MeshBlend Item from the selected meshes, all
    /// Primaries (SB-01). The new item is selected (SB-60); nothing else
    /// is added to the scene (SB-42).
    fn meshblend_new(&mut self) -> Result<()> {
        let picked: Vec<u64> = self
            .scene
            .picked_items()
            .into_iter()
            .filter(|&id| self.scene.item(id).is_some_and(|i| i.is_mesh()))
            .collect();
        if picked.is_empty() {
            return Err("Select the meshes to blend first".into());
        }
        // Q04: Strip Width starts at 2 % of the sources' size.
        let (mut lo, mut hi) = (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY));
        for &id in &picked {
            let (Some(item), Ok(m)) = (self.scene.item(id), self.scene.world_matrix(id)) else {
                continue;
            };
            for p in &item.mesh.vertices {
                let w = m.transform_point3(*p);
                lo = lo.min(w);
                hi = hi.max(w);
            }
        }
        let size = (hi - lo).max_element();
        let recipe = Recipe {
            sources: picked
                .iter()
                .map(|&id| BlendSource::new(id, Role::Primary))
                .collect(),
            strip_width: if size.is_finite() && size > 0. {
                0.02 * size
            } else {
                Recipe::default().strip_width
            },
            ..Default::default()
        };
        let before = self.scene.clone();
        let name = crate::user_defaults::free_name_among(
            "MeshBlend",
            self.scene.items.iter().map(|i| i.name.as_str()),
        );
        self.scene.new_mesh_named(Some(name))?;
        let id = self.scene.selection.item.expect("new_mesh selects it");
        let values = serde_json::to_value(&recipe).map_err(|e| e.to_string())?;
        let item = self.scene.item_mut(id).expect("just made");
        item.mesh_stack = Some(MeshStack {
            ops: vec![MeshOp {
                id: 1,
                enabled: true,
                kind: OpKind::Op {
                    type_id: MESHBLEND_OP.into(),
                    values,
                },
                selection: None,
                selops: Vec::new(),
            }],
            next_id: 2,
        });
        self.meshop_row = Some(OpRow {
            item: id,
            op: Some(1),
        });
        self.record("New MeshBlend".into(), before);
        Ok(())
    }
}

const NOT_A_SOURCE: &str = "That mesh isn't a source of this MeshBlend";

/// A Trim's Primaries: the ones named, checked, or every Primary.
fn primaries_or_all(r: &Recipe, of: &[u64]) -> Result<Vec<u64>> {
    if of.is_empty() {
        return Ok(r
            .sources
            .iter()
            .filter(|s| s.role == Role::Primary)
            .map(|s| s.item)
            .collect());
    }
    for &p in of {
        if r.source(p).map(|s| s.role) != Some(Role::Primary) {
            return Err("A Trim relates to Primaries of the same MeshBlend".into());
        }
    }
    Ok(of.to_vec())
}

fn set_recipe(scene: &mut crate::Scene, item: u64, op: u64, recipe: &Recipe) {
    let Some(o) = scene
        .item_mut(item)
        .and_then(|i| i.mesh_stack.as_mut())
        .and_then(|s| s.ops.iter_mut().find(|o| o.id == op))
    else {
        return;
    };
    if let Ok(values) = serde_json::to_value(recipe) {
        o.kind = OpKind::Op {
            type_id: MESHBLEND_OP.into(),
            values,
        };
    }
}

/// The command a `meshblend.*` script names, from its arguments.
pub(crate) fn parse(
    name: &str,
    a: &serde_json::Map<String, serde_json::Value>,
) -> Result<Option<MeshBlendCommand>> {
    use crate::command_exec::{opt_bool, opt_str, opt_u64};
    if !name.starts_with("meshblend.") {
        return Ok(None);
    }
    let ids = |key: &str| -> Result<Vec<u64>> {
        match a.get(key) {
            None | Some(serde_json::Value::Null) => Ok(Vec::new()),
            Some(serde_json::Value::Array(list)) => list
                .iter()
                .map(|v| {
                    v.as_u64()
                        .ok_or_else(|| format!("{key} must be a list of item ids"))
                })
                .collect(),
            Some(v) => v
                .as_u64()
                .map(|id| vec![id])
                .ok_or_else(|| format!("{key} must be an item id or a list of them")),
        }
    };
    let blend = opt_u64(a, "item")?;
    let source = || -> Result<u64> { opt_u64(a, "source")?.ok_or("source is required".into()) };
    let role = |required: bool| -> Result<Option<Role>> {
        match opt_str(a, "role")? {
            Some(r) => Role::parse(r).map(Some),
            None if required => Err("role is required (primary, subtract or intersect)".into()),
            None => Ok(None),
        }
    };
    Ok(Some(match name {
        "meshblend.new" => MeshBlendCommand::New,
        "meshblend.addSource" => MeshBlendCommand::AddSource {
            blend,
            source: source()?,
            role: role(false)?.unwrap_or_default(),
            of: ids("of")?,
        },
        "meshblend.role" => MeshBlendCommand::Role {
            blend,
            source: source()?,
            role: role(true)?.expect("required"),
        },
        "meshblend.trim" | "meshblend.untrim" => MeshBlendCommand::Trim {
            blend,
            trims: {
                let t = ids("source")?;
                if t.is_empty() {
                    return Err("source is required: the Trim (or a list of them)".into());
                }
                t
            },
            of: {
                let p = ids("of")?;
                if p.is_empty() {
                    return Err("of is required: the Primary (or a list of them)".into());
                }
                p
            },
            untrim: name == "meshblend.untrim",
        },
        "meshblend.clearRole" => MeshBlendCommand::Clear {
            blend,
            source: source()?,
        },
        "meshblend.source" => MeshBlendCommand::Source {
            blend,
            source: source()?,
            enabled: opt_bool(a, "enabled")?,
            subdivision: opt_u64(a, "subdivision")?
                .map(|v| u8::try_from(v).map_err(|_| "Subdivision must be 0 to 3".to_string()))
                .transpose()?,
            keep_cage: opt_bool(a, "keepCage")?,
        },
        "meshblend.convert" => MeshBlendCommand::Convert {
            blend,
            duplicate: opt_bool(a, "duplicate")?.unwrap_or(false),
        },
        _ => return Ok(None),
    }))
}
