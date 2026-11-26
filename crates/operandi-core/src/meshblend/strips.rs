//! MeshBlend strips (B2; the strip builder Jason picked in B0, prototype D,
//! 2026-11-24): our own strip builder, on the sharp result of the smooth
//! sources.
//!
//! 1. **Joins.** Edges between triangles of different sources, chained into
//!    lines that run between corners (points where three or more sources
//!    meet) or close on themselves.
//! 2. **Distance.** For every point of each surface, the distance to the
//!    nearest join that surface borders, spread out from the joins along
//!    the surface (so a join never reaches through a thin wall).
//! 3. **Cut.** Each surface loses the part closer than Width to a join: the
//!    triangles are clipped exactly at distance = Width, so each join gets
//!    one cut line on each side. Width is the distance from the old crease
//!    to each band edge (Q05).
//! 4. **Stations.** Every cut-line point is matched with a point at the
//!    same place along the join on the other side; where the other side
//!    has no point there, one is added by splitting its cut edge (no
//!    T-junctions, so the weld is exact by construction, SB-50).
//! 5. **Loft.** Between each pair of matched points the profile curve runs
//!    tangent to both surfaces: a rational quadratic through the two
//!    tangent lines' meeting point, a true circular arc at 100 % (Q06),
//!    straight at 0 %, past the corner above 100 %. Rows N gives 2N quads
//!    across (SB-33), evenly spaced along the curve.
//! 6. **Corners.** Where three strips end, the hole between their three
//!    end profiles is filled with three quad patches around one centre
//!    point, which has three edges (SB-34's Y junction).
//!
//! Spike limits (would be B2 work): width isn't limited to what fits
//! (ST-11), distance is straight-line to the join (not measured along a
//! curved surface), and corner patches are flat-ish Coons patches.
use crate::Mesh;
use crate::meshblend_spike::poly;
use crate::topology::edge_key;
use glam::DVec3;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};

/// The sharp result the strips are built on: a closed triangle mesh in
/// which every triangle names its source and source polygon.
#[derive(Clone, Debug, Default)]
pub struct Joined {
    pub pos: Vec<DVec3>,
    pub tris: Vec<[u32; 3]>,
    /// Per triangle: the source (an index into `names`).
    pub source: Vec<u32>,
    /// Per triangle: its polygon in that source's surface.
    pub face: Vec<u32>,
    pub names: Vec<String>,
    /// Per source: its item id (names stable strip ids, ST-07).
    pub items: Vec<u64>,
}

/// One strip's settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StripParams {
    /// Distance from the old crease to each band edge.
    pub width: f64,
    /// Profile in percent (0 chamfer, 100 round).
    pub profile: f64,
}

/// A strip the builder laid, for the panel's list and per-strip edits.
#[derive(Clone, Debug, PartialEq)]
pub struct StripInfo {
    /// Stable id (ST-07): the two source items, lower first, and the join's
    /// place among that pair's joins, ordered by lowest point (Y, then X,
    /// then Z) and then length: `"<a>-<b>-<n>"`.
    pub id: String,
    pub items: (u64, u64),
    /// Length of the join line.
    pub length: f64,
}

/// What the builder made.
#[derive(Clone, Debug, Default)]
pub struct Strips {
    pub mesh: Mesh,
    /// Per polygon: 0 = surface, 1 = strip, 2 = corner patch.
    pub kind: Vec<u8>,
    /// Per polygon: the source it lies on (a strip: its first side's).
    pub source: Vec<u32>,
    /// Per polygon: the source polygon a surface piece came from;
    /// `u32::MAX` for strips and corners.
    pub face: Vec<u32>,
    /// Strips or corners that couldn't be built, in plain words.
    pub problems: Vec<String>,
    /// Every join's strip, in join order.
    pub strips: Vec<StripInfo>,
}

/// A join line between two sources.
struct Chain {
    verts: Vec<u32>,
    closed: bool,
    /// Source of the triangles to the left (running verts forwards), right.
    sides: (u32, u32),
    /// Arc length at each vertex.
    cum: Vec<f64>,
}

impl Chain {
    fn segments(&self) -> usize {
        if self.closed {
            self.verts.len()
        } else {
            self.verts.len() - 1
        }
    }
    fn length(&self) -> f64 {
        *self.cum.last().unwrap()
    }
    fn ends(&self, pos: &[DVec3], i: usize) -> (DVec3, DVec3) {
        let n = self.verts.len();
        (
            pos[self.verts[i] as usize],
            pos[self.verts[(i + 1) % n] as usize],
        )
    }
    /// Nearest point on the chain: (distance, arc length, point).
    fn foot(&self, pos: &[DVec3], p: DVec3) -> (f64, f64, DVec3) {
        let mut best = (f64::INFINITY, 0., p);
        for i in 0..self.segments() {
            let (a, b) = self.ends(pos, i);
            let (t, q) = closest_on_segment(p, a, b);
            let d = q.distance(p);
            if d < best.0 {
                best = (d, self.cum[i] + t * a.distance(b), q);
            }
        }
        best
    }
}

fn closest_on_segment(p: DVec3, a: DVec3, b: DVec3) -> (f64, DVec3) {
    let ab = b - a;
    let l2 = ab.length_squared();
    let t = if l2 > 0. {
        ((p - a).dot(ab) / l2).clamp(0., 1.)
    } else {
        0.
    };
    (t, a + ab * t)
}

/// Point positions plus the polygons being built.
struct Build {
    pos: Vec<DVec3>,
    /// (corners, kind, source, source polygon or u32::MAX).
    faces: Vec<(Vec<u32>, u8, u32, u32)>,
    problems: Vec<String>,
}

impl Build {
    fn point(&mut self, p: DVec3) -> u32 {
        self.pos.push(p);
        (self.pos.len() - 1) as u32
    }
}

