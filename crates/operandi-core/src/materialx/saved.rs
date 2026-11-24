//! The saved MaterialX graphs (materials architecture stage 4,
//! docs/MATERIALX_GRAPHS.md, AR-32): what a scene file keeps as the
//! master copy of its materials.
//!
//! A scene file holds every distinct surface its polygons render with, as
//! the MaterialX form Cycles renders (`materialx::tree`; texture
//! placement by reference, so items with the same material share one
//! graph), and a binding table saying which polygons use which graph: an
//! item, its polygons'
//! material and part tags and their selection sets, as MaterialX and
//! UsdShade bind materials to geometry. The Shader Tree's layers are
//! saved next to them as the view's layout (layer order, names, masks,
//! hidden layers), so the tree comes back exactly.
//!
//! On open the graphs are the truth. For each binding the layers are
//! evaluated again; where they give a different surface than the saved
//! graph (a file written by a build whose layer sums differed, or a graph
//! edited outside Operandi), the saved graph is *pinned* to those
//! polygons: renders and exports use it, so the scene renders as it was
//! saved. A pin holds only until the layers that shade those polygons
//! change (an edit in the Shader Tree is a new truth), and a save writes
//! whichever is in force. The viewport still draws the layers
//! (provisional, MXG-Q12).
//!
//! Older scenes carry no graphs; they open from their layers and save
//! their graphs from then on (the upgrade), which is exact because the
//! graphs are made by the same code that renders them.
use super::tree;
use crate::shader_eval::{Context, Principled};
use crate::shader_tree::ShaderTree;
use crate::texture_locator::LocatorFrame;
use crate::{Result, Scene};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One saved graph: a surface's MaterialX text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedGraph {
    /// The material's name (the first polygon tag that uses it).
    pub name: String,
    /// Texture placement goes by reference: each locator matrix entry is
    /// a reference number, filled in with the bound item's own value when
    /// the graph is read for it (see `place`). Off: the graph is one
    /// item's, numbers and all.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub placed: bool,
    /// The surface's MaterialX form (`materialx::tree`).
    pub materialx: String,
}

/// Which polygons render with which graph.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    /// The item the polygons belong to.
    pub item: u64,
    /// Their material tag (empty for untagged polygons).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub material: String,
    /// Their part tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// Every selection set they are in (G-01), in the mesh's order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sets: Vec<String>,
    /// Index into the file's graphs.
    pub graph: usize,
}

/// The file's materials section.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedGraphs {
    pub graphs: Vec<SavedGraph>,
    pub bindings: Vec<Binding>,
}
impl SavedGraphs {
    pub fn is_empty(&self) -> bool {
        self.graphs.is_empty() && self.bindings.is_empty()
    }
}

/// A saved graph in force for one binding's polygons (see the module
/// notes). Not saved: a save writes the graph itself.
#[derive(Clone, Debug, PartialEq)]
pub struct Pin {
    key: Key,
    /// What the layers gave when the scene was opened. While they still
    /// give this, the pin holds.
    basis: Principled,
    /// The saved graph, read.
    graph: Principled,
    /// The material's name, for notes.
    pub name: String,
}

/// The pins of an open scene (`Scene::material_pins`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pins(Vec<Pin>);
impl Pins {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &Pin> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    item: u64,
    material: String,
    part: Option<String>,
    sets: Vec<String>,
}
impl Key {
    fn of(ctx: &Context) -> Key {
        Key {
            item: ctx.item,
            material: ctx.material.to_owned(),
            part: ctx.part.map(str::to_owned),
            sets: ctx.sets.iter().map(|s| (*s).to_owned()).collect(),
        }
    }
    fn matches(&self, ctx: &Context) -> bool {
        self.item == ctx.item
            && self.material == ctx.material
            && self.part.as_deref() == ctx.part
            && self.sets.len() == ctx.sets.len()
            && self.sets.iter().zip(ctx.sets).all(|(a, b)| a == b)
    }
}

/// The surface `ctx`'s polygons render with: `layers` (what the Shader
/// Tree evaluates to there, with or without material values) unless a
/// saved graph is pinned to them. Every renderer and exporter asks this,
/// never the layers alone.
pub fn surface(scene: &Scene, ctx: &Context, mut layers: Principled) -> Principled {
    if scene.material_pins.is_empty() {
        return layers;
    }
    let values = std::mem::take(&mut layers.values);
    let Some(pin) = scene.material_pins.0.iter().find(|p| p.key.matches(ctx)) else {
        layers.values = values;
        return layers;
    };
    let mut out = if pin.basis == layers {
        pin.graph.clone()
    } else {
        layers
    };
    out.values = values;
    out
}

/// Whether a scene saves graphs: one whose materials differ from a new
/// scene's (a default scene's graphs are made again exactly by any build).
pub fn saves_graphs(scene: &Scene) -> bool {
    !scene.shader_tree.is_default()
        || !scene.materials.is_empty()
        || !scene.material_pins.is_empty()
}

