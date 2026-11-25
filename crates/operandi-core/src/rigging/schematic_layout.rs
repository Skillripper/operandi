//! The Schematic's automatic layout (RG-70): where each node goes when the
//! user hasn't moved it. A layered (Sugiyama-style) layout:
//!
//! 1. Nodes that touch nothing in common are laid out apart and stacked
//!    (each connected group is its own block).
//! 2. Wires point the way data flows, so a node's column is one past the
//!    furthest node that feeds it; drivers then move right to sit just
//!    before what they feed. A modifier lands in the column between its
//!    inputs and the item it drives. Loops (RG-02) have one wire turned
//!    round so the columns still exist.
//! 3. Wires that skip columns get a bend point in each column they pass,
//!    so they run between nodes instead of through them.
//! 4. Each column's order is sorted by where its neighbours sit (barycentre
//!    sweeps, then swapping neighbours while that removes crossings),
//!    keeping the order with the fewest crossings.
//! 5. Heights: each node moves level with the ports it connects to, as far
//!    as its column's order and spacing allow (a least-squares fit per
//!    column), so a modifier sits beside the item it drives.
//!
//! Plain geometry, no GUI: the app draws with [`Metrics`] so its ports
//! land where the layout put them.
use crate::rig_schematic::{Node, Schematic};
use std::collections::BTreeMap;

/// The "add channel" port every item node has, both sides (RG-70).
pub const ADD: &str = "+";

/// Node sizes and spacing, in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub node_w: f32,
    /// The title row.
    pub head_h: f32,
    /// The second header row a node with a detail (an expression) has.
    pub detail_h: f32,
    pub row_h: f32,
    /// Between columns (room for the wires).
    pub gap_x: f32,
    /// Between nodes in a column.
    pub gap_y: f32,
    /// Between separate groups.
    pub group_gap: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            node_w: 170.,
            head_h: 22.,
            detail_h: 14.,
            row_h: 18.,
            gap_x: 110.,
            gap_y: 14.,
            group_gap: 40.,
        }
    }
}

impl Metrics {
    /// The header's height (title, and the detail row when there is one).
    pub fn head(&self, n: &Node) -> f32 {
        self.head_h
            + if n.detail.is_some() {
                self.detail_h
            } else {
                0.
            }
    }

    /// Port rows: the longer port list, and on items one more for the
    /// "add channel" port ([`ADD`], RG-70) after both lists.
    pub fn rows(&self, n: &Node) -> usize {
        let ports = n.inputs.len().max(n.outputs.len());
        if n.piece { ports.max(1) } else { ports + 1 }
    }

    pub fn height(&self, n: &Node) -> f32 {
        self.head(n) + self.rows(n) as f32 * self.row_h + 6.
    }

    /// A port's height below the node's top.
    pub fn port_y(&self, n: &Node, port: &str, output: bool) -> Option<f32> {
        let list = if output { &n.outputs } else { &n.inputs };
        let i = match list.iter().position(|p| p == port) {
            Some(i) => i,
            None if port == ADD && !n.piece => self.rows(n) - 1,
            None => return None,
        };
        Some(self.head(n) + (i as f32 + 0.5) * self.row_h)
    }
}

/// Where the nodes go.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    /// Each node's top-left corner, by key.
    pub positions: BTreeMap<String, [f32; 2]>,
    /// Points a wire passes through on its way across other columns, from
    /// its output end to its input end, by index into `Schematic::wires`.
    pub bends: BTreeMap<usize, Vec<[f32; 2]>>,
}

/// One wire between nodes (indices into the node list).
#[derive(Clone, Copy)]
struct Edge {
    from: usize,
    to: usize,
    /// Port heights below each node's top.
    from_y: f32,
    to_y: f32,
    /// Port order within the node, as a fraction, to break ties.
    from_rank: f32,
    to_rank: f32,
    wire: usize,
}