/// Step 1: the join lines.
fn chains(sharp: &Joined, problems: &mut Vec<String>) -> Vec<Chain> {
    let mut dir: HashMap<(u32, u32), usize> = HashMap::new();
    for (t, tri) in sharp.tris.iter().enumerate() {
        for k in 0..3 {
            dir.insert((tri[k], tri[(k + 1) % 3]), t);
        }
    }
    let mut neighbours: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (&(a, b), &t) in &dir {
        if a < b
            && let Some(&u) = dir.get(&(b, a))
            && sharp.source[t] != sharp.source[u]
        {
            neighbours.entry(a).or_default().push(b);
            neighbours.entry(b).or_default().push(a);
        }
    }
    for list in neighbours.values_mut() {
        list.sort();
    }
    let mut sources_at: HashMap<u32, HashSet<u32>> = HashMap::new();
    for (t, tri) in sharp.tris.iter().enumerate() {
        for &v in tri {
            if neighbours.contains_key(&v) {
                sources_at.entry(v).or_default().insert(sharp.source[t]);
            }
        }
    }
    let corner = |v: u32| neighbours[&v].len() != 2 || sources_at[&v].len() >= 3;
    let mut used: HashSet<(u32, u32)> = HashSet::new();
    let mut out = Vec::new();
    let walk = |start: u32, next: u32, used: &mut HashSet<(u32, u32)>| -> Chain {
        let mut verts = vec![start];
        let (mut prev, mut at) = (start, next);
        used.insert(edge_key(prev, at));
        loop {
            if at == start {
                break;
            }
            verts.push(at);
            if corner(at) {
                break;
            }
            let Some(&n) = neighbours[&at]
                .iter()
                .find(|&&n| n != prev && !used.contains(&edge_key(at, n)))
            else {
                break;
            };
            used.insert(edge_key(at, n));
            (prev, at) = (at, n);
        }
        let closed = at == start && !corner(start);
        let left = dir[&(verts[0], verts[1])];
        let right = dir[&(verts[1], verts[0])];
        let mut cum = vec![0.];
        let n = verts.len();
        let segs = if closed { n } else { n - 1 };
        for i in 0..segs {
            let d = sharp.pos[verts[i] as usize].distance(sharp.pos[verts[(i + 1) % n] as usize]);
            cum.push(cum[i] + d);
        }
        Chain {
            verts,
            closed,
            sides: (sharp.source[left], sharp.source[right]),
            cum,
        }
    };
    let starts: Vec<u32> = neighbours.keys().copied().filter(|&v| corner(v)).collect();
    for &s in &starts {
        for &n in &neighbours[&s].clone() {
            if !used.contains(&edge_key(s, n)) {
                out.push(walk(s, n, &mut used));
            }
        }
    }
    let rest: Vec<u32> = neighbours.keys().copied().collect();
    for s in rest {
        for &n in &neighbours[&s].clone() {
            if !used.contains(&edge_key(s, n)) {
                out.push(walk(s, n, &mut used));
            }
        }
    }
    for c in &out {
        // The same pair of sources the whole way along.
        for w in c.verts.windows(2) {
            let (l, r) = (dir[&(w[0], w[1])], dir[&(w[1], w[0])]);
            if (sharp.source[l], sharp.source[r]) != c.sides {
                problems.push("a join changes sources without a corner".into());
                break;
            }
        }
    }
    out
}

/// Strips of one width and profile on every join.
pub fn build(sharp: &Joined, width: f64, profile: f64, rows: u32) -> Strips {
    build_with(sharp, rows, |_| StripParams { width, profile })
}

/// The stable id of every chain (ST-07).
fn strip_ids(sharp: &Joined, chains: &[Chain]) -> Vec<StripInfo> {
    let item = |s: u32| sharp.items.get(s as usize).copied().unwrap_or(u64::from(s));
    let key = |ch: &Chain| {
        let low = ch
            .verts
            .iter()
            .map(|&v| sharp.pos[v as usize])
            .min_by(|a, b| {
                a.y.total_cmp(&b.y)
                    .then(a.x.total_cmp(&b.x))
                    .then(a.z.total_cmp(&b.z))
            })
            .unwrap_or_default();
        (low.y, low.x, low.z, ch.length())
    };
    let mut order: Vec<usize> = (0..chains.len()).collect();
    order.sort_by(|&x, &y| {
        let (a, b) = (key(&chains[x]), key(&chains[y]));
        a.0.total_cmp(&b.0)
            .then(a.1.total_cmp(&b.1))
            .then(a.2.total_cmp(&b.2))
            .then(a.3.total_cmp(&b.3))
    });
    let mut out: Vec<Option<StripInfo>> = vec![None; chains.len()];
    let mut seen: HashMap<(u64, u64), usize> = HashMap::new();
    for c in order {
        let (x, y) = (item(chains[c].sides.0), item(chains[c].sides.1));
        let pair = (x.min(y), x.max(y));
        let n = seen.entry(pair).or_default();
        out[c] = Some(StripInfo {
            id: format!("{}-{}-{}", pair.0, pair.1, *n),
            items: pair,
            length: chains[c].length(),
        });
        *n += 1;
    }
    out.into_iter().map(|s| s.expect("every chain")).collect()
}

