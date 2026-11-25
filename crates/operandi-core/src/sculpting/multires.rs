//! Multiresolution sculpting (provisional: no behaviour spec yet, questions
//! MR-Q01..MR-Q08 in docs/SPECIFICATION_QUESTIONS.md).
//!
//! A multiresolution mesh is a base cage plus levels of Catmull-Clark
//! subdivision you can step between and sculpt at. Level k's points are the
//! smoothed level below plus a detail offset per point:
//!
//! ```text
//! P0 = base cage points
//! Pk = S(Pk-1) + F(S(Pk-1)) · Dk        (k = 1..N)
//! ```
//!
//! `S` is one Catmull-Clark step of the whole cage (creases and the mesh's
//! Boundary Rule as at Add Level), `Dk` the detail of level k and `F` a
//! frame at each point of the smoothed surface (its normal, and a tangent
//! toward its first neighbour). Details are stored in those frames, so
//! detail sculpted at a high level rides along when a lower level is
//! reshaped: step down, move the cage or sculpt big forms, step back up and
//! the fine detail sits on the new shape.
//!
//! The item's own `mesh` always holds the level being worked at, as an
//! ordinary mesh, so every brush, the viewport, picking, export and render
//! work on it unchanged. `Multires` keeps everything else: the base cage
//! (with all its maps and attributes), and the detail of every level. The
//! level in `mesh` is the truth for that level; stepping, adding or
//! deleting a level folds it back first (`folded`). Each command is one
//! undo step of the whole item, and the data saves with the scene (native
//! version 64; big detail arrays are packed like mesh points).
//!
//! If a tool changes the polygons of the mesh while it is multiresolution,
//! the stored detail can no longer follow: stepping refuses with a reason,
//! and Apply Multiresolution keeps the mesh as it is.
use crate::subdivision::{self, BoundaryRule};
use crate::topology::edge_key;
use crate::{Editor, Face, Mesh, Result};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Most levels above the base (an Operandi budget, MR-Q03).
pub const MAX_LEVELS: u8 = 6;
/// Most polygons a new level may have (an Operandi budget, MR-Q03).
pub const MAX_POLYGONS: usize = 8_000_000;

/// A mesh item's multiresolution data (`Item::multires`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Multires {
    /// The level the item's mesh holds now: 0 is the base cage.
    pub current: u8,
    /// The Boundary Rule the levels were built with.
    #[serde(default)]
    pub boundary_rule: BoundaryRule,
    /// The base cage, with its maps and attributes. While `current` is 0
    /// the item's mesh is the base and this copy may be behind it.
    pub base: Mesh,
    /// Levels 1..=N, lowest first.
    #[serde(default)]
    pub levels: Vec<Level>,
}

/// One level's detail: an offset per point of that level, in the point's
/// frame on the smoothed lower level (x along the tangent, y across it,
/// z along the normal).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level {
    #[serde(with = "crate::mesh_pack::points")]
    pub detail: Vec<DVec3>,
}

/// Why stepping refuses when the mesh's polygons changed under it.
pub const STALE: &str = "This mesh's polygons changed since its multiresolution level was set, so the stored detail can't follow; Apply Multiresolution keeps the mesh as it is";

/// A polygon net being refined: points, polygons, edge sharpness.
#[derive(Clone)]
struct Net {
    points: Vec<DVec3>,
    faces: Vec<Vec<u32>>,
    sharp: BTreeMap<(u32, u32), f64>,
}

fn base_net(base: &Mesh) -> Result<Net> {
    if base.has_polylines() {
        return Err("Multiresolution needs a mesh of polygons only; this one has lines".into());
    }
    if base.faces.is_empty() {
        return Err("The mesh has no polygons to subdivide".into());
    }
    let sharp = base
        .attributes
        .creases
        .iter()
        .map(|c| (edge_key(c.a, c.b), 10.0 * c.weight.clamp(0.0, 1.0)))
        .filter(|&(_, s)| s > 0.0)
        .collect();
    Ok(Net {
        points: base.vertices.clone(),
        faces: base.faces.iter().map(|f| f.vertices.clone()).collect(),
        sharp,
    })
}

