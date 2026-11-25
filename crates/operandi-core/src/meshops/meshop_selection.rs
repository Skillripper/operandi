//! Selection operations (MO-50..54, PG-xx): an op's Selection input,
//! re-answered on every evaluation. A stack of them sits on each op after
//! its Select by Index (MO-20): each picks polygons of the mesh below and
//! combines with what's under it by its Blend Mode (MO-51).
//!
//! Select by Previous Operation (PG-xx) reads named groups an op lower in
//! the stack produced, such as a primitive's `caps` or an Array's `clone2`.
//! A producing op writes each group as a reserved polygon set on its
//! result ([`OpContext::tag`](crate::meshop_types::OpContext::tag)), so the
//! group follows the polygons through every op above that carries
//! selection sets along. The runtime strips reserved sets from what it
//! hands out (viewport, render, Freeze, links).
//!
//! Each op says what its Selection input picks ([`SelElement`], GitHub
//! #19): polygons, edges (numbered as `Mesh::edges`) or points. Index,
//! Range, Pattern and Random count in that kind; groups, sets and the rule
//! ops pick in their own kind and convert (an edge is picked when one of
//! its polygons is, a polygon when one of its edges is, a point when it
//! is on a picked polygon or edge).
//!
//! Provisional (MOQ-21..24, SEL-Q01..Q04): Range and Pattern channel
//! names, Select by Selection Set's type id and the rule ops' ids are
//! Operandi's own.
use crate::meshop_types::SelElement;
use crate::{Mesh, Result};
use serde::{Deserialize, Serialize};

/// Reserved polygon set names start with this (not typeable).
const GROUP_PREFIX: &str = "\u{1}meshop:";

/// The reserved set that holds op `op`'s group `group` (PG-03 short id).
pub fn group_set(op: u64, group: &str) -> String {
    format!("{GROUP_PREFIX}{op}:{group}")
}

/// `mesh` without reserved group sets; borrowed when it has none.
pub fn strip_groups(mesh: &crate::Shared<Mesh>) -> crate::Shared<Mesh> {
    if !mesh
        .attributes
        .polygon_sets
        .keys()
        .any(|k| k.starts_with(GROUP_PREFIX))
    {
        return mesh.clone();
    }
    let mut m = mesh.to_owned_value();
    m.attributes
        .polygon_sets
        .retain(|k, _| !k.starts_with(GROUP_PREFIX));
    let out = crate::Shared::new(m);
    out.record_checked();
    out
}

/// `mesh` without its group sets, and those sets. Kernels never see the
/// sets (some refuse any selection set); [`carry_groups`] puts them back.
pub fn split_groups(mesh: &Mesh) -> (std::borrow::Cow<'_, Mesh>, Vec<(String, Vec<u32>)>) {
    if !mesh
        .attributes
        .polygon_sets
        .keys()
        .any(|k| k.starts_with(GROUP_PREFIX))
    {
        return (std::borrow::Cow::Borrowed(mesh), Vec::new());
    }
    let mut clean = mesh.clone();
    let mut groups = Vec::new();
    clean.attributes.polygon_sets.retain(|k, v| {
        if k.starts_with(GROUP_PREFIX) {
            groups.push((k.clone(), v.iter().copied().collect()));
            false
        } else {
            true
        }
    });
    (std::borrow::Cow::Owned(clean), groups)
}

/// Puts `groups` of `input` onto `output`: a polygon of the output that
/// has exactly the corners of a grouped input polygon (same point numbers
/// in the same order) is in its groups. Kernels keep untouched polygons
/// that way, so a group follows the polygons an op leaves alone; a
/// polygon an op rebuilt leaves its groups (provisional, MOQ-22).
pub fn carry_groups(groups: &[(String, Vec<u32>)], input: &Mesh, output: &mut Mesh) {
    if groups.is_empty() {
        return;
    }
    let mut at: std::collections::HashMap<&[u32], u32> = std::collections::HashMap::new();
    for (f, face) in input.faces.iter().enumerate() {
        at.entry(face.vertices.as_slice()).or_insert(f as u32);
    }
    let mut map: Vec<Option<u32>> = vec![None; input.faces.len()];
    for (f, face) in output.faces.iter().enumerate() {
        if let Some(&old) = at.get(face.vertices.as_slice())
            && map[old as usize].is_none()
        {
            map[old as usize] = Some(f as u32);
        }
    }
    for (name, faces) in groups {
        if output.attributes.polygon_sets.contains_key(name) {
            continue;
        }
        let set: std::collections::BTreeSet<u32> = faces
            .iter()
            .filter_map(|&f| map.get(f as usize).copied().flatten())
            .collect();
        output.attributes.polygon_sets.insert(name.clone(), set);
    }
}

