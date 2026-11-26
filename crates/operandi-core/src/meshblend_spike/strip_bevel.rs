//! Prototype B (PLAN.md "Strip method"): bevel the sharp junction.
//!
//! 1. Rebuild polygons from the sharp result's provenance: the triangles
//!    of one source polygon become that polygon again (untouched quads
//!    come back as quads, cut ones as one n-gon per piece), as the
//!    destructive Boolean does (BOOL-09).
//! 2. The join edges are the edges between polygons of different sources.
//! 3. Tidy: join edges shorter than 1.5 × width are collapsed (the bevel
//!    can't offset a corner further than its neighbouring join edges are
//!    long), then points near the join slide away from it so the polygons
//!    beside it are wider than the strip (`make_room`).
//! 4. Our Edge Chamfer (parallel offset, so Width is the distance from the
//!    old crease to each band edge, Q05) with 2 × Rows segments and Depth =
//!    Profile (0–100 %; Edge Bevel has no over-round).
//! 5. Leftover n-gons are split into triangles (ST-10: quads and
//!    triangles only).
use super::sharp::Sharp;
use super::{Blended, poly};
use crate::edge_bevel::{EdgeBevelOptions, EdgeBevelSettings, edge_bevel_with};
use crate::topology::edge_key;
use crate::{Mesh, Result};
use glam::DVec3;
use std::collections::{BTreeMap, HashMap};

const STRIP: &str = "MeshBlend Strip";

/// Step 1: polygons, each with its canonical source in its material name.
pub fn polygons(sharp: &Sharp) -> Mesh {
    let mut groups: BTreeMap<(u32, u32), Vec<[u32; 3]>> = BTreeMap::new();
    for (t, tri) in sharp.tris.iter().enumerate() {
        groups
            .entry((sharp.source[t], sharp.face[t]))
            .or_default()
            .push(*tri);
    }
    let mut mesh = Mesh {
        vertices: sharp.pos.clone(),
        ..Default::default()
    };
    for ((s, _), tris) in groups {
        for ring in crate::boolean::regions(&tris) {
            let mut v = ring;
            v.dedup();
            if v.len() >= 3 {
                let mut f = poly(v);
                f.material = format!("source {s}");
                mesh.faces.push(f);
            }
        }
    }
    crate::mesh_cleanup::remove_unreferenced(&mesh, None)
        .map(|r| r.mesh)
        .unwrap_or(mesh)
}

fn source_of(material: &str) -> Option<u32> {
    material.strip_prefix("source ")?.parse().ok()
}

/// Step 2: edges whose two polygons come from different sources.
pub fn join_edges(mesh: &Mesh) -> Vec<(u32, u32)> {
    let mut by_edge: HashMap<(u32, u32), Vec<Option<u32>>> = HashMap::new();
    for f in mesh.faces.iter() {
        let v = &f.vertices;
        for i in 0..v.len() {
            by_edge
                .entry(edge_key(v[i], v[(i + 1) % v.len()]))
                .or_default()
                .push(source_of(&f.material));
        }
    }
    let mut out: Vec<_> = by_edge
        .into_iter()
        .filter(|(_, s)| s.len() == 2 && s[0] != s[1])
        .map(|(e, _)| e)
        .collect();
    out.sort();
    out
}

/// Step 3: collapse join edges shorter than `min` into their midpoints.
/// Polygons that drop below three corners go.
fn tidy(mesh: &Mesh, min: f64) -> Mesh {
    let mut parent: Vec<u32> = (0..mesh.vertices.len() as u32).collect();
    fn root(p: &mut [u32], mut i: u32) -> u32 {
        while p[i as usize] != i {
            i = p[i as usize];
        }
        i
    }
    let mut pos = mesh.vertices.clone();
    for (a, b) in join_edges(mesh) {
        let (ra, rb) = (root(&mut parent, a), root(&mut parent, b));
        if ra != rb && pos[ra as usize].distance(pos[rb as usize]) < min {
            let mid = (pos[ra as usize] + pos[rb as usize]) * 0.5;
            let (keep, gone) = (ra.min(rb), ra.max(rb));
            parent[gone as usize] = keep;
            pos[keep as usize] = mid;
        }
    }
    let mut out = Mesh {
        vertices: pos,
        ..Default::default()
    };
    for f in mesh.faces.iter() {
        let mut v: Vec<u32> = f.vertices.iter().map(|&i| root(&mut parent, i)).collect();
        v.dedup();
        while v.len() > 1 && v.first() == v.last() {
            v.pop();
        }
        if v.len() >= 3 {
            let mut g = f.clone();
            g.vertices = v;
            out.faces.push(g);
        }
    }
    crate::mesh_cleanup::remove_unreferenced(&out, None)
        .map(|r| r.mesh)
        .unwrap_or(out)
}