/// A slot in a column: a node, or a bend point of a wire passing through.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Node(usize),
    Bend { edge: usize, step: usize },
}

/// A wire's piece between neighbouring columns.
#[derive(Clone, Copy)]
struct Segment {
    /// The slot in the left column and the one in the right column.
    left: usize,
    right: usize,
    left_y: f32,
    right_y: f32,
    left_rank: f32,
    right_rank: f32,
}

/// The layout for every node in `s` (RG-70).
pub fn layout(s: &Schematic, m: &Metrics) -> Layout {
    let index: BTreeMap<&str, usize> = s
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.key.as_str(), i))
        .collect();
    let mut edges = Vec::new();
    for (wire, w) in s.wires.iter().enumerate() {
        let (Some(&from), Some(&to)) = (index.get(w.from.as_str()), index.get(w.to.as_str()))
        else {
            continue;
        };
        if from == to {
            continue;
        }
        let (a, b) = (&s.nodes[from], &s.nodes[to]);
        let rank = |n: &Node, port: &str, output: bool| {
            let list = if output { &n.outputs } else { &n.inputs };
            let i = list.iter().position(|p| p == port).unwrap_or(0);
            (i as f32 + 0.5) / (list.len().max(1) as f32)
        };
        edges.push(Edge {
            from,
            to,
            from_y: m.port_y(a, &w.from_port, true).unwrap_or(m.head(a)),
            to_y: m.port_y(b, &w.to_port, false).unwrap_or(m.head(b)),
            from_rank: rank(a, &w.from_port, true),
            to_rank: rank(b, &w.to_port, false),
            wire,
        });
    }
    let mut out = Layout::default();
    let mut top = 0f32;
    for group in groups(s.nodes.len(), &edges) {
        let bottom = layout_group(s, m, &edges, &group, top, &mut out);
        top = bottom + m.group_gap;
    }
    out
}

/// The connected groups, each in node order, ordered by their first node.
fn groups(n: usize, edges: &[Edge]) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for e in edges {
        let (a, b) = (root(&mut parent, e.from), root(&mut parent, e.to));
        if a != b {
            parent[a.max(b)] = a.min(b);
        }
    }
    let mut by_root: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let r = root(&mut parent, i);
        by_root.entry(r).or_default().push(i);
    }
    // Lone nodes (items added by hand) go after the wired groups.
    let (wired, lone): (Vec<_>, Vec<_>) = by_root.into_values().partition(|g| g.len() > 1);
    wired.into_iter().chain(lone).collect()
}