/// MO-51 Blend Mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Blend {
    /// Replaces what's below it.
    #[default]
    Override,
    Add,
    Subtract,
    /// Keeps only what both select.
    Multiply,
}
impl Blend {
    pub const ALL: [Blend; 4] = [Self::Override, Self::Add, Self::Subtract, Self::Multiply];
    pub fn label(self) -> &'static str {
        match self {
            Self::Override => "Override",
            Self::Add => "Add",
            Self::Subtract => "Subtract",
            Self::Multiply => "Multiply",
        }
    }
}

/// What a selection op picks (MO-53).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SelKind {
    /// Stored polygon numbers, `0-9,12` form.
    #[serde(rename = "index.selop.item")]
    Index { index: String },
    /// Every `step`th polygon from `start` to `end` (inclusive; 0 = the
    /// last polygon).
    #[serde(rename = "selbyrange.selop.item")]
    Range { start: u32, end: u32, step: u32 },
    /// A repeating on/off list: `1,0` = every other polygon. Accepts
    /// 1/0, True/False, Yes/No.
    #[serde(rename = "pattern.selop.item")]
    Pattern { pattern: String },
    /// A group an op lower in the stack produced (PG-01..04): `source` is
    /// that op's id, `name` its short group id (case-sensitive).
    #[serde(rename = "previous.selop")]
    Previous { source: u64, name: String },
    /// The polygons of a selection set on the mesh below.
    #[serde(rename = "operandi.selset.selop")]
    SelectionSet { name: String },
    /// Random selection
    #[serde(rename = "random.selop")]
    Random { seed: u32, probability: f32 },
    /// Edges whose two polygons meet at between `min` and `max` degrees
    /// (0 = flat). Rebuilt on every evaluation, so it survives topology
    /// changes below (#19, Steve Hill's Selma idea).
    #[serde(rename = "operandi.edgeAngle.selop")]
    EdgeAngle { min: f64, max: f64 },
    /// Edges on an open border (one polygon).
    #[serde(rename = "operandi.border.selop")]
    Border {},
    /// Polygons by corner count: 0 triangles, 1 quads, 2 n-gons.
    #[serde(rename = "operandi.sides.selop")]
    Sides { sides: u32 },
    /// Polygons facing within `angle` degrees of the direction `x y z`.
    #[serde(rename = "operandi.facing.selop")]
    Facing { x: f64, y: f64, z: f64, angle: f64 },
}
impl SelKind {
    /// The kinds Add Selection offers, with their labels.
    pub fn menu() -> [(&'static str, SelKind); 10] {
        [
            (
                "Select by Index",
                SelKind::Index {
                    index: String::new(),
                },
            ),
            (
                "Select by Range",
                SelKind::Range {
                    start: 0,
                    end: 0,
                    step: 1,
                },
            ),
            (
                "Select by Pattern",
                SelKind::Pattern {
                    pattern: "1,0".into(),
                },
            ),
            (
                "Select by Previous Operation",
                SelKind::Previous {
                    source: 0,
                    name: String::new(),
                },
            ),
            (
                "Select by Selection Set",
                SelKind::SelectionSet {
                    name: String::new(),
                },
            ),
            (
                "Select Randomly",
                SelKind::Random {
                    seed: 314159265,
                    probability: 50.0,
                },
            ),
            (
                "Select Edges by Angle",
                SelKind::EdgeAngle {
                    min: 30.,
                    max: 180.,
                },
            ),
            ("Select Border", SelKind::Border {}),
            ("Select by Polygon Sides", SelKind::Sides { sides: 1 }),
            (
                "Select by Facing",
                SelKind::Facing {
                    x: 0.,
                    y: 1.,
                    z: 0.,
                    angle: 45.,
                },
            ),
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Index { .. } => "Select by Index",
            Self::Range { .. } => "Select by Range",
            Self::Pattern { .. } => "Select by Pattern",
            Self::Previous { .. } => "Select by Previous Operation",
            Self::SelectionSet { .. } => "Select by Selection Set",
            Self::Random { .. } => "Select Randomly",
            Self::EdgeAngle { .. } => "Select Edges by Angle",
            Self::Border {} => "Select Border",
            Self::Sides { .. } => "Select by Polygon Sides",
            Self::Facing { .. } => "Select by Facing",
        }
    }
}

/// One selection op (MO-51 common settings + its kind).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelOp {
    pub enable: bool,
    pub invert: bool,
    #[serde(rename = "blendmode")]
    pub blend: Blend,
    #[serde(flatten)]
    pub kind: SelKind,
}
impl SelOp {
    pub fn new(kind: SelKind) -> Self {
        Self {
            enable: true,
            invert: false,
            blend: Blend::Override,
            kind,
        }
    }
}

