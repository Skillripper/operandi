//! The MeshBlend evaluator (PLAN.md rule 2): a plain core function, recipe
//! and source meshes in, mesh and where each polygon came from out. The
//! MeshBlend op calls it; nothing else about it depends on the Mesh Stack.
//!
//! 1. Each enabled source becomes its smooth Catmull-Clark surface at its
//!    Subdivision level, or its cage at 0 or with Keep cage points (SB-20..23).
//! 2. Pre-flight (ST-06, SB-70): open, non-manifold, inside-out,
//!    self-crossing or empty sources are left out with one plain line;
//!    the rest still builds.
//! 3. Sources go in a canonical order (a hash of their geometry and role)
//!    before anything is numbered, so the result never depends on the
//!    order the items were made or listed in (ST-09).
//! 4. Per branch (SB-13): each Primary minus the Subtractive Trims related
//!    to it, intersected with its Intersect Trims (one that misses leaves
//!    it whole, SB-14); then the union of every branch. One exact Manifold
//!    run per step (ST-12: identical and coplanar inputs give exact
//!    answers), panics caught (ST-05).
//! 5. Polygons are rebuilt from Manifold's provenance: an untouched source
//!    polygon comes back as it was, a cut one as one piece per region,
//!    split into triangles when it has more than four corners (SB-51).
//!    Each keeps its source polygon's material and part (SB-52). If the
//!    rebuilt polygons break a mesh rule, the engine's triangles are used.
//!
//! B1 makes sharp joins; strips along them come in B2 (the strip builder
//! picked in B0).
use super::recipe::{Recipe, Role};
use crate::boolean::{Tri, guarded, manifold_solid, self_intersection};
use crate::cone::PolygonRepresentation;
use crate::subdivision::{SubdivisionSettings, subdivide};
use crate::{Face, Mesh, Result};
use glam::DVec3;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

/// One source mesh as the evaluator gets it: already in the MeshBlend
/// Item's space.
#[derive(Clone, Debug)]
pub struct Input {
    pub item: u64,
    pub name: String,
    pub mesh: Mesh,
}

/// A line about one source: a problem that left it out (red badge,
/// SB-70) or a hint (SB-14).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub item: u64,
    pub text: String,
    pub problem: bool,
}

/// Where a result polygon came from (SB-52).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Origin {
    pub item: u64,
    /// The polygon of that source's smooth surface.
    pub polygon: u32,
}

/// The evaluator's result.
#[derive(Clone, Debug, Default)]
pub struct Blend {
    pub mesh: Mesh,
    /// Per polygon of `mesh`.
    pub origin: Vec<Origin>,
    pub notes: Vec<Note>,
}