fn step(net: Net, rule: BoundaryRule) -> Net {
    let (points, faces, sharp) =
        subdivision::catmull_clark_net(net.points, net.faces, net.sharp, rule);
    Net {
        points,
        faces,
        sharp,
    }
}

/// Each point's frame on `points` (tangent, bitangent, normal), or `None`
/// where the surface gives none (a loose or flat-folded point): its detail
/// is then a plain offset.
fn frames(points: &[DVec3], faces: &[Vec<u32>]) -> Vec<Option<[DVec3; 3]>> {
    let mut normal = vec![DVec3::ZERO; points.len()];
    let mut toward = vec![u32::MAX; points.len()];
    for f in faces {
        let k = f.len();
        let mut n = DVec3::ZERO;
        for i in 0..k {
            let a = points[f[i] as usize];
            let b = points[f[(i + 1) % k] as usize];
            n += a.cross(b);
        }
        for i in 0..k {
            let v = f[i] as usize;
            normal[v] += n;
            if toward[v] == u32::MAX {
                toward[v] = f[(i + 1) % k];
            }
        }
    }
    (0..points.len())
        .map(|v| {
            let n = normal[v].normalize_or_zero();
            if n == DVec3::ZERO || toward[v] == u32::MAX {
                return None;
            }
            let d = points[toward[v] as usize] - points[v];
            let t = (d - n * d.dot(n)).normalize_or_zero();
            if t == DVec3::ZERO {
                return None;
            }
            Some([t, n.cross(t), n])
        })
        .collect()
}

/// `smooth` plus each point's detail.
fn with_detail(smooth: &[DVec3], faces: &[Vec<u32>], detail: &[DVec3]) -> Vec<DVec3> {
    let frames = frames(smooth, faces);
    smooth
        .iter()
        .zip(detail)
        .zip(frames)
        .map(|((&q, &d), f)| match f {
            Some([t, b, n]) => q + t * d.x + b * d.y + n * d.z,
            None => q + d,
        })
        .collect()
}

/// The detail that takes `smooth` to `points`.
fn detail_of(smooth: &[DVec3], faces: &[Vec<u32>], points: &[DVec3]) -> Vec<DVec3> {
    let frames = frames(smooth, faces);
    smooth
        .iter()
        .zip(points)
        .zip(frames)
        .map(|((&q, &p), f)| {
            let d = p - q;
            match f {
                Some([t, b, n]) => DVec3::new(d.dot(t), d.dot(b), d.dot(n)),
                None => d,
            }
        })
        .collect()
}

/// Whether `mesh` has exactly `net`'s points and polygons.
fn same_topology(mesh: &Mesh, net: &Net) -> bool {
    mesh.vertices.len() == net.points.len()
        && mesh.faces.len() == net.faces.len()
        && mesh
            .faces
            .iter()
            .zip(&net.faces)
            .all(|(a, b)| a.vertices == *b)
}

/// Points and polygons per level, from the base's: (points, edges,
/// polygons, corners) of level k + 1 follow from level k's.
fn counts(base: &Mesh, levels: usize) -> Vec<usize> {
    let corners: usize = base.faces.iter().map(|f| f.vertices.len()).sum();
    let (mut v, mut e, mut f, mut c) = (
        base.vertices.len(),
        base.edges().len(),
        base.faces.len(),
        corners,
    );
    let mut out = vec![v];
    for _ in 0..levels {
        (v, e, f, c) = (v + e + f, 2 * e + c, c, 4 * c);
        out.push(v);
    }
    out
}

/// Cheap checks of saved or scripted data (memoised per allocation).
pub fn validate(m: &crate::Shared<Multires>) -> Result<()> {
    m.checked(|m| {
        if m.levels.len() > MAX_LEVELS as usize || m.current as usize > m.levels.len() {
            return Err("Invalid multiresolution level".into());
        }
        m.base.validate()?;
        if !m.levels.is_empty() && m.base.has_polylines() {
            return Err("A multiresolution base holds polygons only".into());
        }
        let counts = counts(&m.base, m.levels.len());
        for (k, level) in m.levels.iter().enumerate() {
            if level.detail.len() != counts[k + 1] {
                return Err(format!(
                    "Multiresolution level {} has {} detail points; it needs {}",
                    k + 1,
                    level.detail.len(),
                    counts[k + 1]
                ));
            }
            if level
                .detail
                .iter()
                .any(|d| !d.is_finite() || d.abs().max_element() > 1e9)
            {
                return Err("Multiresolution detail must be finite".into());
            }
        }
        Ok(())
    })
}