/// Lays out one connected group with its top at `top`; returns its bottom.
fn layout_group(
    s: &Schematic,
    m: &Metrics,
    all_edges: &[Edge],
    group: &[usize],
    top: f32,
    out: &mut Layout,
) -> f32 {
    let in_group: std::collections::BTreeSet<usize> = group.iter().copied().collect();
    // Edges of this group, turned to point along the flow (loops broken).
    let mut edges: Vec<Edge> = all_edges
        .iter()
        .filter(|e| in_group.contains(&e.from))
        .copied()
        .collect();
    let reversed = break_loops(group, &edges);
    for (e, r) in edges.iter_mut().zip(&reversed) {
        if *r {
            *e = Edge {
                from: e.to,
                to: e.from,
                from_y: e.to_y,
                to_y: e.from_y,
                from_rank: e.to_rank,
                to_rank: e.from_rank,
                wire: e.wire,
            };
        }
    }
    let column = columns(group, &edges);
    let columns_n = group.iter().map(|i| column[i]).max().unwrap_or(0) + 1;

    // Slots per column, and the segments between neighbouring columns.
    let mut slots: Vec<Slot> = group.iter().map(|&i| Slot::Node(i)).collect();
    let mut slot_of: BTreeMap<usize, usize> =
        group.iter().enumerate().map(|(s, &i)| (i, s)).collect();
    let mut slot_column: Vec<usize> = group.iter().map(|i| column[i]).collect();
    let mut segments: Vec<Segment> = Vec::new();
    let half_row = m.row_h * 0.5;
    for (k, e) in edges.iter().enumerate() {
        let (c0, c1) = (column[&e.from], column[&e.to]);
        let mut prev = slot_of[&e.from];
        let (mut prev_y, mut prev_rank) = (e.from_y, e.from_rank);
        for c in c0 + 1..c1 {
            slots.push(Slot::Bend {
                edge: k,
                step: c - c0 - 1,
            });
            slot_column.push(c);
            let here = slots.len() - 1;
            segments.push(Segment {
                left: prev,
                right: here,
                left_y: prev_y,
                right_y: half_row,
                left_rank: prev_rank,
                right_rank: 0.5,
            });
            prev = here;
            prev_y = half_row;
            prev_rank = 0.5;
        }
        segments.push(Segment {
            left: prev,
            right: slot_of[&e.to],
            left_y: prev_y,
            right_y: e.to_y,
            left_rank: prev_rank,
            right_rank: e.to_rank,
        });
    }
    slot_of.clear();

    // Order within columns.
    let mut order: Vec<Vec<usize>> = vec![Vec::new(); columns_n];
    for (k, &c) in slot_column.iter().enumerate() {
        order[c].push(k);
    }
    order_columns(&mut order, &segments, slots.len());

    // Heights.
    let height = |k: usize| match slots[k] {
        Slot::Node(i) => m.height(&s.nodes[i]),
        Slot::Bend { .. } => m.row_h,
    };
    let mut y = vec![0f32; slots.len()];
    for col in &order {
        let mut at = 0.;
        for &k in col {
            y[k] = at;
            at += height(k) + m.gap_y;
        }
    }
    let gap = |a: usize, b: usize| match (slots[a], slots[b]) {
        (Slot::Bend { .. }, Slot::Bend { .. }) => 4.,
        _ => m.gap_y,
    };
    for round in 0..12 {
        // Alternate sweeps lean on the left, then the right neighbours;
        // the last round on both.
        let left_first = round % 2 == 0;
        let (use_left, use_right) = if round == 11 {
            (true, true)
        } else {
            (left_first, !left_first)
        };
        let cols: Vec<usize> = if left_first {
            (0..columns_n).collect()
        } else {
            (0..columns_n).rev().collect()
        };
        for c in cols {
            // Wanted top for each slot: level with what it connects to.
            let mut want: Vec<(f32, f32)> = order[c].iter().map(|&k| (y[k], 0.05)).collect();
            let pos: BTreeMap<usize, usize> =
                order[c].iter().enumerate().map(|(p, &k)| (k, p)).collect();
            let mut sums: BTreeMap<usize, (f32, f32)> = BTreeMap::new();
            for sg in &segments {
                if use_left && pos.contains_key(&sg.right) {
                    let e = sums.entry(sg.right).or_default();
                    e.0 += y[sg.left] + sg.left_y - sg.right_y;
                    e.1 += 1.;
                }
                if use_right && pos.contains_key(&sg.left) {
                    let e = sums.entry(sg.left).or_default();
                    e.0 += y[sg.right] + sg.right_y - sg.left_y;
                    e.1 += 1.;
                }
            }
            for (k, (sum, n)) in sums {
                want[pos[&k]] = (sum / n, n);
            }
            let seps: Vec<f32> = order[c]
                .windows(2)
                .map(|w| height(w[0]) + gap(w[0], w[1]))
                .collect();
            for (k, v) in order[c].iter().zip(fit_in_order(&want, &seps)) {
                y[*k] = v;
            }
        }
    }

    // Write out, the group's top at `top`.
    let min_y = y.iter().copied().fold(f32::MAX, f32::min);
    let min_y = if min_y == f32::MAX { 0. } else { min_y };
    let mut bottom = top;
    let x = |c: usize| c as f32 * (m.node_w + m.gap_x);
    let mut bend_points: BTreeMap<usize, Vec<(usize, [f32; 2])>> = BTreeMap::new();
    for (k, slot) in slots.iter().enumerate() {
        let ty = y[k] - min_y + top;
        bottom = bottom.max(ty + height(k));
        match *slot {
            Slot::Node(i) => {
                out.positions
                    .insert(s.nodes[i].key.clone(), [x(slot_column[k]), ty]);
            }
            Slot::Bend { edge, step } => {
                let row = ty + half_row;
                let e = &edges[edge];
                let list = bend_points.entry(e.wire).or_default();
                let c = slot_column[k];
                list.push((step * 2, [x(c), row]));
                list.push((step * 2 + 1, [x(c) + m.node_w, row]));
            }
        }
    }
    for (k, e) in edges.iter().enumerate() {
        let Some(mut list) = bend_points.remove(&e.wire) else {
            continue;
        };
        list.sort_by_key(|p| p.0);
        let mut points: Vec<[f32; 2]> = list.into_iter().map(|p| p.1).collect();
        if reversed[k] {
            points.reverse();
        }
        out.bends.insert(e.wire, points);
    }
    bottom
}

