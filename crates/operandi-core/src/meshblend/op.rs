//! The MeshBlend op (PLAN.md rule 4): a MeshBlend Item is a mesh item
//! whose Mesh Stack starts with this op. The op reads its sources through
//! item links (SAFE-07..09), so the stack's guard, cache, background
//! evaluation, saving and loop checks all apply, and hands them to the
//! plain evaluator ([`super::evaluate`]).
//!
//! The mesh below the op (the item's Base Mesh, normally empty) is not
//! used: the result is the blend of the sources alone.
use super::evaluate::{Input, evaluate};
use super::recipe::Recipe;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, OpNote};
use crate::{Mesh, Result};
use std::sync::Arc;

/// The strips each recipe last made, for the panel's strip list (keyed by
/// a hash of the op's values; a cache, never saved).
static STRIPS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<u64, Vec<super::strips::StripInfo>>>,
> = std::sync::LazyLock::new(Default::default);

fn values_key(values: &serde_json::Value) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    values.to_string().hash(&mut h);
    h.finish()
}

/// The strips the MeshBlend op with these values made when it last ran.
pub fn last_strips(values: &serde_json::Value) -> Vec<super::strips::StripInfo> {
    STRIPS
        .lock()
        .map(|m| m.get(&values_key(values)).cloned().unwrap_or_default())
        .unwrap_or_default()
}

/// The saved type id.
pub const MESHBLEND_OP: &str = "meshblend.item";

struct MeshBlendOp {
    fields: Vec<Field>,
}

impl OpDef for MeshBlendOp {
    fn id(&self) -> &str {
        MESHBLEND_OP
    }
    fn label(&self) -> &str {
        "MeshBlend"
    }
    fn folder(&self) -> &str {
        "Edit"
    }
    fn takes_selection(&self) -> bool {
        false
    }
    fn fields(&self) -> &[Field] {
        &self.fields
    }
    fn defaults(&self) -> serde_json::Value {
        serde_json::to_value(Recipe::default()).unwrap_or(serde_json::Value::Null)
    }
    fn read(&self, version: u32, values: &serde_json::Value) -> Result<serde_json::Value> {
        if version != self.version() {
            return Err(format!(
                "Saved by a newer Operandi (version {version} of MeshBlend)"
            ));
        }
        let recipe: Recipe = serde_json::from_value(values.clone()).map_err(|e| e.to_string())?;
        serde_json::to_value(recipe).map_err(|e| e.to_string())
    }
    /// Enabled sources only: a source switched off isn't read (SB-16).
    fn links(&self, values: &serde_json::Value, _scene: &crate::Scene) -> Vec<u64> {
        serde_json::from_value::<Recipe>(values.clone())
            .map(|r| {
                r.sources
                    .iter()
                    .filter(|s| s.enabled)
                    .map(|s| s.item)
                    .collect()
            })
            .unwrap_or_default()
    }
    /// Only the width in the chosen units applies (UIC-117).
    fn field_unavailable(&self, key: &str, values: &serde_json::Value) -> Option<String> {
        let relative = values["widthUnits"].as_u64() == Some(1);
        match key {
            "stripWidth" if relative => {
                Some("Width Units is Percent of Polygon Size: Strip Width % applies".into())
            }
            "stripWidthRel" if !relative => {
                Some("Width Units is Distance: Strip Width applies".into())
            }
            _ => None,
        }
    }
    fn run(&self, ctx: &OpContext, values: &serde_json::Value) -> Result<Mesh> {
        let recipe: Recipe = serde_json::from_value(values.clone()).map_err(|e| e.to_string())?;
        if recipe.sources.is_empty() {
            ctx.note(OpNote {
                item: 0,
                text: "No sources yet: add meshes with MeshBlend ▸ Add Source".into(),
                problem: false,
            });
            return Ok(Mesh::default());
        }
        let mut inputs = Vec::new();
        for l in ctx.links {
            match &l.source {
                Ok(s) => inputs.push(Input {
                    item: s.item,
                    name: s.name.clone(),
                    mesh: s.local_mesh(),
                }),
                Err(reason) => ctx.note(OpNote {
                    item: l.item,
                    text: reason.clone(),
                    problem: true,
                }),
            }
        }
        // A link that failed is already noted; don't note it twice.
        let failed: Vec<u64> = ctx
            .links
            .iter()
            .filter(|l| l.source.is_err())
            .map(|l| l.item)
            .collect();
        let blend = evaluate(&recipe, &inputs)?;
        if let Ok(mut m) = STRIPS.lock() {
            if m.len() > 256 {
                m.clear();
            }
            m.insert(values_key(values), blend.strips.clone());
        }
        for n in blend.notes {
            if n.problem && failed.contains(&n.item) {
                continue;
            }
            ctx.note(OpNote {
                item: n.item,
                text: n.text,
                problem: n.problem,
            });
        }
        Ok(blend.mesh)
    }
}

/// The op's table entry. The strip settings use the generic op form; the
/// sources are drawn by the MeshBlend panel section (meshblend_ui.rs), and
/// what isn't built yet (relative width, per-strip overrides) stays
/// hidden (UIC-117).
pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let hidden = |key: &'static str| Field::new(key, key, FieldKind::Hidden);
    vec![Arc::new(MeshBlendOp {
        fields: vec![
            hidden("sources"),
            Field::new(
                "widthUnits",
                "Width Units",
                FieldKind::Choice(vec!["Distance".into(), "Percent of Polygon Size".into()]),
            ),
            Field::new("stripWidth", "Strip Width", FieldKind::Distance),
            Field::new("stripWidthRel", "Strip Width %", FieldKind::Percent),
            Field::new("stripProfile", "Strip Profile", FieldKind::Percent),
            Field::new(
                "stripRows",
                "Strip Rows",
                FieldKind::Count { min: 1, max: 16 },
            ),
            Field::new("designMode", "Design Mode", FieldKind::Check),
            Field::new(
                "output",
                "Output",
                FieldKind::Choice(super::recipe::OUTPUTS.iter().map(|&o| o.into()).collect()),
            ),
            hidden("strips"),
        ],
    })]
}