impl Multires {
    /// A new multiresolution mesh: `mesh` as its base, no levels yet.
    pub fn new(mesh: &Mesh, rule: BoundaryRule) -> Self {
        Self {
            current: 0,
            boundary_rule: rule,
            base: mesh.clone(),
            levels: Vec::new(),
        }
    }

    /// Levels above the base.
    pub fn top(&self) -> u8 {
        self.levels.len() as u8
    }

    /// Climb from level `from`'s net to level `to`.
    fn climb(&self, mut net: Net, from: u8, to: u8) -> Net {
        for k in from + 1..=to {
            net = step(net, self.boundary_rule);
            let detail = &self.levels[k as usize - 1].detail;
            net.points = with_detail(&net.points, &net.faces, detail);
        }
        net
    }

    fn net_at(&self, k: u8) -> Result<Net> {
        Ok(self.climb(base_net(&self.base)?, 0, k))
    }

    /// Level `k`'s points with no detail of its own: S(Pk-1).
    fn smooth_at(&self, k: u8) -> Result<Net> {
        debug_assert!(k > 0);
        Ok(step(self.net_at(k - 1)?, self.boundary_rule))
    }

    /// This data with `mesh`, the level now being worked at, folded back
    /// in, and that level's net. Refuses when `mesh`'s polygons no longer
    /// match the level's (STALE).
    fn folded(&self, mesh: &Mesh) -> Result<(Self, Net)> {
        let mut out = self.clone();
        if self.current == 0 {
            if !self.levels.is_empty() {
                let base = base_net(&self.base)?;
                if !same_topology(mesh, &base) {
                    return Err(STALE.into());
                }
            }
            out.base = mesh.clone();
            let net = base_net(&out.base)?;
            return Ok((out, net));
        }
        let mut net = self.smooth_at(self.current)?;
        if !same_topology(mesh, &net) {
            return Err(STALE.into());
        }
        out.levels[self.current as usize - 1].detail =
            detail_of(&net.points, &net.faces, &mesh.vertices);
        net.points = mesh.vertices.clone();
        Ok((out, net))
    }

    /// Level `k`'s mesh from its net: the base itself at 0; above it the
    /// refined polygons, each keeping its base polygon's material, part and
    /// smoothing, and the base's UV maps subdivided linearly (MR-Q04).
    fn level_mesh(&self, k: u8, net: &Net) -> Result<Mesh> {
        if k == 0 {
            let mut mesh = self.base.clone();
            mesh.vertices = net.points.clone();
            return Ok(mesh);
        }
        let per_corner = 4usize.pow(k as u32 - 1);
        let mut faces: Vec<Face> = Vec::with_capacity(net.faces.len());
        let mut refined = net.faces.iter();
        for source in self.base.faces.iter() {
            for _ in 0..source.vertices.len() * per_corner {
                let vertices = refined
                    .next()
                    .ok_or("Multiresolution polygons don't match")?;
                faces.push(Face {
                    vertices: vertices.clone(),
                    uv: None,
                    material: source.material.clone(),
                    smooth: source.smooth,
                    part: source.part.clone(),
                    material_label: source.material_label.clone(),
                });
            }
        }
        let uv_maps: BTreeMap<_, _> = self
            .base
            .uv_maps
            .iter()
            .map(|(name, map)| {
                let mut out = Vec::with_capacity(faces.len());
                for corners in map {
                    subdivision::linear_uvs(corners, k, false, &mut out);
                }
                (name.clone(), out)
            })
            .collect();
        let mesh = Mesh {
            vertices: net.points.clone(),
            faces: faces.into(),
            uv_maps: uv_maps.into(),
            ..Default::default()
        };
        mesh.validate()?;
        Ok(mesh)
    }