/// Strips on every join, each with the settings `params` gives its id.
/// Rows are shared, so corner patches always meet strips of the same row
/// count (SB-34).
pub fn build_with(sharp: &Joined, rows: u32, params: impl Fn(&StripInfo) -> StripParams) -> Strips {
    let mut problems = Vec::new();
    let chains = chains(sharp, &mut problems);
    let infos = strip_ids(sharp, &chains);
    let chain_params: Vec<StripParams> = infos.iter().map(&params).collect();
    let widest = chain_params.iter().map(|p| p.width).fold(0., f64::max);
    let narrowest = chain_params
        .iter()
        .map(|p| p.width)
        .fold(f64::INFINITY, f64::min);
    let pos = &sharp.pos;
    let nv = pos.len();

    // ── Step 2: distance to the nearest bordering join ────────────────
    // Segments, numbered across chains.
    let mut segs: Vec<(usize, usize)> = Vec::new();
    for (c, ch) in chains.iter().enumerate() {
        for i in 0..ch.segments() {
            segs.push((c, i));
        }
    }
    let seg_dist = |s: usize, p: DVec3| {
        let (c, i) = segs[s];
        let (a, b) = chains[c].ends(pos, i);
        closest_on_segment(p, a, b).1.distance(p)
    };
    let mut first_seg = vec![0usize];
    for ch in &chains {
        first_seg.push(first_seg.last().unwrap() + ch.segments());
    }
    let seg_near = |s: usize| -> [usize; 3] {
        let (c, i) = segs[s];
        let n = chains[c].segments();
        let wrap = |j: usize| first_seg[c] + j;
        let prev = if i > 0 {
            i - 1
        } else if chains[c].closed {
            n - 1
        } else {
            i
        };
        let next = if i + 1 < n {
            i + 1
        } else if chains[c].closed {
            0
        } else {
            i
        };
        [wrap(prev), s, wrap(next)]
    };
    let mut on_join = vec![false; nv];
    let mut join_segs: HashMap<u32, Vec<usize>> = HashMap::new();
    for (s, &(c, i)) in segs.iter().enumerate() {
        let ch = &chains[c];
        for v in [ch.verts[i], ch.verts[(i + 1) % ch.verts.len()]] {
            on_join[v as usize] = true;
            join_segs.entry(v).or_default().push(s);
        }
    }
    let mut vsource = vec![u32::MAX; nv];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); nv];
    for (t, tri) in sharp.tris.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (tri[k], tri[(k + 1) % 3]);
            adj[a as usize].push(b);
            if !on_join[a as usize] {
                vsource[a as usize] = sharp.source[t];
            }
        }
    }
    for a in &mut adj {
        a.sort();
        a.dedup();
    }
    let mut best: Vec<(f64, usize)> = vec![(f64::INFINITY, usize::MAX); nv];
    let mut heap = BinaryHeap::new();
    let mut joins: Vec<u32> = join_segs.keys().copied().collect();
    joins.sort();
    for &v in &joins {
        best[v as usize] = (0., join_segs[&v][0]);
        heap.push(Reverse((0u64, v)));
    }
    let reach = 3. * widest;
    while let Some(Reverse((dbits, v))) = heap.pop() {
        let d = f64::from_bits(dbits);
        if d > best[v as usize].0 || d > reach {
            continue;
        }
        let candidates: Vec<usize> = if on_join[v as usize] {
            join_segs[&v].clone()
        } else {
            seg_near(best[v as usize].1).to_vec()
        };
        for &u in &adj[v as usize] {
            if on_join[u as usize] {
                continue;
            }
            let su = vsource[u as usize];
            for &s in &candidates {
                let sides = chains[segs[s].0].sides;
                if su != sides.0 && su != sides.1 {
                    continue;
                }
                let du = seg_dist(s, pos[u as usize]);
                if du < best[u as usize].0 {
                    best[u as usize] = (du, s);
                    heap.push(Reverse((du.to_bits(), u)));
                }
            }
        }
    }
    // f = distance - width of the nearest join's strip, snapped to 0 near
    // the cut line.
    let f: Vec<f64> = (0..nv)
        .map(|v| {
            let (d, seg) = best[v];
            if seg == usize::MAX {
                return d;
            }
            let width = chain_params[segs[seg].0].width;
            let x = d - width;
            if x.abs() < 0.02 * width { 0. } else { x }
        })
        .collect();

    // ── Step 3: cut ────────────────────────────────────────────────────
    let mut b = Build {
        pos: pos.clone(),
        faces: Vec::new(),
        problems: Vec::new(),
    };
    let mut crossing: HashMap<(u32, u32), u32> = HashMap::new();
    // Kept pieces: corners, sharp triangle, untouched.
    let mut pieces: Vec<(Vec<u32>, usize, bool)> = Vec::new();
    for (t, tri) in sharp.tris.iter().enumerate() {
        let fv = tri.map(|v| f[v as usize]);
        if fv.iter().all(|&x| x >= 0.) {
            pieces.push((tri.to_vec(), t, true));
            continue;
        }
        let mut out = Vec::new();
        for k in 0..3 {
            let (p, q) = (tri[k], tri[(k + 1) % 3]);
            let (fp, fq) = (fv[k], fv[(k + 1) % 3]);
            if fp >= 0. {
                out.push(p);
            }
            if (fp > 0. && fq < 0.) || (fp < 0. && fq > 0.) {
                let key = edge_key(p, q);
                let x = *crossing.entry(key).or_insert_with(|| {
                    let s = fp / (fp - fq);
                    let at = pos[p as usize].lerp(pos[q as usize], s);
                    b.pos.push(at);
                    (b.pos.len() - 1) as u32
                });
                out.push(x);
            }
        }
        if out.len() >= 3 {
            pieces.push((out, t, false));
        }
    }
    // Pieces are "untouched" only if they are the whole sharp triangle.
    for p in &mut pieces {
        p.2 = p.0.len() == 3 && sharp.tris[p.1].to_vec() == p.0;
    }

    // ── Rim: edges of the kept surface with nothing on the other side ──
    let mut alias: Vec<u32> = (0..b.pos.len() as u32).collect();
    fn root(a: &mut [u32], mut i: u32) -> u32 {
        while a[i as usize] != i {
            i = a[i as usize];
        }
        i
    }
    let directed = |pieces: &[(Vec<u32>, usize, bool)]| {
        let mut d: HashMap<(u32, u32), usize> = HashMap::new();
        for (pi, (v, _, _)) in pieces.iter().enumerate() {
            for k in 0..v.len() {
                d.insert((v[k], v[(k + 1) % v.len()]), pi);
            }
        }
        d
    };
    let dmap = directed(&pieces);
    let mut rim_next: BTreeMap<u32, u32> = BTreeMap::new();
    let mut rim_piece: HashMap<(u32, u32), usize> = HashMap::new();
    for (&(a, c), &pi) in &dmap {
        if !dmap.contains_key(&(c, a)) {
            if rim_next.insert(a, c).is_some() {
                problems.push("the cut line touches itself".into());
            }
            rim_piece.insert((a, c), pi);
        }
    }
    let normal_of = |t: usize| {
        let [x, y, z] = sharp.tris[t].map(|v| pos[v as usize]);
        (y - x).cross(z - x).normalize_or_zero()
    };
    let mut loops: Vec<Vec<u32>> = Vec::new();
    let mut seen: HashSet<u32> = HashSet::new();
    for &s in rim_next.keys() {
        if seen.contains(&s) {
            continue;
        }
        let mut l = vec![s];
        seen.insert(s);
        let mut at = rim_next[&s];
        while at != s {
            if !seen.insert(at) {
                break;
            }
            l.push(at);
            match rim_next.get(&at) {
                Some(&n) => at = n,
                None => break,
            }
        }
        loops.push(l);
    }
    // Rim points closer than 1 % of the width merge (ST-10: no tiny
    // edges); the polygons around them follow through `alias`.
    let mut merged: HashSet<u32> = HashSet::new();
    for l in &mut loops {
        let n = l.len();
        for k in 0..n {
            let (x, y) = (root(&mut alias, l[k]), root(&mut alias, l[(k + 1) % n]));
            if x != y && b.pos[x as usize].distance(b.pos[y as usize]) < 0.01 * narrowest {
                let mid = (b.pos[x as usize] + b.pos[y as usize]) * 0.5;
                let (keep, gone) = (x.min(y), x.max(y));
                alias[gone as usize] = keep;
                b.pos[keep as usize] = mid;
                merged.insert(keep);
                merged.insert(gone);
            }
        }
        let mut v: Vec<u32> = l.iter().map(|&x| root(&mut alias, x)).collect();
        v.dedup();
        while v.len() > 1 && v.first() == v.last() {
            v.pop();
        }
        *l = v;
    }
    let rim_next: BTreeMap<u32, u32> = rim_next
        .iter()
        .map(|(&a, &c)| (root(&mut alias, a), root(&mut alias, c)))
        .filter(|(a, c)| a != c)
        .collect();
    let rim_piece: HashMap<(u32, u32), usize> = rim_piece
        .iter()
        .map(|(&(a, c), &pi)| ((root(&mut alias, a), root(&mut alias, c)), pi))
        .filter(|((a, c), _)| a != c)
        .collect();

    // ── Feet on joins, runs, corners ───────────────────────────────────
    // Chains each source borders.
    let mut borders: HashMap<u32, Vec<usize>> = HashMap::new();
    for (c, ch) in chains.iter().enumerate() {
        borders.entry(ch.sides.0).or_default().push(c);
        borders.entry(ch.sides.1).or_default().push(c);
    }
    struct Run {
        verts: Vec<u32>,
        closed: bool,
        side: u32,
        chain: usize,
    }
    let mut runs: Vec<Run> = Vec::new();
    let mut corner_points: HashSet<u32> = HashSet::new();
    for l in &loops {
        let side = sharp.source[pieces[rim_piece[&(l[0], rim_next[&l[0]])]].1];
        let near = borders.get(&side).cloned().unwrap_or_default();
        if near.is_empty() {
            problems.push("a cut line with no join near it".into());
            continue;
        }
        let chain_of: Vec<usize> = l
            .iter()
            .map(|&v| {
                let p = b.pos[v as usize];
                *near
                    .iter()
                    .min_by(|&&x, &&y| {
                        chains[x]
                            .foot(pos, p)
                            .0
                            .total_cmp(&chains[y].foot(pos, p).0)
                    })
                    .unwrap()
            })
            .collect();
        let n = l.len();
        let Some(start) = (0..n).find(|&i| chain_of[i] != chain_of[(i + n - 1) % n]) else {
            runs.push(Run {
                verts: l.clone(),
                closed: true,
                side,
                chain: chain_of[0],
            });
            continue;
        };
        // Runs of the same chain; at each change the two rim points meet.
        let mut cur: Vec<u32> = Vec::new();
        let mut cur_chain = chain_of[start];
        let mut pending: Vec<Run> = Vec::new();
        for k in 0..n {
            let i = (start + k) % n;
            if chain_of[i] != cur_chain {
                pending.push(Run {
                    verts: std::mem::take(&mut cur),
                    closed: false,
                    side,
                    chain: cur_chain,
                });
                cur_chain = chain_of[i];
            }
            cur.push(l[i]);
        }
        pending.push(Run {
            verts: cur,
            closed: false,
            side,
            chain: cur_chain,
        });
        // Collapse the last point of each run with the first of the next.
        let m = pending.len();
        for r in 0..m {
            let x = *pending[r].verts.last().unwrap();
            let y = pending[(r + 1) % m].verts[0];
            let (rx, ry) = (root(&mut alias, x), root(&mut alias, y));
            if rx != ry {
                let mid = (b.pos[rx as usize] + b.pos[ry as usize]) * 0.5;
                let (keep, gone) = (rx.min(ry), rx.max(ry));
                alias[gone as usize] = keep;
                b.pos[keep as usize] = mid;
            }
        }
        for r in 0..m {
            let y = pending[(r + 1) % m].verts[0];
            pending[r].verts.push(y);
        }
        for r in pending {
            let mut v: Vec<u32> = r.verts.iter().map(|&x| root(&mut alias, x)).collect();
            v.dedup();
            corner_points.insert(v[0]);
            corner_points.insert(*v.last().unwrap());
            runs.push(Run { verts: v, ..r });
        }
    }
    if std::env::var("OPERANDI_B0_DEBUG").is_ok() {
        for (c, ch) in chains.iter().enumerate() {
            eprintln!(
                "chain {c}: closed {} verts {} sides {:?} len {:.3}",
                ch.closed,
                ch.verts.len(),
                ch.sides,
                ch.length()
            );
        }
        for l in &loops {
            eprintln!("rim loop of {}", l.len());
        }
        for r in &runs {
            eprintln!(
                "run chain {} side {} closed {} verts {}",
                r.chain,
                r.side,
                r.closed,
                r.verts.len()
            );
        }
    }
    // Rim direction (after the collapses).
    let mut rim_dir: HashSet<(u32, u32)> = HashSet::new();
    for (&a, &c) in &rim_next {
        let (ra, rc) = (root(&mut alias, a), root(&mut alias, c));
        if ra != rc {
            rim_dir.insert((ra, rc));
        }
    }
    let vnormal = {
        let mut n: HashMap<u32, DVec3> = HashMap::new();
        let mut rims: Vec<_> = rim_piece.iter().map(|(&e, &pi)| (e, pi)).collect();
        rims.sort();
        for ((a, c), pi) in rims {
            let nn = normal_of(pieces[pi].1);
            for v in [a, c] {
                *n.entry(root(&mut alias, v)).or_default() += nn;
            }
        }
        n
    };

    // ── Steps 4–5: stations and loft, per chain ────────────────────────
    let mut inserts: HashMap<(u32, u32), Vec<(f64, u32)>> = HashMap::new();
    for (c, ch) in chains.iter().enumerate() {
        let on = |side: u32| -> Vec<&Run> {
            runs.iter()
                .filter(|r| r.chain == c && r.side == side)
                .collect()
        };
        let (sa, sb) = (on(ch.sides.0), on(ch.sides.1));
        if sa.len() != 1 || sb.len() != 1 {
            problems.push(format!(
                "a join between {} and {} got {} and {} cut lines instead of one each",
                sharp.names[ch.sides.0 as usize],
                sharp.names[ch.sides.1 as usize],
                sa.len(),
                sb.len()
            ));
            continue;
        }
        let len = ch.length().max(1e-300);
        // Each side: its points with their place along the join, 0..1.
        let side_params = |r: &Run| -> Option<Vec<f64>> {
            let raw: Vec<f64> = r
                .verts
                .iter()
                .map(|&v| ch.foot(pos, b.pos[v as usize]).1 / len)
                .collect();
            if ch.closed {
                Some(raw)
            } else {
                let (lo, hi) = (raw[0].min(*raw.last()?), raw[0].max(*raw.last()?));
                if hi - lo <= 0. {
                    return None;
                }
                let u: Vec<f64> = raw
                    .iter()
                    .map(|x| ((x - lo) / (hi - lo)).clamp(0., 1.))
                    .collect();
                Some(u)
            }
        };
        let (Some(ua), Some(ub)) = (side_params(sa[0]), side_params(sb[0])) else {
            problems.push("a join too short to place a strip on".into());
            continue;
        };
        // Rim order with the join's direction: increasing along it. Rim
        // points whose feet land on the same place (round a bend in the
        // join) are spread out along the rim, so no two stations coincide.
        let positions = &b.pos;
        let width = chain_params[c].width;
        let profile = chain_params[c].profile;
        let tie = 0.02 * width / len;
        let ordered = |r: &Run, u: &[f64]| -> Option<(Vec<u32>, Vec<f64>, bool)> {
            let n = u.len();
            if n < 2 {
                return None;
            }
            let step = |i: usize, j: usize| {
                let d = u[j] - u[i];
                if ch.closed { d - d.round() } else { d }
            };
            let rise: f64 = (0..n - 1).map(|i| step(i, i + 1)).sum();
            let forward = rise >= 0.;
            let (mut v, mut uu) = (r.verts.clone(), u.to_vec());
            if !forward {
                v.reverse();
                uu.reverse();
            }
            if r.closed {
                // Start just after the biggest drop (where it wraps).
                let k = (0..n)
                    .max_by(|&i, &j| {
                        let di = uu[(i + n - 1) % n] - uu[i];
                        let dj = uu[(j + n - 1) % n] - uu[j];
                        di.total_cmp(&dj).then(j.cmp(&i))
                    })
                    .unwrap();
                v.rotate_left(k);
                uu.rotate_left(k);
            }
            if uu.windows(2).any(|w| w[1] < w[0] - tie) {
                return None;
            }
            // Arc length along the rim.
            let mut s = vec![0.];
            for i in 0..n - 1 {
                let d = positions[v[i] as usize].distance(positions[v[i + 1] as usize]);
                s.push(s[i] + d);
            }
            let s_end = s[n - 1]
                + if ch.closed {
                    positions[v[n - 1] as usize].distance(positions[v[0] as usize])
                } else {
                    0.
                };
            let mut i = 0;
            while i < n {
                let mut j = i;
                while j + 1 < n && uu[j + 1] - uu[i] <= tie {
                    j += 1;
                }
                if j > i {
                    if j + 1 < n || ch.closed {
                        // Spread i..=j over [u_i, u_next).
                        let (lo, hi) = (uu[i], if j + 1 < n { uu[j + 1] } else { uu[0] + 1. });
                        let (s0, s1) = (s[i], if j + 1 < n { s[j + 1] } else { s_end });
                        for k in i + 1..=j {
                            uu[k] = lo + (hi - lo) * (s[k] - s0) / (s1 - s0).max(1e-300);
                        }
                    } else if i > 0 {
                        // A tie at the open end: spread from the point before.
                        let (lo, hi) = (uu[i - 1], uu[j]);
                        let (s0, s1) = (s[i - 1], s[j]);
                        for k in i..j {
                            uu[k] = lo + (hi - lo) * (s[k] - s0) / (s1 - s0).max(1e-300);
                        }
                    }
                }
                i = j + 1;
            }
            Some((v, uu, forward))
        };
        let (Some((va, ua, fa)), Some((vb, ub, fb))) = (ordered(sa[0], &ua), ordered(sb[0], &ub))
        else {
            problems.push(format!(
                "the strip between {} and {} folds over (cut line runs backwards)",
                sharp.names[ch.sides.0 as usize], sharp.names[ch.sides.1 as usize]
            ));
            continue;
        };
        // Merge the two sides' places into stations.
        let tol = 0.15 * width / len;
        let mut entries: Vec<(f64, usize, usize)> = Vec::new(); // (u, side, index)
        entries.extend(ua.iter().enumerate().map(|(i, &u)| (u, 0, i)));
        entries.extend(ub.iter().enumerate().map(|(i, &u)| (u, 1, i)));
        entries.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
        let mut groups: Vec<[Option<usize>; 2]> = Vec::new();
        let mut gu: Vec<f64> = Vec::new();
        for &(u, side, i) in &entries {
            if let Some(g) = groups.last_mut()
                && g[side].is_none()
                && (u - gu.last().unwrap()).abs() <= tol
            {
                g[side] = Some(i);
                continue;
            }
            let mut g = [None, None];
            g[side] = Some(i);
            groups.push(g);
            gu.push(u);
        }
        if ch.closed
            && groups.len() > 1
            && (gu[0] + 1. - gu.last().unwrap()) <= tol
            && groups[0][0].is_none() != groups.last().unwrap()[0].is_none()
        {
            let last = groups.pop().unwrap();
            gu.pop();
            for s in 0..2 {
                if groups[0][s].is_none() {
                    groups[0][s] = last[s];
                }
            }
        }
        // Points for each station on each side: an existing rim point, or
        // a new one splitting a rim edge. New points on one edge keep at
        // least half their even share of it apart from each other and its
        // ends, so none lands on an existing point (ST-10 tiny edges).
        let mut station = |verts: &[u32], us: &[f64], side: usize, b: &mut Build| -> Vec<u32> {
            let n = verts.len();
            let mut out = vec![u32::MAX; groups.len()];
            let mut on_edge: BTreeMap<usize, Vec<(f64, usize)>> = BTreeMap::new();
            for (gi, (g, &u)) in groups.iter().zip(&gu).enumerate() {
                if let Some(i) = g[side] {
                    out[gi] = verts[i];
                    continue;
                }
                let j = us.partition_point(|&x| x <= u);
                let (i0, u0, u1) = if j == 0 {
                    if !ch.closed {
                        out[gi] = verts[0];
                        continue;
                    }
                    (n - 1, us[n - 1] - 1., us[0])
                } else if j == n {
                    if !ch.closed {
                        out[gi] = verts[n - 1];
                        continue;
                    }
                    (n - 1, us[n - 1], us[0] + 1.)
                } else {
                    (j - 1, us[j - 1], us[j])
                };
                let s = if u1 > u0 { (u - u0) / (u1 - u0) } else { 0.5 };
                on_edge.entry(i0).or_default().push((s.clamp(0., 1.), gi));
            }
            for (i0, mut list) in on_edge {
                list.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
                let m = list.len();
                let (p, q) = (verts[i0], verts[(i0 + 1) % n]);
                for (k, (raw, gi)) in list.into_iter().enumerate() {
                    let s = 0.5 * raw + 0.5 * (k + 1) as f64 / (m + 1) as f64;
                    let at = b.pos[p as usize].lerp(b.pos[q as usize], s);
                    let x = b.point(at);
                    // Keyed by the rim edge as it runs in the surface.
                    let (key, t) = if rim_dir.contains(&(p, q)) {
                        ((p, q), s)
                    } else {
                        ((q, p), 1. - s)
                    };
                    inserts.entry(key).or_default().push((t, x));
                    out[gi] = x;
                }
            }
            out
        };
        let pa = station(&va, &ua, 0, &mut b);
        let pb = station(&vb, &ub, 1, &mut b);
        let ns = pa.len();
        if ns < 2 {
            continue;
        }
        // Profile rows.
        let segs_across = (2 * rows.max(1)) as usize;
        let mut grid: Vec<Vec<u32>> = Vec::with_capacity(ns);
        for i in 0..ns {
            let (a, z) = (b.pos[pa[i] as usize], b.pos[pb[i] as usize]);
            let fa_ = ch.foot(pos, a).2;
            let fz = ch.foot(pos, z).2;
            let crease = (fa_ + fz) * 0.5;
            let tangent = |p: DVec3, v: u32| {
                let n = vnormal
                    .get(&v)
                    .copied()
                    .unwrap_or(DVec3::ZERO)
                    .normalize_or_zero();
                let d = crease - p;
                (d - n * d.dot(n)).normalize_or_zero()
            };
            let (ta, tz) = (tangent(a, pa[i]), tangent(z, pb[i]));
            let pts = profile_points(a, ta, z, tz, crease, profile, segs_across);
            let mut col = vec![pa[i]];
            for p in &pts[1..segs_across] {
                col.push(b.point(*p));
            }
            col.push(pb[i]);
            grid.push(col);
        }
        // Winding: side A's surface runs a_i -> a_{i+1} or the reverse.
        // Runs are in rim order, so stations rising along the join run
        // with side A's rim when that side went forwards; side B's rim
        // must then run the other way.
        let along = fa;
        if fa == fb {
            problems.push("a strip's two cut lines run the same way".into());
            continue;
        }
        let spans = if ch.closed { ns } else { ns - 1 };
        for i in 0..spans {
            let j = (i + 1) % ns;
            for k in 0..segs_across {
                let q = if along {
                    vec![grid[j][k], grid[i][k], grid[i][k + 1], grid[j][k + 1]]
                } else {
                    vec![grid[i][k], grid[j][k], grid[j][k + 1], grid[i][k + 1]]
                };
                b.faces.push((q, 1, ch.sides.0, u32::MAX));
            }
        }
    }

    // ── Emit the surface pieces (with aliases and inserted points) ─────
    for list in inserts.values_mut() {
        list.sort_by(|x, y| x.0.total_cmp(&y.0));
    }
    // Rejoin untouched triangle pairs of one source polygon into quads.
    let mut whole: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    let mut per_face: HashMap<(u32, u32), usize> = HashMap::new();
    for t in 0..sharp.tris.len() {
        *per_face
            .entry((sharp.source[t], sharp.face[t]))
            .or_default() += 1;
    }
    let mut emitted = vec![false; pieces.len()];
    for (pi, p) in pieces.iter().enumerate() {
        if p.2 {
            whole
                .entry((sharp.source[p.1], sharp.face[p.1]))
                .or_default()
                .push(pi);
        }
    }
    for (key, list) in &whole {
        if list.len() != 2 || per_face[key] != 2 {
            continue;
        }
        let (x, y) = (&pieces[list[0]].0, &pieces[list[1]].0);
        if x.iter()
            .chain(y)
            .any(|v| corner_points.contains(v) || merged.contains(v))
        {
            continue;
        }
        if let Some(q) = join_triangles(x, y)
            && !q.windows(2).any(|w| inserts.contains_key(&(w[0], w[1])))
            && !inserts.contains_key(&(q[3], q[0]))
        {
            b.faces.push((q, 0, key.0, key.1));
            emitted[list[0]] = true;
            emitted[list[1]] = true;
        }
    }
    for (pi, (v, t, _)) in pieces.iter().enumerate() {
        if emitted[pi] {
            continue;
        }
        let mut ring: Vec<u32> = Vec::new();
        let mut split_edges = 0;
        let n = v.len();
        for k in 0..n {
            let (p, q) = (v[k], v[(k + 1) % n]);
            ring.push(root(&mut alias, p));
            let (rp, rq) = (root(&mut alias, p), root(&mut alias, q));
            if let Some(list) = inserts.get(&(rp, rq)) {
                split_edges += 1;
                ring.extend(list.iter().map(|x| x.1));
            }
        }
        ring.dedup();
        while ring.len() > 1 && ring.first() == ring.last() {
            ring.pop();
        }
        if ring.len() < 3 {
            continue;
        }
        let src = sharp.source[*t];
        let face = sharp.face[*t];
        if ring.len() <= 4 && split_edges == 0 {
            b.faces.push((ring, 0, src, face));
        } else {
            for tri in fan(&b.pos, &ring) {
                b.faces.push((tri, 0, src, face));
            }
        }
    }

    // ── Step 6: corners ────────────────────────────────────────────────
    fill_corners(&mut b, &corner_points, rows.max(1) as usize);
    problems.append(&mut b.problems);

    let mut mesh = Mesh {
        vertices: b.pos,
        ..Default::default()
    };
    let mut kind = Vec::new();
    let mut source = Vec::new();
    let mut face = Vec::new();
    for (v, k, s, f) in b.faces {
        mesh.faces.push(poly(v));
        kind.push(k);
        source.push(s);
        face.push(f);
    }
    let mesh = crate::mesh_cleanup::remove_unreferenced(&mesh, None)
        .map(|r| r.mesh)
        .unwrap_or(mesh);
    problems.sort();
    problems.dedup();
    Strips {
        mesh,
        kind,
        source,
        face,
        problems,
        strips: infos,
    }
}

