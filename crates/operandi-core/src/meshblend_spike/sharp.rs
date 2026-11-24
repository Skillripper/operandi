//! Smooth sources and the sharp boolean both strip prototypes start from.
//!
//! * SB-20: each source becomes its Catmull-Clark surface at the scene's
//!   level (0 = the cage, n-gons split, SB-21).
//! * ST-06 / SB-70: each source is checked first (closed, edge-manifold,
//!   facing out, finite, not passing through itself); a failing source is
//!   left out with a reason and the rest still builds.
//! * ST-09: sources are put in a canonical order (by a hash of their
//!   geometry and role) before anything is numbered, so the result never
//!   depends on the order the items were made in.
//! * SB-13: per branch (a Primary and the Trims related to it): subtract
//!   its Subtractive Trims, intersect with its Intersect Trims, then union
//!   every branch. SB-14: an Intersect Trim that misses leaves the Primary
//!   whole.
//!
//! The result is a closed triangle mesh in which every triangle names its
//! source and the source polygon it came from (Manifold's run ids).
use crate::boolean::{Tri, guarded, manifold_solid, self_intersection};
use crate::cone::PolygonRepresentation;
use crate::subdivision::{SubdivisionSettings, subdivide};
use crate::{Mesh, Result};
use glam::DVec3;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Instant;

/// A source's role (SB-10). Trims name the Primaries they're related to
/// (SB-11), by index in the scene's source list.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Primary,
    Subtract(Vec<usize>),
    Intersect(Vec<usize>),
}

/// One source mesh, in world space.
#[derive(Clone, Debug)]
pub struct Source {
    pub name: String,
    pub mesh: Mesh,
    pub role: Role,
}

/// The sharp result.
#[derive(Clone, Debug, Default)]
pub struct Sharp {
    pub pos: Vec<DVec3>,
    pub tris: Vec<[u32; 3]>,
    /// Per triangle: the canonical source.
    pub source: Vec<u32>,
    /// Per triangle: its polygon in that source's smooth surface.
    pub face: Vec<u32>,
    /// Smooth surface per canonical source.
    pub surfaces: Vec<Mesh>,
    /// Name per canonical source.
    pub names: Vec<String>,
    /// Canonical index → index in the caller's list.
    pub order: Vec<usize>,
    /// Sources left out, with the reason (SB-70), and other notes (SB-14).
    pub notes: Vec<String>,
    pub smooth_secs: f64,
    pub boolean_secs: f64,
}

/// SB-20 / SB-21: the source's smooth surface.
pub fn smooth(mesh: &Mesh, level: u8) -> Result<Mesh> {
    if level == 0 {
        return Ok(mesh.clone());
    }
    let mut m = mesh.clone();
    m.attributes.representations = vec![PolygonRepresentation::CatmullClark; m.faces.len()];
    let settings = SubdivisionSettings {
        subdivision_level: level,
        catmull_clark_level: level,
        ..Default::default()
    };
    let mut out = subdivide(&m, &settings)?;
    out.attributes.representations.clear();
    Ok(out)
}

/// Triangles of `mesh`, each tagged with its polygon. Quads split along
/// their shorter diagonal (ties: corner 0 to 2), n-gons by ear clipping.
pub(crate) fn triangles(mesh: &Mesh) -> Tri {
    let mut t = Tri {
        pos: mesh.vertices.clone(),
        ..Default::default()
    };
    for (f, face) in mesh.faces.iter().enumerate() {
        let v = &face.vertices;
        let p = |i: usize| mesh.vertices[v[i] as usize];
        let mut add = |a: usize, b: usize, c: usize| {
            t.tris.push([v[a], v[b], v[c]]);
            t.owner.push(f as u32);
        };
        match v.len() {
            0..=2 => {}
            3 => add(0, 1, 2),
            4 => {
                if p(1).distance_squared(p(3)) < p(0).distance_squared(p(2)) {
                    add(0, 1, 3);
                    add(1, 2, 3);
                } else {
                    add(0, 1, 2);
                    add(0, 2, 3);
                }
            }
            _ => {
                let pts: Vec<DVec3> = (0..v.len()).map(p).collect();
                let fan: Vec<[usize; 3]> = (1..v.len() - 1).map(|i| [0, i, i + 1]).collect();
                let tris =
                    crate::polygon_triple::triangulate_polygon(&pts, |_, _| false).unwrap_or(fan);
                for [a, b, c] in tris {
                    add(a, b, c);
                }
            }
        }
    }
    t
}

/// ST-06: why this surface can't be a source, if it can't.
pub(crate) fn preflight(name: &str, t: &Tri) -> std::result::Result<(), String> {
    if t.tris.is_empty() {
        return Err(format!("{name} has no polygons"));
    }
    if t.pos.iter().any(|p| !p.is_finite()) {
        return Err(format!("{name} has points at impossible positions"));
    }
    let mut uses: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in &t.tris {
        for k in 0..3 {
            *uses.entry((tri[k], tri[(k + 1) % 3])).or_default() += 1;
        }
    }
    let mut open = 0;
    let mut crowded = 0;
    for (&(a, b), &n) in &uses {
        let back = uses.get(&(b, a)).copied().unwrap_or(0);
        if n > 1 || back > 1 {
            crowded += 1;
        } else if back == 0 {
            open += 1;
        }
    }
    if crowded > 0 {
        return Err(format!(
            "{name} has edges shared by more than two polygons (or polygons facing different ways)"
        ));
    }
    if open > 0 {
        return Err(format!(
            "{name} has {}",
            crate::boolean::plural(open, "open edge", "open edges")
        ));
    }
    if t.volume() <= 0. {
        return Err(format!("{name} is inside out or has no volume"));
    }
    if self_intersection(t).is_some() {
        return Err(format!("{name} passes through itself"));
    }
    Ok(())
}