    /// Level `current`'s points with no detail of its own (what Attenuate
    /// fades toward), when `mesh` still matches the level. `None` at the
    /// base.
    pub fn rest_points(&self, mesh: &Mesh) -> Option<Vec<DVec3>> {
        if self.current == 0 {
            return None;
        }
        let net = self.smooth_at(self.current).ok()?;
        same_topology(mesh, &net).then_some(net.points)
    }

    /// Polygons at each level, base first.
    pub fn polygon_counts(&self) -> Vec<usize> {
        let mut out = vec![self.base.faces.len()];
        let mut corners: usize = self.base.faces.iter().map(|f| f.vertices.len()).sum();
        for _ in 0..self.levels.len() {
            out.push(corners);
            corners *= 4;
        }
        out
    }
}

/// Whether the item's mesh still matches its multiresolution level (a
/// cheap count check; stepping checks every polygon).
pub fn in_step(item: &crate::Item) -> bool {
    let Some(m) = &item.multires else {
        return false;
    };
    let polygons = m.polygon_counts()[m.current as usize];
    if m.current == 0 {
        return m.levels.is_empty()
            || (item.mesh.faces.len() == polygons
                && item.mesh.vertices.len() == m.base.vertices.len());
    }
    item.mesh.faces.len() == polygons
        && item.mesh.vertices.len() == counts(&m.base, m.current as usize)[m.current as usize]
}

/// What a level command changes: the level to show, or a structural edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelChange {
    /// Add a level above the top and step to it.
    Add,
    /// Step to this level.
    To(u8),
    /// One level up or down.
    Up,
    Down,
    /// Delete the levels above the current one.
    DeleteHigher,
    /// Keep the mesh at the current level as a plain mesh.
    Apply,
}

impl Editor {
    /// The mesh item a multiresolution command acts on: `item`, or the
    /// active one.
    fn multires_target(&self, item: Option<u64>) -> Result<u64> {
        let id = item
            .or(self.scene.selection.item)
            .ok_or("Select a mesh first")?;
        let it = self.scene.item(id).ok_or("No such item")?;
        if !it.is_mesh() {
            return Err("Multiresolution works on mesh items".into());
        }
        if it.instance_of.is_some() {
            return Err("A Mesh Instance shows its source's mesh; add levels to the source".into());
        }
        Ok(id)
    }

    /// Why a level command can't run on `item` (or the active mesh) now;
    /// `None` when it can. For greying buttons (UIC-223).
    pub fn multires_unavailable(&self, change: LevelChange, item: Option<u64>) -> Option<String> {
        let id = match self.multires_target(item) {
            Ok(id) => id,
            Err(e) => return Some(e),
        };
        let it = self.scene.item(id)?;
        let m = it.multires.as_deref();
        let (current, top) = m.map_or((0, 0), |m| (m.current, m.top()));
        if m.is_some() && !in_step(it) && change != LevelChange::Apply {
            return Some(STALE.into());
        }
        match change {
            LevelChange::Add if top >= MAX_LEVELS => {
                Some(format!("Multiresolution goes up to {MAX_LEVELS} levels"))
            }
            LevelChange::Add if it.mesh.has_polylines() && m.is_none() => {
                Some("Multiresolution needs a mesh of polygons only".into())
            }
            LevelChange::Add if it.mesh.faces.is_empty() => {
                Some("The mesh has no polygons to subdivide".into())
            }
            LevelChange::Add => None,
            _ if m.is_none() => {
                Some("The mesh has no multiresolution levels; Add Level first".into())
            }
            LevelChange::Up if current >= top => Some("Already at the highest level".into()),
            LevelChange::Down if current == 0 => Some("Already at the base level".into()),
            LevelChange::To(k) if k > top => Some(format!("The mesh has levels 0 to {top}")),
            LevelChange::DeleteHigher if current >= top => Some("No levels above this one".into()),
            _ => None,
        }
    }