/// Two triangles sharing an edge as one quad, in their winding.
fn join_triangles(x: &[u32], y: &[u32]) -> Option<Vec<u32>> {
    for k in 0..3 {
        let (a, c) = (x[k], x[(k + 1) % 3]);
        for j in 0..3 {
            if y[j] == c && y[(j + 1) % 3] == a {
                let xo = x[(k + 2) % 3];
                let yo = y[(j + 2) % 3];
                return Some(vec![a, yo, c, xo]);
            }
        }
    }
    None
}

/// Split a convex ring with extra points along some sides into
/// triangles: a fan from the corner farthest from the line the extra
/// points lie on (ear clipping when that isn't clear).
fn fan(pos: &[DVec3], ring: &[u32]) -> Vec<Vec<u32>> {
    let n = ring.len();
    let p = |i: usize| pos[ring[i % n] as usize];
    // Apex: the point with the largest triangle against its neighbours'
    // line, so it isn't on a run of collinear points.
    let mut apex = 0;
    let mut best = -1.;
    for i in 0..n {
        let a = (p(i + n - 1) - p(i)).cross(p(i + 1) - p(i)).length();
        if a > best {
            best = a;
            apex = i;
        }
    }
    let mut out = Vec::new();
    for k in 1..n - 1 {
        let (i, j) = ((apex + k) % n, (apex + k + 1) % n);
        out.push(vec![ring[apex], ring[i], ring[j]]);
    }
    out
}