/// SB-20 / SB-21: the source's smooth surface at `level` (0 = the cage).
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
/// their shorter diagonal (ties: corner 0 to 2), n-gons by ear clipping;
/// curves are skipped.
pub(crate) fn triangles(mesh: &Mesh) -> Tri {
    let mut t = Tri {
        pos: mesh.vertices.clone(),
        ..Default::default()
    };
    for (f, face) in mesh.faces.iter().enumerate() {
        if mesh.is_polyline(f) {
            continue;
        }
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
            "{name} has edges shared by more than two polygons, or polygons facing different ways"
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

/// ST-09: a hash of a mesh's geometry and a role, for the canonical order.
fn content_hash(mesh: &Mesh, role: Role) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for p in &mesh.vertices {
        p.to_array().map(f64::to_bits).hash(&mut h);
    }
    for f in mesh.faces.iter() {
        f.vertices.hash(&mut h);
    }
    role.hash(&mut h);
    h.finish()
}

/// One source ready for the engine.
struct Prepared {
    item: u64,
    name: String,
    role: Role,
    trims: Vec<u64>,
    surface: Mesh,
    tri: Tri,
}

/// The MeshBlend result for `recipe` from `inputs` (see the module
/// comment). Errors only when the engine itself fails; bad sources become
/// notes.
pub fn evaluate(recipe: &Recipe, inputs: &[Input]) -> Result<Blend> {
    recipe.check()?;
    let mut notes = Vec::new();
    let mut prepared = Vec::new();
    for s in recipe.sources.iter().filter(|s| s.enabled) {
        let Some(input) = inputs.iter().find(|i| i.item == s.item) else {
            notes.push(Note {
                item: s.item,
                text: "This source is missing".into(),
                problem: true,
            });
            continue;
        };
        let level = if s.keep_cage { 0 } else { s.subdivision };
        let surface = match smooth(&input.mesh, level) {
            Ok(m) => m,
            Err(e) => {
                notes.push(Note {
                    item: s.item,
                    text: format!(
                        "{} couldn't be smoothed ({e}), so it is left out",
                        input.name
                    ),
                    problem: true,
                });
                continue;
            }
        };
        let tri = triangles(&surface);
        if let Err(e) = preflight(&input.name, &tri) {
            notes.push(Note {
                item: s.item,
                text: format!("{e}, so it is left out"),
                problem: true,
            });
            continue;
        }
        prepared.push(Prepared {
            item: s.item,
            name: input.name.clone(),
            role: s.role,
            trims: s.trims.clone(),
            surface,
            tri,
        });
    }
    // ST-09: canonical order.
    prepared.sort_by_cached_key(|p| (content_hash(&p.surface, p.role), p.item));

    // Run ids: one per polygon of every source, numbered across sources.
    let mut offsets = vec![0u32];
    for p in &prepared {
        offsets.push(offsets.last().unwrap() + p.surface.faces.len() as u32);
    }
    let all: Vec<DVec3> = prepared
        .iter()
        .flat_map(|p| p.tri.pos.iter().copied())
        .collect();
    let centre = if all.is_empty() {
        DVec3::ZERO
    } else {
        let (lo, hi) = crate::boolean::bbox(&all);
        (lo + hi) * 0.5
    };
    let solid = |c: usize| {
        let mut t = prepared[c].tri.clone();
        for p in &mut t.pos {
            *p -= centre;
        }
        for o in &mut t.owner {
            *o += offsets[c];
        }
        manifold_solid(&t)
    };
    let engine = guarded(|| {
        let mut branches = Vec::new();
        for (c, p) in prepared.iter().enumerate() {
            if p.role != Role::Primary {
                continue;
            }
            let mut b = solid(c)?;
            for (tc, t) in prepared.iter().enumerate() {
                if !t.role.is_trim() || !t.trims.contains(&p.item) {
                    continue;
                }
                let cutter = solid(tc)?;
                match t.role {
                    Role::Subtract => b = b.difference(&cutter),
                    Role::Intersect => {
                        let cut = b.intersection(&cutter);
                        if cut.to_meshgl64().tri_verts().is_empty() {
                            notes.push(Note {
                                item: t.item,
                                text: format!("{} doesn't reach {}", t.name, p.name),
                                problem: false,
                            });
                        } else {
                            b = cut;
                        }
                    }
                    Role::Primary => {}
                }
            }
            branches.push(b);
        }
        let mut out = Tri::default();
        if branches.is_empty() {
            return Ok(out);
        }
        let all = manifold_csg::Manifold::batch_union(&branches);
        all.status()
            .map_err(|e| format!("the engine failed: {e}"))?;
        let gl = all.to_meshgl64();
        let props = gl.num_prop();
        out.pos = gl
            .vert_properties()
            .chunks(props)
            .map(|c| DVec3::new(c[0], c[1], c[2]) + centre)
            .collect();
        let verts = gl.tri_verts();
        let starts = gl.run_index();
        for (r, &id) in gl.run_original_id().iter().enumerate() {
            let id = id
                .checked_sub(1 << 30)
                .ok_or("the engine lost track of where a result polygon came from")?;
            for k in (starts[r] as usize..starts[r + 1] as usize).step_by(3) {
                out.tris
                    .push([verts[k] as u32, verts[k + 1] as u32, verts[k + 2] as u32]);
                out.owner.push(id);
            }
        }
        Ok(out)
    })
    .map_err(|e| format!("MeshBlend: {e}"))?;
    // Trims related to nothing in the result (SB-14 hint).
    for t in prepared.iter().filter(|t| t.role.is_trim()) {
        if !t.trims.iter().any(|p| {
            prepared
                .iter()
                .any(|q| q.item == *p && q.role == Role::Primary)
        }) {
            notes.push(Note {
                item: t.item,
                text: format!("{} isn't related to any Primary yet", t.name),
                problem: false,
            });
        }
    }
    // Polygons rebuilt from provenance; if a region came back in a shape
    // the mesh rules refuse, fall back to the engine's triangles, which
    // are always closed and consistently wound (ST-10 before swap-in).
    let (mesh, origin) = match rebuild(&prepared, &offsets, &engine, false) {
        Ok(built) if built.0.validate().is_ok() => built,
        _ => rebuild(&prepared, &offsets, &engine, true)?,
    };
    Ok(Blend {
        mesh,
        origin,
        notes,
    })
}

/// Step 5: polygons from provenance.
fn rebuild(
    prepared: &[Prepared],
    offsets: &[u32],
    out: &Tri,
    triangles_only: bool,
) -> Result<(Mesh, Vec<Origin>)> {
    let mut groups: BTreeMap<u32, Vec<[u32; 3]>> = BTreeMap::new();
    for (t, tri) in out.tris.iter().enumerate() {
        groups.entry(out.owner[t]).or_default().push(*tri);
    }
    let mut mesh = Mesh {
        vertices: out.pos.clone(),
        ..Default::default()
    };
    let mut origin = Vec::new();
    for (g, tris) in groups {
        let s = offsets.partition_point(|&o| o <= g) - 1;
        let polygon = g - offsets[s];
        let template: &Face = &prepared[s].surface.faces[polygon as usize];
        let rings = if triangles_only {
            tris.iter().map(|t| t.to_vec()).collect()
        } else {
            crate::boolean::regions(&tris)
        };
        for ring in rings {
            let mut v = ring;
            v.dedup();
            while v.len() > 1 && v.first() == v.last() {
                v.pop();
            }
            if v.len() < 3 {
                continue;
            }
            let pieces = if v.len() <= 4 {
                vec![v]
            } else {
                let pts: Vec<DVec3> = v.iter().map(|&i| mesh.vertices[i as usize]).collect();
                let n = v.len();
                crate::polygon_triple::triangulate_polygon(&pts, |_, _| false)
                    .unwrap_or_else(|_| (1..n - 1).map(|i| [0, i, i + 1]).collect())
                    .into_iter()
                    .map(|t| t.iter().map(|&i| v[i]).collect())
                    .collect()
            };
            for vertices in pieces {
                let mut f = template.clone();
                f.vertices = vertices;
                f.uv = None;
                mesh.faces.push(f);
                origin.push(Origin {
                    item: prepared[s].item,
                    polygon,
                });
            }
        }
    }
    let mesh = crate::mesh_cleanup::remove_unreferenced(&mesh, None)?.mesh;
    Ok((mesh, origin))
}