/// Step 5: n-gons into triangles (ear clipping; a fan if that fails).
fn split_ngons(mesh: &mut Mesh) {
    let mut faces = Vec::with_capacity(mesh.faces.len());
    for f in mesh.faces.iter() {
        if f.vertices.len() <= 4 {
            faces.push(f.clone());
            continue;
        }
        let pts: Vec<DVec3> = f
            .vertices
            .iter()
            .map(|&v| mesh.vertices[v as usize])
            .collect();
        let n = pts.len();
        let tris = crate::polygon_triple::triangulate_polygon(&pts, |_, _| false)
            .unwrap_or_else(|_| (1..n - 1).map(|i| [0, i, i + 1]).collect());
        for t in tris {
            let mut g = f.clone();
            g.vertices = t.iter().map(|&i| f.vertices[i]).collect();
            faces.push(g);
        }
    }
    mesh.faces = faces.into();
}

/// Step 3, second half: give the bevel room. Points closer to a join than
/// 2.5 × width slide away from it along their surface so that distances
/// 0..2.5w become 1.25w..2.5w: the polygons beside the join are then
/// wider than the strip, and no two rows land on each other.
fn make_room(mesh: &mut Mesh, joins: &[(u32, u32)], width: f64) {
    let reach = 2.5 * width;
    let floor = 1.25 * width;
    let mut on_join = vec![false; mesh.vertices.len()];
    for &(a, b) in joins {
        on_join[a as usize] = true;
        on_join[b as usize] = true;
    }
    // Surface normal per point (sum of its polygons' normals).
    let mut normal = vec![DVec3::ZERO; mesh.vertices.len()];
    for f in mesh.faces.iter() {
        let v = &f.vertices;
        let mut n = DVec3::ZERO;
        for i in 0..v.len() {
            n += mesh.vertices[v[i] as usize].cross(mesh.vertices[v[(i + 1) % v.len()] as usize]);
        }
        for &i in v {
            normal[i as usize] += n;
        }
    }
    let segs: Vec<(DVec3, DVec3)> = joins
        .iter()
        .map(|&(a, b)| (mesh.vertices[a as usize], mesh.vertices[b as usize]))
        .collect();
    let (lo, hi) = segs.iter().fold(
        (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY)),
        |(lo, hi), (a, b)| (lo.min(a.min(*b)), hi.max(a.max(*b))),
    );
    let old = mesh.vertices.clone();
    for (v, p) in mesh.vertices.iter_mut().enumerate() {
        if on_join[v] || p.cmplt(lo - reach).any() || p.cmpgt(hi + reach).any() {
            continue;
        }
        let q = old[v];
        let mut best = (f64::INFINITY, q);
        for &(a, b) in &segs {
            let ab = b - a;
            let t = ((q - a).dot(ab) / ab.length_squared().max(1e-300)).clamp(0., 1.);
            let f = a + ab * t;
            let d = f.distance(q);
            if d < best.0 {
                best = (d, f);
            }
        }
        let (d, foot) = best;
        if d >= reach || d <= 0. {
            continue;
        }
        let n = normal[v].normalize_or_zero();
        let away = q - foot;
        let dir = (away - n * away.dot(n)).normalize_or_zero();
        let target = floor + (reach - floor) * d / reach;
        *p = q + dir * (target - d);
    }
}

/// Prototype B on a sharp result.
pub fn blend(sharp: &Sharp, width: f64, profile: f64, rows: u32) -> Result<Blended> {
    let mut problems = Vec::new();
    let mut base = tidy(&polygons(sharp), width * 1.5);
    let joins = join_edges(&base);
    make_room(&mut base, &joins, width);
    let mut mesh = if joins.is_empty() {
        base
    } else {
        if profile > 100. {
            problems.push("Edge Bevel has no over-round, so the profile stops at 100%".into());
        }
        let settings = EdgeBevelSettings {
            width,
            rows: (2 * rows).clamp(1, 64),
            depth: profile.clamp(0., 100.),
            chamfer: true,
            options: EdgeBevelOptions {
                use_material: true,
                material: STRIP.into(),
                auto_weld: true,
                ..Default::default()
            },
        };
        match edge_bevel_with(&base, &joins, &settings, None) {
            Ok(r) => r.mesh,
            Err(e) => {
                problems.push(format!("Edge Bevel refused the joins: {e}"));
                base
            }
        }
    };
    split_ngons(&mut mesh);
    let kind = mesh
        .faces
        .iter()
        .map(|f| u8::from(f.material == STRIP))
        .collect();
    let source = mesh
        .faces
        .iter()
        .map(|f| source_of(&f.material).unwrap_or(0))
        .collect();
    Ok(Blended {
        mesh,
        kind,
        source,
        problems,
    })
}