/// Profile from `a` (tangent `ta` towards the crease) to `z` (tangent
/// `tz`): `segs + 1` points evenly spaced along the curve.
fn profile_points(
    a: DVec3,
    ta: DVec3,
    z: DVec3,
    tz: DVec3,
    crease: DVec3,
    profile: f64,
    segs: usize,
) -> Vec<DVec3> {
    // Where the two tangent lines pass closest.
    let r = a - z;
    let (aa, bb, cc) = (ta.dot(ta), ta.dot(tz), tz.dot(tz));
    let (dd, ee) = (ta.dot(r), tz.dot(r));
    let den = aa * cc - bb * bb;
    let corner = if den.abs() > 1e-9 {
        let s = (bb * ee - cc * dd) / den;
        let t = (aa * ee - bb * dd) / den;
        if s > 0. && t > 0. {
            (a + ta * s + z + tz * t) * 0.5
        } else {
            crease
        }
    } else {
        crease
    };
    let (u, v) = (
        (a - corner).normalize_or_zero(),
        (z - corner).normalize_or_zero(),
    );
    let alpha = u.dot(v).clamp(-1., 1.).acos();
    let w_round = (alpha * 0.5).sin().max(1e-6);
    let k = profile / 100.;
    let mid = (a + z) * 0.5;
    let q = mid + (corner - mid) * k;
    let w = if k <= 1. {
        1. + k * (w_round - 1.)
    } else {
        w_round
    };
    let at = |t: f64| {
        let (b0, b1, b2) = ((1. - t) * (1. - t), 2. * t * (1. - t), t * t);
        (a * b0 + q * (b1 * w) + z * b2) / (b0 + b1 * w + b2)
    };
    // Even spacing along the curve.
    const N: usize = 128;
    let samples: Vec<DVec3> = (0..=N).map(|i| at(i as f64 / N as f64)).collect();
    let mut cum = vec![0.];
    for i in 0..N {
        cum.push(cum[i] + samples[i].distance(samples[i + 1]));
    }
    let total = cum[N].max(1e-300);
    (0..=segs)
        .map(|k| {
            if k == 0 {
                return a;
            }
            if k == segs {
                return z;
            }
            let target = total * k as f64 / segs as f64;
            let j = cum.partition_point(|&c| c < target).clamp(1, N);
            let s = (target - cum[j - 1]) / (cum[j] - cum[j - 1]).max(1e-300);
            samples[j - 1].lerp(samples[j], s)
        })
        .collect()
}

