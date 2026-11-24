//! The ST-10 topology gate and the ST-09 repeatability fingerprint.
//!
//! ST-10, checked on every result before it is shown: closed, every edge
//! shared by exactly two polygons that run it opposite ways, no point
//! duplicated, no edge shorter than max(width / 1000, 1e-6 × size), only
//! quads and triangles, finite. B0 also counts what the comparison page
//! shows (polygons by sides, shells, volume, strip polygons that aren't
//! quads).
use crate::Mesh;
use glam::DVec3;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub polygons: usize,
    pub triangles: usize,
    pub quads: usize,
    /// Polygons with five or more sides.
    pub ngons: usize,
    /// Edges with one polygon.
    pub open_edges: usize,
    /// Edges with three or more polygons, or two running the same way.
    pub bad_edges: usize,
    /// Pairs of distinct points at the same place.
    pub duplicate_points: usize,
    /// Edges shorter than the ST-10 minimum.
    pub short_edges: usize,
    pub nonfinite: usize,
    /// Strip or corner polygons that aren't quads (SB-30, SB-34).
    pub strip_non_quads: usize,
    pub shells: usize,
    pub volume: f64,
}

impl Report {
    /// Passes ST-10.
    pub fn passes(&self) -> bool {
        self.open_edges == 0
            && self.bad_edges == 0
            && self.duplicate_points == 0
            && self.short_edges == 0
            && self.ngons == 0
            && self.nonfinite == 0
    }

    /// What fails, in plain words (empty when it passes).
    pub fn failures(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut say = |n: usize, what: &str| {
            if n > 0 {
                out.push(format!("{n} {what}"));
            }
        };
        say(self.open_edges, "open edges");
        say(self.bad_edges, "edges with 3+ polygons or flipped");
        say(self.duplicate_points, "duplicate points");
        say(self.short_edges, "tiny edges");
        say(self.ngons, "n-gons");
        say(self.nonfinite, "bad points");
        out
    }
}

/// Run the ST-10 checks. `kind` marks strip / corner polygons (non-zero).
pub fn st10(mesh: &Mesh, width: f64, kind: Option<&[u8]>) -> Report {
    let mut r = Report {
        polygons: mesh.faces.len(),
        ..Default::default()
    };
    let used: Vec<u32> = {
        let mut u = vec![false; mesh.vertices.len()];
        for f in mesh.faces.iter() {
            for &v in &f.vertices {
                u[v as usize] = true;
            }
        }
        (0..u.len() as u32).filter(|&v| u[v as usize]).collect()
    };
    r.nonfinite = used
        .iter()
        .filter(|&&v| !mesh.vertices[v as usize].is_finite())
        .count();
    let (lo, hi) = used.iter().fold(
        (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY)),
        |(lo, hi), &v| {
            let p = mesh.vertices[v as usize];
            (lo.min(p), hi.max(p))
        },
    );
    let size = if used.is_empty() {
        1.
    } else {
        (hi - lo).length().max(1e-12)
    };
    let min_edge = (width / 1000.).max(1e-6 * size);
    let mut directed: HashMap<(u32, u32), u32> = HashMap::new();
    for (f, face) in mesh.faces.iter().enumerate() {
        let v = &face.vertices;
        match v.len() {
            3 => r.triangles += 1,
            4 => r.quads += 1,
            _ => r.ngons += 1,
        }
        if kind.is_some_and(|k| k[f] != 0) && v.len() != 4 {
            r.strip_non_quads += 1;
        }
        for i in 0..v.len() {
            let (a, b) = (v[i], v[(i + 1) % v.len()]);
            *directed.entry((a, b)).or_default() += 1;
        }
    }
    let undirected: std::collections::HashSet<(u32, u32)> = directed
        .keys()
        .map(|&(a, b)| crate::topology::edge_key(a, b))
        .collect();
    r.short_edges = undirected
        .iter()
        .filter(|&&(a, b)| mesh.vertices[a as usize].distance(mesh.vertices[b as usize]) < min_edge)
        .count();
    for (&(a, b), &n) in &directed {
        let back = directed.get(&(b, a)).copied().unwrap_or(0);
        if a < b || back == 0 {
            match (n, back) {
                (1, 1) => {}
                (1, 0) => r.open_edges += 1,
                _ => r.bad_edges += 1,
            }
        }
    }
    // Duplicate points: a grid at 1e-9 of the size.
    let cell = 1e-9 * size;
    let mut grid: HashMap<[i64; 3], Vec<u32>> = HashMap::new();
    for &v in &used {
        let p = mesh.vertices[v as usize];
        if !p.is_finite() {
            continue;
        }
        let c = (p / cell).floor().as_i64vec3().to_array();
        let mut dup = false;
        'near: for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(list) = grid.get(&[c[0] + dx, c[1] + dy, c[2] + dz]) {
                        for &w in list {
                            if mesh.vertices[w as usize].distance(p) <= cell {
                                dup = true;
                                break 'near;
                            }
                        }
                    }
                }
            }
        }
        if dup {
            r.duplicate_points += 1;
        }
        grid.entry(c).or_default().push(v);
    }
    r.shells = shells(mesh);
    r.volume = volume(mesh);
    r
}

/// Connected pieces (polygons joined through shared edges).
pub fn shells(mesh: &Mesh) -> usize {
    let n = mesh.faces.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    let mut first: HashMap<(u32, u32), usize> = HashMap::new();
    for (f, face) in mesh.faces.iter().enumerate() {
        let v = &face.vertices;
        for i in 0..v.len() {
            let e = crate::topology::edge_key(v[i], v[(i + 1) % v.len()]);
            if let Some(&g) = first.get(&e) {
                let (a, b) = (root(&mut parent, f), root(&mut parent, g));
                parent[a] = b;
            } else {
                first.insert(e, f);
            }
        }
    }
    (0..n).filter(|&f| root(&mut parent, f) == f).count()
}

/// Enclosed volume (fan per polygon about the bounding box centre).
pub fn volume(mesh: &Mesh) -> f64 {
    let c =
        mesh.vertices.iter().fold(DVec3::ZERO, |s, p| s + *p) / mesh.vertices.len().max(1) as f64;
    let mut v = 0.;
    for f in mesh.faces.iter() {
        let p: Vec<DVec3> = f
            .vertices
            .iter()
            .map(|&i| mesh.vertices[i as usize] - c)
            .collect();
        for i in 1..p.len().saturating_sub(1) {
            v += p[0].dot(p[i].cross(p[i + 1]));
        }
    }
    v / 6.
}

/// ST-09: a fingerprint of every point (bit for bit) and polygon.
pub fn fingerprint(mesh: &Mesh) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for p in &mesh.vertices {
        p.to_array().map(f64::to_bits).hash(&mut h);
    }
    for f in mesh.faces.iter() {
        f.vertices.hash(&mut h);
    }
    h.finish()
}