/// ST-09: a hash of a source's geometry and role kind, used to put sources
/// in an order that doesn't depend on how the scene was made.
fn content_hash(s: &Source) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for p in &s.mesh.vertices {
        p.to_array().map(f64::to_bits).hash(&mut h);
    }
    for f in s.mesh.faces.iter() {
        f.vertices.hash(&mut h);
    }
    std::mem::discriminant(&s.role).hash(&mut h);
    h.finish()
}

/// Smooth, check and combine the sources (see the module comment).
pub fn build(sources: &[Source], level: u8) -> Result<Sharp> {
    let mut order: Vec<usize> = (0..sources.len()).collect();
    let hashes: Vec<u64> = sources.iter().map(content_hash).collect();
    order.sort_by_key(|&i| hashes[i]);
    let canonical: HashMap<usize, usize> = order.iter().enumerate().map(|(c, &i)| (i, c)).collect();

    let started = Instant::now();
    let mut surfaces = Vec::new();
    let mut tris = Vec::new();
    for &i in &order {
        let s = smooth(&sources[i].mesh, level)?;
        tris.push(triangles(&s));
        surfaces.push(s);
    }
    let smooth_secs = started.elapsed().as_secs_f64();

    let started = Instant::now();
    let names: Vec<String> = order.iter().map(|&i| sources[i].name.clone()).collect();
    let mut notes = Vec::new();
    let ok: Vec<bool> = tris
        .iter()
        .zip(&names)
        .map(|(t, n)| match preflight(n, t) {
            Ok(()) => true,
            Err(e) => {
                notes.push(format!("{e}, so it is left out"));
                false
            }
        })
        .collect();
    // Run ids: one per polygon of every source, numbered across sources.
    let mut offsets = vec![0u32];
    for t in &tris {
        offsets.push(offsets.last().unwrap() + t.owner.iter().max().map_or(0, |m| m + 1));
    }
    // Recentre (precision; as the destructive Boolean, step 3).
    let all: Vec<DVec3> = tris.iter().flat_map(|t| t.pos.iter().copied()).collect();
    let centre = if all.is_empty() {
        DVec3::ZERO
    } else {
        let (lo, hi) = crate::boolean::bbox(&all);
        (lo + hi) * 0.5
    };
    let engine_input = |c: usize| {
        let mut t = tris[c].clone();
        for p in &mut t.pos {
            *p -= centre;
        }
        for o in &mut t.owner {
            *o += offsets[c];
        }
        t
    };
    let solid = |c: usize| manifold_solid(&engine_input(c));
    let mut branches = Vec::new();
    for (c, &i) in order.iter().enumerate() {
        if sources[i].role != Role::Primary || !ok[c] {
            continue;
        }
        let mut b = solid(c)?;
        for (tc, &ti) in order.iter().enumerate() {
            if !ok[tc] {
                continue;
            }
            match &sources[ti].role {
                Role::Subtract(of) if of.iter().any(|p| canonical.get(p) == Some(&c)) => {
                    b = b.difference(&solid(tc)?);
                }
                Role::Intersect(of) if of.iter().any(|p| canonical.get(p) == Some(&c)) => {
                    let cut = b.intersection(&solid(tc)?);
                    if cut.to_meshgl64().tri_verts().is_empty() {
                        notes.push(format!("{} doesn't reach {}", names[tc], names[c]));
                    } else {
                        b = cut;
                    }
                }
                _ => {}
            }
        }
        branches.push(b);
    }
    let mut sharp = Sharp {
        surfaces,
        names,
        order: order.clone(),
        notes,
        smooth_secs,
        ..Default::default()
    };
    if branches.is_empty() {
        sharp.boolean_secs = started.elapsed().as_secs_f64();
        return Ok(sharp);
    }
    let out = guarded(|| {
        let all = manifold_csg::Manifold::batch_union(&branches);
        all.status()
            .map_err(|e| format!("the engine failed: {e}"))?;
        let gl = all.to_meshgl64();
        let props = gl.num_prop();
        let mut t = Tri {
            pos: gl
                .vert_properties()
                .chunks(props)
                .map(|c| DVec3::new(c[0], c[1], c[2]) + centre)
                .collect(),
            ..Default::default()
        };
        let verts = gl.tri_verts();
        let starts = gl.run_index();
        for (r, &id) in gl.run_original_id().iter().enumerate() {
            let id = id.checked_sub(1 << 30).ok_or("lost provenance")?;
            for k in (starts[r] as usize..starts[r + 1] as usize).step_by(3) {
                t.tris
                    .push([verts[k] as u32, verts[k + 1] as u32, verts[k + 2] as u32]);
                t.owner.push(id);
            }
        }
        Ok(t)
    })?;
    sharp.boolean_secs = started.elapsed().as_secs_f64();
    for &g in &out.owner {
        let s = offsets.partition_point(|&o| o <= g) - 1;
        sharp.source.push(s as u32);
        sharp.face.push(g - offsets[s]);
    }
    sharp.pos = out.pos;
    sharp.tris = out.tris;
    if sharp.tris.is_empty() {
        sharp.notes.push("the result is empty".into());
    }
    Ok(sharp)
}

impl Sharp {
    /// The sharp result as polygons: triangles only, unwelded nothing.
    pub fn mesh(&self) -> Mesh {
        Mesh {
            vertices: self.pos.clone(),
            faces: self
                .tris
                .iter()
                .map(|t| super::poly(t.to_vec()))
                .collect::<Vec<_>>()
                .into(),
            ..Default::default()
        }
    }
}