/// Every polygon context of the scene, in item and polygon order: the
/// items' polygons as the renderer groups them (material tag, part tag
/// and selection sets).
fn each_context(scene: &Scene, mut f: impl FnMut(&Context, &ShaderTree)) {
    let tree = scene.shading_tree();
    for item in &scene.items {
        if !item.is_mesh() || item.mesh.faces.is_empty() {
            continue;
        }
        // Copies shade as their prototype, which is saved for itself.
        if crate::replicator::copy_replicator(item.id).is_some() {
            continue;
        }
        let ancestors = scene.item_ancestors(item.id);
        let locators = scene.locator_frames(item.id);
        let mut sets: Vec<Vec<&str>> = vec![Vec::new(); item.mesh.faces.len()];
        for (name, faces) in &item.mesh.attributes.polygon_sets {
            for &face in faces {
                if let Some(list) = sets.get_mut(face as usize) {
                    list.push(name.as_str());
                }
            }
        }
        let mut seen: std::collections::HashSet<(&str, Option<&str>, &[&str])> = Default::default();
        for (index, face) in item.mesh.faces.iter().enumerate() {
            let key = (
                face.material.as_str(),
                face.part.as_deref(),
                sets[index].as_slice(),
            );
            if !seen.insert(key) {
                continue;
            }
            let ctx = Context {
                item: item.id,
                ancestors: &ancestors,
                material: key.0,
                part: key.1,
                sets: key.2,
                locators: &locators,
            };
            f(&ctx, &tree);
        }
    }
}

// Placement by reference (MXG-Q13). A texture's placement depends on the
// item it lands on: the form holds the locator's matrices as this item
// sees them. So that one material is one graph for every item, the graph
// is written with each matrix entry replaced by a reference number
// (`REF_BASE` + locator id × 128 + matrix × 16 + entry), and an item's
// own matrices are put back when the graph is read for it. Saving checks
// that putting them back gives exactly what the layers give; a surface
// whose sums go through a placement number (so a reference can't stand
// in for it) is saved for its item alone, with its own numbers.

/// The first reference number: far above any placement value, and with
/// whole numbers still exact (below 2^53).
const REF_BASE: f64 = 1e15;
/// Locator ids up to this keep their references exact.
const REF_MAX_ID: u64 = 50_000_000_000_000;

fn ref_number(id: u64, matrix: u64, entry: u64) -> f64 {
    REF_BASE + (id * 128 + matrix * 16 + entry) as f64
}

/// `frames` with every matrix entry replaced by its reference number;
/// `None` when an id is too large to reference.
fn reference_frames(frames: &[LocatorFrame]) -> Option<Vec<LocatorFrame>> {
    use glam::{DMat3, DMat4};
    let mat4 = |id: u64, m: u64| {
        let mut a = [0.; 16];
        for (e, v) in a.iter_mut().enumerate() {
            *v = ref_number(id, m, e as u64);
        }
        DMat4::from_cols_array(&a)
    };
    let mat3 = |id: u64, m: u64| {
        let mut a = [0.; 9];
        for (e, v) in a.iter_mut().enumerate() {
            *v = ref_number(id, m, e as u64);
        }
        DMat3::from_cols_array(&a)
    };
    frames
        .iter()
        .map(|f| {
            (f.id <= REF_MAX_ID).then(|| LocatorFrame {
                id: f.id,
                settings: f.settings.clone(),
                world_to_locator: mat4(f.id, 0),
                world_to_box: mat4(f.id, 1),
                normal_to_locator: mat3(f.id, 2),
                normal_to_space: mat3(f.id, 3),
                front: f.front.map(|_| mat4(f.id, 4)),
            })
        })
        .collect()
}

/// Reference number -> the item's own value, for `frames`.
fn reference_values(frames: &[LocatorFrame]) -> HashMap<u64, f64> {
    let mut out = HashMap::new();
    for f in frames {
        let mut put = |m: u64, values: &[f64]| {
            for (e, v) in values.iter().enumerate() {
                out.insert(ref_number(f.id, m, e as u64) as u64, *v);
            }
        };
        put(0, &f.world_to_locator.to_cols_array());
        put(1, &f.world_to_box.to_cols_array());
        put(2, &f.normal_to_locator.to_cols_array());
        put(3, &f.normal_to_space.to_cols_array());
        if let Some(m) = f.front {
            put(4, &m.to_cols_array());
        }
    }
    out
}

/// A placed graph's text for one item: each reference number written in
/// the form's own way (shortest round-trip text) as the item's value. A
/// reference to a matrix the item doesn't have stays as it is (the read
/// surface then differs, and the caller treats it as such).
fn place(text: &str, frames: &[LocatorFrame]) -> String {
    let values = reference_values(frames);
    let mut out = String::with_capacity(text.len());
    let mut number = String::new();
    let flush = |number: &mut String, out: &mut String| {
        if number.len() == 16
            && number.bytes().all(|b| b.is_ascii_digit())
            && let Ok(n) = number.parse::<u64>()
            && let Some(v) = values.get(&n)
        {
            out.push_str(&format!("{v}"));
        } else {
            out.push_str(number);
        }
        number.clear();
    };
    for c in text.chars() {
        if c.is_ascii_digit() || (c == '-' && number.is_empty()) || c == '.' || c == 'e' {
            number.push(c);
        } else {
            flush(&mut number, &mut out);
            out.push(c);
        }
    }
    flush(&mut number, &mut out);
    out
}