/// Fill the holes left where strips end: three end profiles of `2 × rows`
/// edges each become three `rows × rows` quad patches around a centre
/// point (SB-34). Any other hole gets a triangle fan and a problem note.
fn fill_corners(b: &mut Build, corners: &HashSet<u32>, rows: usize) {
    let mut directed: HashMap<(u32, u32), u32> = HashMap::new();
    for (v, _, _, _) in &b.faces {
        for k in 0..v.len() {
            *directed.entry((v[k], v[(k + 1) % v.len()])).or_default() += 1;
        }
    }
    let mut next: BTreeMap<u32, u32> = BTreeMap::new();
    for &(x, y) in directed.keys() {
        if !directed.contains_key(&(y, x)) {
            next.insert(x, y);
        }
    }
    let mut seen: HashSet<u32> = HashSet::new();
    let mut holes = Vec::new();
    for &s in next.keys() {
        if seen.contains(&s) {
            continue;
        }
        let mut l = vec![s];
        seen.insert(s);
        let mut at = next[&s];
        while at != s && seen.insert(at) {
            l.push(at);
            match next.get(&at) {
                Some(&n) => at = n,
                None => break,
            }
        }
        holes.push(l);
    }
    for hole in holes {
        // Fill direction: against the existing edges.
        let mut h = hole.clone();
        h.reverse();
        let n = h.len();
        let m = 2 * rows;
        let ks: Vec<usize> = (0..n).filter(|&i| corners.contains(&h[i])).collect();
        let fits = n == 3 * m && ks.len() == 3 && (ks[1] - ks[0]) == m && (ks[2] - ks[1]) == m;
        if !fits {
            b.problems.push(format!(
                "a hole of {n} edges that isn't a three-strip corner"
            ));
            let c = h.iter().fold(DVec3::ZERO, |s, &v| s + b.pos[v as usize]) / n as f64;
            let cv = b.point(c);
            for i in 0..n {
                b.faces
                    .push((vec![h[i], h[(i + 1) % n], cv], 2, 0, u32::MAX));
            }
            continue;
        }
        h.rotate_left(ks[0]);
        let p = |b: &Build, i: usize| b.pos[h[i % n] as usize];
        let centre = (0..n).fold(DVec3::ZERO, |s, i| s + p(b, i)) / n as f64;
        let cv = b.point(centre);
        // Spokes from each side's middle to the centre.
        let mut spokes: Vec<Vec<u32>> = Vec::new();
        for j in 0..3 {
            let mid = h[(j * m + rows) % n];
            let a = b.pos[mid as usize];
            let mut s = vec![mid];
            for k in 1..rows {
                s.push(b.point(a.lerp(centre, k as f64 / rows as f64)));
            }
            s.push(cv);
            spokes.push(s);
        }
        for j in 0..3 {
            let base = j * m;
            let back = &spokes[(j + 2) % 3];
            let fwd = &spokes[j];
            let mut g = vec![vec![0u32; rows + 1]; rows + 1];
            for x in 0..=rows {
                g[x][0] = h[(base + x) % n];
                g[x][rows] = back[x];
            }
            for y in 0..=rows {
                g[0][y] = h[(base + n - y) % n];
                g[rows][y] = fwd[y];
            }
            for x in 1..rows {
                for y in 1..rows {
                    let (u, v) = (x as f64 / rows as f64, y as f64 / rows as f64);
                    let at = |i: usize, k: usize| b.pos[g[i][k] as usize];
                    let coons = at(x, 0) * (1. - v)
                        + at(x, rows) * v
                        + at(0, y) * (1. - u)
                        + at(rows, y) * u
                        - (at(0, 0) * (1. - u) * (1. - v)
                            + at(rows, 0) * u * (1. - v)
                            + at(0, rows) * (1. - u) * v
                            + at(rows, rows) * u * v);
                    g[x][y] = b.point(coons);
                }
            }
            for x in 0..rows {
                for y in 0..rows {
                    b.faces.push((
                        vec![g[x][y], g[x + 1][y], g[x + 1][y + 1], g[x][y + 1]],
                        2,
                        0,
                        u32::MAX,
                    ));
                }
            }
        }
    }
}