/// Which edges to turn round so the group has no loops: the ones that
/// close a loop in a depth-first walk in node order.
fn break_loops(group: &[usize], edges: &[Edge]) -> Vec<bool> {
    let mut out_edges: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (k, e) in edges.iter().enumerate() {
        out_edges.entry(e.from).or_default().push(k);
    }
    // 0 unseen, 1 on the walk, 2 done.
    let mut state: BTreeMap<usize, u8> = group.iter().map(|&i| (i, 0)).collect();
    let mut reversed = vec![false; edges.len()];
    for &start in group {
        if state[&start] != 0 {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        state.insert(start, 1);
        while let Some(top) = stack.last_mut() {
            let node = top.0;
            let list = out_edges.get(&node).map_or(&[][..], |l| l.as_slice());
            if let Some(&k) = list.get(top.1) {
                top.1 += 1;
                let to = edges[k].to;
                match state[&to] {
                    0 => {
                        state.insert(to, 1);
                        stack.push((to, 0));
                    }
                    1 => reversed[k] = true,
                    _ => {}
                }
            } else {
                state.insert(node, 2);
                stack.pop();
            }
        }
    }
    reversed
}

/// Each node's column: one past its furthest feeder; then nodes only
/// feeding others move right to just before the nearest one they feed.
fn columns(group: &[usize], edges: &[Edge]) -> BTreeMap<usize, usize> {
    let mut preds: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let mut succs: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for e in edges {
        preds.entry(e.to).or_default().push(e.from);
        succs.entry(e.from).or_default().push(e.to);
    }
    // Topological order (Kahn), ties in node order.
    let mut waiting: BTreeMap<usize, usize> = group
        .iter()
        .map(|&i| (i, preds.get(&i).map_or(0, Vec::len)))
        .collect();
    let mut ready: std::collections::BTreeSet<usize> = waiting
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(i, _)| *i)
        .collect();
    let mut topo = Vec::with_capacity(group.len());
    while let Some(i) = ready.pop_first() {
        topo.push(i);
        for &t in succs.get(&i).map_or(&[][..], |l| l.as_slice()) {
            let w = waiting.get_mut(&t).expect("in group");
            *w -= 1;
            if *w == 0 {
                ready.insert(t);
            }
        }
    }
    let mut column: BTreeMap<usize, usize> = BTreeMap::new();
    for &i in &topo {
        let c = preds
            .get(&i)
            .map_or(0, |p| p.iter().map(|j| column[j] + 1).max().unwrap_or(0));
        column.insert(i, c);
    }
    for &i in topo.iter().rev() {
        if preds.contains_key(&i) {
            continue;
        }
        if let Some(s) = succs.get(&i)
            && let Some(nearest) = s.iter().map(|j| column[j]).min()
        {
            column.insert(i, nearest.saturating_sub(1));
        }
    }
    column
}