/// The scene's graphs and bindings, for its file. Identical surfaces share
/// one graph, across items too where placement goes by reference.
pub fn graphs(scene: &Scene) -> SavedGraphs {
    let mut out = SavedGraphs::default();
    if !saves_graphs(scene) {
        return out;
    }
    // Placed graphs by their reference surface, with whether placing has
    // been checked to give the layers exactly (`None`: not shareable).
    let mut placed: Vec<(Principled, Option<usize>)> = Vec::new();
    // Graphs saved for one item, by surface.
    let mut own: Vec<(Principled, usize)> = Vec::new();
    let name_of = |ctx: &Context| {
        if ctx.material.is_empty() {
            crate::material::DEFAULT_MATERIAL.to_owned()
        } else {
            ctx.material.to_owned()
        }
    };
    each_context(scene, |ctx, tree| {
        let real = || surface(scene, ctx, tree.evaluate(ctx).principled());
        let pinned = scene.material_pins.0.iter().any(|p| p.key.matches(ctx));
        let reference = if pinned {
            None
        } else {
            reference_frames(ctx.locators)
        };
        let mut graph = None;
        if let Some(frames) = &reference {
            let refs = Context {
                locators: frames,
                ..ctx.clone()
            };
            let p = tree.evaluate(&refs).principled();
            match placed.iter().position(|(s, _)| *s == p) {
                Some(k) => graph = placed[k].1,
                None => {
                    let name = name_of(ctx);
                    let text = tree::write(&p, scene, &name);
                    // Checked once per graph, on its first item.
                    let fits = tree::read(&place(&text, ctx.locators)).ok() == Some(real());
                    let g = fits.then(|| {
                        out.graphs.push(SavedGraph {
                            name,
                            placed: true,
                            materialx: text,
                        });
                        out.graphs.len() - 1
                    });
                    placed.push((p, g));
                    graph = g;
                }
            }
        }
        let graph = graph.unwrap_or_else(|| {
            let p = real();
            match own.iter().find(|(s, _)| *s == p) {
                Some((_, g)) => *g,
                None => {
                    let name = name_of(ctx);
                    out.graphs.push(SavedGraph {
                        materialx: tree::write(&p, scene, &name),
                        name,
                        placed: false,
                    });
                    own.push((p, out.graphs.len() - 1));
                    out.graphs.len() - 1
                }
            }
        });
        let key = Key::of(ctx);
        out.bindings.push(Binding {
            item: key.item,
            material: key.material,
            part: key.part,
            sets: key.sets,
            graph,
        });
    });
    out
}

/// On open: reads the saved graphs and pins each one whose polygons the
/// layers would now render differently. Returns a note per pinned
/// material. A binding whose polygons are gone (the file was edited) is
/// left out; a graph that doesn't read refuses the file, naming it, since
/// opening without it would lose the material.
pub fn open(scene: &mut Scene, saved: &SavedGraphs) -> Result<Vec<String>> {
    let mut read: Vec<Principled> = Vec::with_capacity(saved.graphs.len());
    for g in &saved.graphs {
        read.push(
            tree::read(&g.materialx)
                .map_err(|e| format!("The saved material '{}' can't be read: {e}", g.name))?,
        );
    }
    let mut bindings: HashMap<Key, usize> = HashMap::with_capacity(saved.bindings.len());
    for b in &saved.bindings {
        if b.graph >= read.len() {
            return Err(format!(
                "A material binding names graph {}, but the file has {}",
                b.graph,
                read.len()
            ));
        }
        bindings.insert(
            Key {
                item: b.item,
                material: b.material.clone(),
                part: b.part.clone(),
                sets: b.sets.clone(),
            },
            b.graph,
        );
    }
    let mut pins = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    each_context(scene, |ctx, tree| {
        let Some(&g) = bindings.get(&Key::of(ctx)) else {
            return;
        };
        let saved_graph = &saved.graphs[g];
        if saved_graph.placed
            && let Some(frames) = reference_frames(ctx.locators)
        {
            // The usual case, without placing: the layers by reference
            // match the graph.
            let refs = Context {
                locators: &frames,
                ..ctx.clone()
            };
            if tree.evaluate(&refs).principled() == read[g] {
                return;
            }
        }
        let layers = tree.evaluate(ctx).principled();
        let graph = if saved_graph.placed {
            match tree::read(&place(&saved_graph.materialx, ctx.locators)) {
                Ok(p) => p,
                Err(_) => return,
            }
        } else {
            read[g].clone()
        };
        if graph == layers {
            return;
        }
        let name = saved_graph.name.clone();
        if !notes.iter().any(|n| n.contains(&format!("'{name}'"))) {
            notes.push(format!(
                "Material '{name}' renders from its saved graph: its layers now give a different surface. Edit its layers to render from them again."
            ));
        }
        pins.push(Pin {
            key: Key::of(ctx),
            basis: layers,
            graph,
            name,
        });
    });
    scene.material_pins = Pins(pins);
    Ok(notes)
}