    /// Add, step, delete or apply multiresolution levels on `item` (or the
    /// active mesh): one undo step.
    pub fn multires_change(&mut self, change: LevelChange, item: Option<u64>) -> Result<()> {
        self.subdivision_ready("changing multiresolution levels")?;
        let id = self.multires_target(item)?;
        let it = self.scene.item(id).ok_or("No such item")?;
        let rule = it.subdivision.clone().unwrap_or_default().boundary_rule;
        let mut data = match (&it.multires, change) {
            (Some(m), _) => (**m).clone(),
            (None, LevelChange::Add) => Multires::new(&it.mesh, rule),
            (None, _) => {
                return Err("The mesh has no multiresolution levels; Add Level first".into());
            }
        };
        let (label, mesh, keep): (String, Option<Mesh>, bool) = match change {
            LevelChange::Apply => ("Apply Multiresolution".into(), None, false),
            _ => {
                let (folded, net) = data.folded(&it.mesh)?;
                data = folded;
                let (current, top) = (data.current, data.top());
                match change {
                    LevelChange::Add => {
                        if top >= MAX_LEVELS {
                            return Err(format!("Multiresolution goes up to {MAX_LEVELS} levels"));
                        }
                        let net = if current == top {
                            net
                        } else {
                            data.climb(net, current, top)
                        };
                        let polygons: usize = net.faces.iter().map(Vec::len).sum();
                        if polygons > MAX_POLYGONS {
                            return Err(format!(
                                "Level {} would have {} polygons; multiresolution stops at {}",
                                top + 1,
                                crate::polygon_subdivide::group_thousands(polygons as u64),
                                crate::polygon_subdivide::group_thousands(MAX_POLYGONS as u64)
                            ));
                        }
                        let next = step(net, data.boundary_rule);
                        data.levels.push(Level {
                            detail: vec![DVec3::ZERO; next.points.len()],
                        });
                        data.current = top + 1;
                        let mesh = data.level_mesh(top + 1, &next)?;
                        (
                            format!("Add Multiresolution Level {}", top + 1),
                            Some(mesh),
                            true,
                        )
                    }
                    LevelChange::DeleteHigher => {
                        if current >= top {
                            return Err("No levels above this one".into());
                        }
                        data.levels.truncate(current as usize);
                        // Nothing left above the base: a plain mesh again.
                        let keep = !data.levels.is_empty();
                        ("Delete Higher Multiresolution Levels".into(), None, keep)
                    }
                    LevelChange::Up | LevelChange::Down | LevelChange::To(_) => {
                        let to = match change {
                            LevelChange::Up if current < top => current + 1,
                            LevelChange::Up => return Err("Already at the highest level".into()),
                            LevelChange::Down if current > 0 => current - 1,
                            LevelChange::Down => {
                                return Err("Already at the base level".into());
                            }
                            LevelChange::To(k) if k <= top => k,
                            _ => return Err(format!("The mesh has levels 0 to {top}")),
                        };
                        if to == current {
                            return Ok(());
                        }
                        let net = if to > current {
                            data.climb(net, current, to)
                        } else if to == 0 {
                            base_net(&data.base)?
                        } else {
                            data.net_at(to)?
                        };
                        data.current = to;
                        let mesh = data.level_mesh(to, &net)?;
                        (format!("Multiresolution Level {to}"), Some(mesh), true)
                    }
                    LevelChange::Apply => unreachable!("handled above"),
                }
            }
        };
        let mut candidate = self.scene.clone();
        let changes_topology = mesh.is_some();
        {
            let it = candidate.item_mut(id).ok_or("No such item")?;
            if let Some(mesh) = mesh {
                it.mesh = mesh.into();
                // MR-Q05 (provisional): locks and hidden elements name the
                // old level's points and polygons; they're cleared.
                it.elements = Default::default();
            }
            it.multires = keep.then(|| crate::Shared::new(data));
        }
        if changes_topology {
            if candidate.selection.item == Some(id) {
                candidate.selection.vertices.clear();
                candidate.selection.edges.clear();
                candidate.selection.faces.clear();
                if let Some(tag) = candidate.selection.tag.as_mut() {
                    tag.regions.clear();
                }
            }
            candidate.selection.others.remove(&id);
        }
        candidate.validate()?;
        let before = std::mem::replace(&mut self.scene, candidate);
        self.record(label, before);
        Ok(())
    }