fn truthy(word: &str) -> Option<bool> {
    match word.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Some(true),
        "0" | "false" | "no" => Some(false),
        _ => None,
    }
}

/// A mesh's elements as one kind sees them: the edge list and which edge
/// numbers each polygon uses, worked out once per resolve.
struct Elements<'a> {
    mesh: &'a Mesh,
    element: SelElement,
    edges: Vec<(u32, u32)>,
    edge_at: std::collections::HashMap<(u32, u32), u32>,
}
impl<'a> Elements<'a> {
    fn new(mesh: &'a Mesh, element: SelElement) -> Self {
        let edges = mesh.edges();
        let edge_at = edges
            .iter()
            .enumerate()
            .map(|(i, &e)| (e, i as u32))
            .collect();
        Self {
            mesh,
            element,
            edges,
            edge_at,
        }
    }
    fn count(&self) -> usize {
        match self.element {
            SelElement::Polygon => self.mesh.faces.len(),
            SelElement::Edge => self.edges.len(),
            SelElement::Point => self.mesh.vertices.len(),
        }
    }
    fn face_edges(&self, f: usize) -> impl Iterator<Item = u32> + '_ {
        self.mesh
            .element_spans(f)
            .filter_map(|(a, b)| self.edge_at.get(&(a.min(b), a.max(b))).copied())
    }
    /// Picked polygons as this kind.
    fn of_polygons(&self, faces: &[bool]) -> Vec<bool> {
        let mut out = vec![false; self.count()];
        for (f, _) in faces.iter().enumerate().filter(|(_, p)| **p) {
            match self.element {
                SelElement::Polygon => out[f] = true,
                SelElement::Edge => self.face_edges(f).for_each(|e| out[e as usize] = true),
                SelElement::Point => {
                    for &v in &self.mesh.faces[f].vertices {
                        out[v as usize] = true;
                    }
                }
            }
        }
        out
    }
    /// Picked edges (numbered as `edges`) as this kind.
    fn of_edges(&self, picked: &[bool]) -> Vec<bool> {
        match self.element {
            SelElement::Edge => picked.to_vec(),
            SelElement::Point => {
                let mut out = vec![false; self.count()];
                for (i, _) in picked.iter().enumerate().filter(|(_, p)| **p) {
                    let (a, b) = self.edges[i];
                    out[a as usize] = true;
                    out[b as usize] = true;
                }
                out
            }
            SelElement::Polygon => (0..self.mesh.faces.len())
                .map(|f| self.face_edges(f).any(|e| picked[e as usize]))
                .collect(),
        }
    }
    fn polygon_set(&self, set: &std::collections::BTreeSet<u32>) -> Vec<bool> {
        let mut faces = vec![false; self.mesh.faces.len()];
        for &f in set {
            if let Some(o) = faces.get_mut(f as usize) {
                *o = true;
            }
        }
        self.of_polygons(&faces)
    }
    /// Each edge's polygons (one for a border edge).
    fn edge_faces(&self) -> Vec<Vec<u32>> {
        let mut out = vec![Vec::new(); self.edges.len()];
        for f in 0..self.mesh.faces.len() {
            for e in self.face_edges(f) {
                out[e as usize].push(f as u32);
            }
        }
        out
    }
}