/// Sorts each column to cut crossings: barycentre sweeps left to right
/// and back, then neighbour swaps, keeping the best order seen.
fn order_columns(order: &mut [Vec<usize>], segments: &[Segment], slots: usize) {
    if order.len() < 2 {
        return;
    }
    let mut place = vec![0f32; slots];
    let set_place = |order: &[Vec<usize>], place: &mut [f32]| {
        for col in order {
            for (p, &k) in col.iter().enumerate() {
                place[k] = p as f32;
            }
        }
    };
    set_place(order, &mut place);
    let mut best = order.to_vec();
    let mut best_count = count_all(order, segments, &place);
    for sweep in 0..24 {
        if best_count == 0 {
            break;
        }
        let rightward = sweep % 2 == 0;
        let cols: Vec<usize> = if rightward {
            (1..order.len()).collect()
        } else {
            (0..order.len() - 1).rev().collect()
        };
        for c in cols {
            let mut sums: BTreeMap<usize, (f32, f32)> = BTreeMap::new();
            for sg in segments {
                let (mine, other, rank) = if rightward {
                    (sg.right, sg.left, sg.left_rank)
                } else {
                    (sg.left, sg.right, sg.right_rank)
                };
                if order[c].contains(&mine) {
                    let e = sums.entry(mine).or_default();
                    e.0 += place[other] + rank;
                    e.1 += 1.;
                }
            }
            let mut keyed: Vec<(f32, usize, usize)> = order[c]
                .iter()
                .enumerate()
                .map(|(p, &k)| {
                    let key = sums.get(&k).map_or(p as f32 + 0.5, |(s, n)| s / n);
                    (key, p, k)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            order[c] = keyed.into_iter().map(|t| t.2).collect();
            for (p, &k) in order[c].iter().enumerate() {
                place[k] = p as f32;
            }
        }
        let n = count_all(order, segments, &place);
        if n < best_count {
            best_count = n;
            best = order.to_vec();
        }
    }
    order.clone_from_slice(&best);
    set_place(order, &mut place);
    // Neighbour swaps while they help.
    let mut column_of = vec![0usize; slots];
    for (c, col) in order.iter().enumerate() {
        for &k in col {
            column_of[k] = c;
        }
    }
    for _ in 0..8 {
        let mut better = false;
        for c in 0..order.len() {
            for p in 0..order[c].len().saturating_sub(1) {
                let before = count_near(c, segments, &place, &column_of);
                order[c].swap(p, p + 1);
                place[order[c][p]] = p as f32;
                place[order[c][p + 1]] = (p + 1) as f32;
                if count_near(c, segments, &place, &column_of) < before {
                    better = true;
                } else {
                    order[c].swap(p, p + 1);
                    place[order[c][p]] = p as f32;
                    place[order[c][p + 1]] = (p + 1) as f32;
                }
            }
        }
        if !better {
            break;
        }
    }
}

fn crossing(a: &Segment, b: &Segment, place: &[f32]) -> bool {
    let l = (place[a.left] + a.left_rank) - (place[b.left] + b.left_rank);
    let r = (place[a.right] + a.right_rank) - (place[b.right] + b.right_rank);
    l * r < 0.
}

fn count_all(order: &[Vec<usize>], segments: &[Segment], place: &[f32]) -> usize {
    let mut column_of = vec![0usize; place.len()];
    for (c, col) in order.iter().enumerate() {
        for &k in col {
            column_of[k] = c;
        }
    }
    let mut n = 0;
    for (i, a) in segments.iter().enumerate() {
        for b in &segments[i + 1..] {
            if column_of[a.left] == column_of[b.left] && crossing(a, b, place) {
                n += 1;
            }
        }
    }
    n
}

/// Crossings on both sides of column `c`.
fn count_near(c: usize, segments: &[Segment], place: &[f32], column_of: &[usize]) -> usize {
    let near: Vec<&Segment> = segments
        .iter()
        .filter(|s| column_of[s.left] == c || column_of[s.right] == c)
        .collect();
    let mut n = 0;
    for (i, a) in near.iter().enumerate() {
        for b in &near[i + 1..] {
            if column_of[a.left] == column_of[b.left] && crossing(a, b, place) {
                n += 1;
            }
        }
    }
    n
}

/// The heights closest to `want` (value, weight), in least squares, that
/// keep the order and leave `seps[i]` between slot `i` and `i + 1`: shift
/// each want by the spacing above it, fit a non-decreasing sequence (pool
/// adjacent violators), shift back.
fn fit_in_order(want: &[(f32, f32)], seps: &[f32]) -> Vec<f32> {
    let mut offset = Vec::with_capacity(want.len());
    let mut at = 0.;
    for i in 0..want.len() {
        offset.push(at);
        if let Some(s) = seps.get(i) {
            at += s;
        }
    }
    // Blocks: (weighted sum, weight, count).
    let mut blocks: Vec<(f32, f32, usize)> = Vec::new();
    for (i, &(v, w)) in want.iter().enumerate() {
        blocks.push(((v - offset[i]) * w, w, 1));
        while blocks.len() > 1 {
            let (s1, w1, n1) = blocks[blocks.len() - 1];
            let (s0, w0, n0) = blocks[blocks.len() - 2];
            if s0 / w0 > s1 / w1 {
                blocks.pop();
                let last = blocks.len() - 1;
                blocks[last] = (s0 + s1, w0 + w1, n0 + n1);
            } else {
                break;
            }
        }
    }
    let mut out = Vec::with_capacity(want.len());
    for (s, w, n) in blocks {
        for _ in 0..n {
            out.push(s / w + offset[out.len()]);
        }
    }
    out
}

/// Wire crossings in a placed schematic: each wire as straight lines from
/// its output port through its bends to its input port. For tests and for
/// judging a layout; drawn wires are curves, so this is a close count.
pub fn crossings(
    s: &Schematic,
    m: &Metrics,
    positions: &BTreeMap<String, [f32; 2]>,
    bends: &BTreeMap<usize, Vec<[f32; 2]>>,
) -> usize {
    let node = |k: &str| s.nodes.iter().find(|n| n.key == k);
    let mut lines: Vec<Vec<[f32; 2]>> = Vec::new();
    for (i, w) in s.wires.iter().enumerate() {
        let (Some(a), Some(b)) = (node(&w.from), node(&w.to)) else {
            continue;
        };
        let (Some(pa), Some(pb)) = (positions.get(&a.key), positions.get(&b.key)) else {
            continue;
        };
        let start = [
            pa[0] + m.node_w,
            pa[1] + m.port_y(a, &w.from_port, true).unwrap_or(0.),
        ];
        let end = [pb[0], pb[1] + m.port_y(b, &w.to_port, false).unwrap_or(0.)];
        let mut pts = vec![start];
        pts.extend(bends.get(&i).into_iter().flatten().copied());
        pts.push(end);
        lines.push(pts);
    }
    let cross = |p: [f32; 2], q: [f32; 2], r: [f32; 2], t: [f32; 2]| {
        let d = |a: [f32; 2], b: [f32; 2], c: [f32; 2]| {
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
        };
        let (d1, d2) = (d(p, q, r), d(p, q, t));
        let (d3, d4) = (d(r, t, p), d(r, t, q));
        d1 * d2 < 0. && d3 * d4 < 0.
    };
    let mut n = 0;
    for (i, a) in lines.iter().enumerate() {
        for b in &lines[i + 1..] {
            // Wires sharing a port don't cross each other there.
            let hit = a
                .windows(2)
                .any(|sa| b.windows(2).any(|sb| cross(sa[0], sa[1], sb[0], sb[1])));
            if hit {
                n += 1;
            }
        }
    }
    n
}