    /// `operandi.multires.query`: the item's levels as JSON.
    pub fn multires_report(&self, item: Option<u64>) -> Result<serde_json::Value> {
        let id = self.multires_target(item)?;
        let it = self.scene.item(id).ok_or("No such item")?;
        Ok(match &it.multires {
            None => serde_json::json!({ "item": id, "multiresolution": false }),
            Some(m) => serde_json::json!({
                "item": id,
                "multiresolution": true,
                "level": m.current,
                "levels": m.top(),
                "polygons": m.polygon_counts(),
                "in_step": in_step(it),
            }),
        })
    }
}

type Args = serde_json::Map<String, serde_json::Value>;

fn item_arg(a: &Args) -> std::result::Result<Option<u64>, String> {
    match a.get("item") {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
            .map(Some)
            .ok_or_else(|| "item is an item id".into()),
    }
}

/// The level command `name` with arguments `a`.
pub fn level_change(name: &str, a: &Args) -> std::result::Result<LevelChange, String> {
    Ok(match name {
        "operandi.multires.subdivide" => LevelChange::Add,
        "operandi.multires.deleteHigher" => LevelChange::DeleteHigher,
        "operandi.multires.apply" => LevelChange::Apply,
        "operandi.multires.level" => {
            let level = a.get("level").filter(|v| !v.is_null());
            let step = a.get("step").and_then(serde_json::Value::as_str);
            match (level, step) {
                (Some(_), Some(_)) => return Err("Give level or step, not both".into()),
                (Some(v), None) => {
                    let k = v
                        .as_u64()
                        .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
                        .filter(|&k| k <= MAX_LEVELS as u64)
                        .ok_or_else(|| format!("level is a whole number, 0 to {MAX_LEVELS}"))?;
                    LevelChange::To(k as u8)
                }
                (None, Some(s)) if s.eq_ignore_ascii_case("up") => LevelChange::Up,
                (None, Some(s)) if s.eq_ignore_ascii_case("down") => LevelChange::Down,
                (None, Some(s)) => return Err(format!("step is up or down, not {s}")),
                (None, None) => return Err("Give level:<n> or step:up / step:down".into()),
            }
        }
        other => return Err(format!("Unknown command {other}")),
    })
}

/// The registry handler of the `operandi.multires.*` commands.
pub fn run(
    e: &mut Editor,
    name: &str,
    a: &Args,
) -> std::result::Result<Option<crate::Command>, String> {
    let item = item_arg(a)?;
    if name == "operandi.multires.query" {
        return e.multires_report(item).map(|_| None);
    }
    e.multires_change(level_change(name, a)?, item)?;
    Ok(None)
}

/// `operandi.multires.query`'s answer.
pub fn answer(e: &Editor, name: &str, a: &Args) -> Option<std::result::Result<String, String>> {
    (name == "operandi.multires.query").then(|| {
        let report = e.multires_report(item_arg(a)?)?;
        Ok(report.to_string())
    })
}

/// The level readout for the Sculpt tab: "Level 2 of 3 · 98,304 polygons".
pub fn readout(item: &crate::Item) -> Option<String> {
    let m = item.multires.as_deref()?;
    let polygons = m.polygon_counts()[m.current as usize];
    Some(format!(
        "Level {} of {} · {} polygons",
        m.current,
        m.top(),
        crate::polygon_subdivide::group_thousands(polygons as u64)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> Mesh {
        crate::primitive::cube_geometry(DVec3::ZERO, DVec3::ONE).unwrap()
    }

    #[test]
    fn counts_follow_the_refinement() {
        let base = cube();
        let m = Multires::new(&base, BoundaryRule::Crease);
        let mut net = base_net(&base).unwrap();
        let c = counts(&base, 3);
        for &count in &c[1..] {
            net = step(net, m.boundary_rule);
            assert_eq!(net.points.len(), count);
        }
    }

    #[test]
    fn detail_round_trips_through_frames() {
        let base = cube();
        let net = step(base_net(&base).unwrap(), BoundaryRule::Crease);
        let moved: Vec<DVec3> = net
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| *p + DVec3::new(0.01 * i as f64, -0.02, 0.03))
            .collect();
        let d = detail_of(&net.points, &net.faces, &moved);
        let back = with_detail(&net.points, &net.faces, &d);
        for (a, b) in back.iter().zip(&moved) {
            assert!(a.distance(*b) < 1e-12);
        }
    }
}