impl SelOp {
    /// The elements of `mesh` this op picks, before Invert and Blend.
    fn picks(&self, el: &Elements) -> Result<Vec<bool>> {
        let mesh = el.mesh;
        let n = el.count();
        let what = el.element.plural();
        let mut out = vec![false; n];
        match &self.kind {
            SelKind::Index { index } => {
                let s = crate::mesh_stack::SelectByIndex::parse(index)?;
                if let Some(&bad) = s.polygons.iter().find(|&&f| f as usize >= n) {
                    return Err(format!(
                        "Select by Index names number {bad}, but the mesh below has {n} {what}"
                    ));
                }
                for f in s.polygons {
                    out[f as usize] = true;
                }
            }
            SelKind::Range { start, end, step } => {
                if *step == 0 {
                    return Err("Select by Range needs a Step of at least 1".into());
                }
                if n > 0 {
                    let end = if *end == 0 {
                        n - 1
                    } else {
                        (*end as usize).min(n - 1)
                    };
                    let mut i = *start as usize;
                    while i <= end {
                        out[i] = true;
                        i += *step as usize;
                    }
                }
            }
            SelKind::Pattern { pattern } => {
                let bits: Vec<bool> = pattern
                    .split(',')
                    .filter(|w| !w.trim().is_empty())
                    .map(|w| {
                        truthy(w).ok_or_else(|| {
                            format!(
                                "Select by Pattern: “{}” isn't 1/0, True/False or Yes/No",
                                w.trim()
                            )
                        })
                    })
                    .collect::<Result<_>>()?;
                if !bits.is_empty() {
                    for (i, o) in out.iter_mut().enumerate() {
                        *o = bits[i % bits.len()];
                    }
                }
            }
            SelKind::Previous { source, name } => {
                // PG-04: an unknown or empty name selects nothing.
                if let Some(set) = mesh.attributes.polygon_sets.get(&group_set(*source, name)) {
                    out = el.polygon_set(set);
                }
            }
            SelKind::SelectionSet { name } => {
                // The set of this kind, else a polygon set of that name.
                match el.element {
                    SelElement::Edge if mesh.attributes.edge_sets.contains_key(name) => {
                        for e in &mesh.attributes.edge_sets[name] {
                            if let Some(&i) = el.edge_at.get(&(e.0.min(e.1), e.0.max(e.1))) {
                                out[i as usize] = true;
                            }
                        }
                    }
                    SelElement::Point if mesh.attributes.vertex_sets.contains_key(name) => {
                        for &v in &mesh.attributes.vertex_sets[name] {
                            if let Some(o) = out.get_mut(v as usize) {
                                *o = true;
                            }
                        }
                    }
                    _ => {
                        if let Some(set) = mesh.attributes.polygon_sets.get(name) {
                            out = el.polygon_set(set);
                        }
                    }
                }
            }
            SelKind::Random { seed, probability } => {
                // Each element's draw comes from the seed and its own index
                // (SplitMix64), not from a library generator, so a saved
                // scene picks the same elements on every version and a
                // pick doesn't depend on the ones before it.
                let p = f64::from(*probability) / 100.0;
                for (i, o) in out.iter_mut().enumerate() {
                    let key = (u64::from(*seed) << 32) | i as u64;
                    *o = crate::vertex_map_texture::random01(key) < p;
                }
            }
            SelKind::EdgeAngle { min, max } => {
                if !(min.is_finite() && max.is_finite()) {
                    return Err("Select Edges by Angle needs finite Min and Max angles".into());
                }
                let (lo, hi) = (min.min(*max), min.max(*max));
                let normals: Vec<glam::DVec3> =
                    (0..mesh.faces.len()).map(|f| mesh.normal(f)).collect();
                let picked: Vec<bool> = el
                    .edge_faces()
                    .iter()
                    .map(|faces| {
                        // The angle between the two polygons' facings:
                        // 0 flat, 90 a right-angle corner.
                        let [a, b] = faces[..] else {
                            return false;
                        };
                        let (na, nb) = (normals[a as usize], normals[b as usize]);
                        if na == glam::DVec3::ZERO || nb == glam::DVec3::ZERO {
                            return false;
                        }
                        let angle = na.dot(nb).clamp(-1., 1.).acos().to_degrees();
                        angle >= lo - 1e-9 && angle <= hi + 1e-9
                    })
                    .collect();
                out = el.of_edges(&picked);
            }
            SelKind::Border {} => {
                let picked: Vec<bool> = el.edge_faces().iter().map(|f| f.len() == 1).collect();
                out = el.of_edges(&picked);
            }
            SelKind::Sides { sides } => {
                if *sides > 2 {
                    return Err(
                        "Select by Polygon Sides: Sides is 0 (triangles), 1 (quads) or 2 (n-gons)"
                            .into(),
                    );
                }
                let faces: Vec<bool> = (0..mesh.faces.len())
                    .map(|f| {
                        let k = mesh.faces[f].vertices.len();
                        !mesh.is_polyline(f)
                            && match sides {
                                0 => k == 3,
                                1 => k == 4,
                                _ => k > 4,
                            }
                    })
                    .collect();
                out = el.of_polygons(&faces);
            }
            SelKind::Facing { x, y, z, angle } => {
                let dir = glam::DVec3::new(*x, *y, *z);
                if !dir.is_finite() || dir == glam::DVec3::ZERO || !angle.is_finite() {
                    return Err("Select by Facing needs a direction and a finite Angle".into());
                }
                let dir = dir.normalize();
                let limit = angle.clamp(0., 180.).to_radians().cos();
                let faces: Vec<bool> = (0..mesh.faces.len())
                    .map(|f| {
                        let nrm = mesh.normal(f);
                        nrm != glam::DVec3::ZERO && nrm.dot(dir) >= limit - 1e-12
                    })
                    .collect();
                out = el.of_polygons(&faces);
            }
        }
        Ok(out)
    }
}

/// An op's Selection input on `mesh`, as `element`s: `None` when it has
/// none (the op works on the whole mesh, PG-05), otherwise the picked
/// elements, which may be none at all (PG-04).
pub fn resolve(
    index: Option<&crate::mesh_stack::SelectByIndex>,
    selops: &[SelOp],
    mesh: &Mesh,
    element: SelElement,
) -> Result<Option<Vec<u32>>> {
    let index = index.filter(|s| !s.polygons.is_empty());
    // Turned-off selection ops act as if they weren't there.
    if index.is_none() && !selops.iter().any(|s| s.enable) {
        return Ok(None);
    }
    let el = Elements::new(mesh, element);
    let n = el.count();
    let mut sel = vec![false; n];
    if let Some(s) = index {
        if let Some(&bad) = s.polygons.iter().find(|&&f| f as usize >= n) {
            return Err(format!(
                "Select by Index names number {bad}, but the mesh below has {n} {}",
                element.plural()
            ));
        }
        for &f in &s.polygons {
            sel[f as usize] = true;
        }
    }
    for s in selops.iter().filter(|s| s.enable) {
        let mut mine = s.picks(&el)?;
        if s.invert {
            mine.iter_mut().for_each(|b| *b = !*b);
        }
        for (a, b) in sel.iter_mut().zip(mine) {
            *a = match s.blend {
                Blend::Override => b,
                Blend::Add => *a || b,
                Blend::Subtract => *a && !b,
                Blend::Multiply => *a && b,
            };
        }
    }
    Ok(Some(
        sel.iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .map(|(i, _)| i as u32)
            .collect(),
    ))
}
