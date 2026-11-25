//! The Schematic's automatic layout (RG-70): columns follow the data flow,
//! a modifier sits between its inputs and the item it drives and level
//! with it, and a rig shaped like a real imported one (many expressions
//! between a few controls and many driven items) crosses far fewer wires
//! than the old three-column stack.
use operandi_core::rig_schematic::{Node, Schematic};
use operandi_core::schematic_layout::{Layout, Metrics, crossings, layout};
use operandi_core::*;
use std::collections::BTreeMap;

fn ok(e: &mut Editor, text: &str) {
    if let Err(err) = command_line::run_line(e, text).expect("headless") {
        panic!("{text}: {err}");
    }
}

fn item(e: &mut Editor) -> u64 {
    e.scene.new_mesh().unwrap();
    e.scene.items.last().unwrap().id
}

/// An expression on `driven:channel` reading `inputs` (item, channel).
fn expression(e: &mut Editor, inputs: &[(u64, &str)], driven: u64, channel: &str, expr: &str) {
    ok(
        e,
        &format!("operandi.rig.modifierAdd kind:expression item:{driven}"),
    );
    let m = e.scene.animation.rig.pieces.last().unwrap().id;
    let names = ["A", "B", "C"];
    let wired: Vec<String> = inputs
        .iter()
        .zip(names)
        .map(|((i, c), n)| format!("{n}={i}:{c}"))
        .collect();
    ok(
        e,
        &format!(
            "operandi.rig.modifierSet piece:{m} inputs:{} output:{driven}:{channel} expr:{{{expr}}}",
            wired.join(",")
        ),
    );
}

fn node<'a>(s: &'a Schematic, key: &str) -> &'a Node {
    s.nodes.iter().find(|n| n.key == key).unwrap()
}

/// The layout before this one: three columns (drivers, pieces, driven),
/// each stacked top to bottom in build order.
fn old_columns(s: &Schematic, m: &Metrics) -> BTreeMap<String, [f32; 2]> {
    let mut y = [0f32; 3];
    let mut out = BTreeMap::new();
    for n in &s.nodes {
        let c = n.column.min(2) as usize;
        out.insert(n.key.clone(), [c as f32 * (m.node_w + 90.), y[c]]);
        y[c] += m.height(n) + 24.;
    }
    out
}

/// Five controls, twenty expressions, twelve driven items: each driven
/// item takes one or two expressions, each expression reads one or two
/// controls, in an order unrelated to where things end up.
fn hc02_shaped() -> Editor {
    let mut e = Editor::default();
    let controls: Vec<u64> = (0..5).map(|_| item(&mut e)).collect();
    let driven: Vec<u64> = (0..12).map(|_| item(&mut e)).collect();
    let chans = ["pos.X", "pos.Y", "rot.Z", "scl.X"];
    for k in 0..20usize {
        let target = driven[(k * 7) % 12];
        let channel = chans[k % 2 + 1 + (k / 12)];
        let a = controls[(k * 3) % 5];
        let mut inputs = vec![(a, chans[k % 4])];
        if k % 3 == 0 {
            inputs.push((controls[(k + 2) % 5], "pos.Y"));
        }
        expression(&mut e, &inputs, target, channel, "A*2+sin(time)");
    }
    e
}

fn assert_no_overlaps(s: &Schematic, m: &Metrics, l: &Layout) {
    for (i, a) in s.nodes.iter().enumerate() {
        for b in &s.nodes[i + 1..] {
            let (pa, pb) = (l.positions[&a.key], l.positions[&b.key]);
            let apart_x = pa[0] + m.node_w <= pb[0] || pb[0] + m.node_w <= pa[0];
            let apart_y =
                pa[1] + m.height(a) <= pb[1] + 0.01 || pb[1] + m.height(b) <= pa[1] + 0.01;
            assert!(apart_x || apart_y, "{} overlaps {}", a.label, b.label);
        }
    }
}

#[test]
fn rg70_an_hc02_shaped_rig_crosses_far_fewer_wires_than_three_columns() {
    let e = hc02_shaped();
    let s = rig_schematic::build(&e.scene);
    let m = Metrics::default();
    let before = crossings(&s, &m, &old_columns(&s, &m), &BTreeMap::new());
    let l = layout(&s, &m);
    let after = crossings(&s, &m, &l.positions, &l.bends);
    eprintln!("crossings: three columns {before}, layout {after}");
    assert!(before > 100, "the test rig should be a tangle ({before})");
    assert!(after * 2 < before, "{after} crossings left of {before}");
    assert_no_overlaps(&s, &m, &l);
    assert_eq!(l.positions.len(), s.nodes.len());
    // The same rig lays out the same way every time.
    assert_eq!(layout(&s, &m), l);
}

#[test]
fn rg70_an_expression_sits_between_its_input_and_target_level_with_the_target() {
    let mut e = Editor::default();
    let a = item(&mut e);
    let b = item(&mut e);
    expression(&mut e, &[(a, "pos.X")], b, "rot.Y", "A*10");
    let s = rig_schematic::build(&e.scene);
    let m = Metrics::default();
    let l = layout(&s, &m);
    let piece = s.nodes.iter().find(|n| n.piece).unwrap();
    let (ka, kb) = (format!("item:{a}"), format!("item:{b}"));
    let x = |k: &str| l.positions[k][0];
    assert!(x(&ka) < x(&piece.key) && x(&piece.key) < x(&kb));
    // Its output port is level with the driven channel's port.
    let out_y = l.positions[&piece.key][1] + m.port_y(piece, "out", true).unwrap();
    let target = node(&s, &kb);
    let in_y = l.positions[&kb][1] + m.port_y(target, "rot.Y", false).unwrap();
    assert!((out_y - in_y).abs() < 0.5, "{out_y} vs {in_y}");
    assert_eq!(crossings(&s, &m, &l.positions, &l.bends), 0);
    // Labelled with its formula, not just "Expression".
    assert_eq!(piece.detail.as_deref(), Some("A*10"));
}

#[test]
fn rg70_a_chain_gets_a_column_per_step_and_long_wires_bend_between_nodes() {
    let mut e = Editor::default();
    let a = item(&mut e);
    let b = item(&mut e);
    let c = item(&mut e);
    expression(&mut e, &[(a, "pos.X")], b, "pos.Y", "A");
    expression(&mut e, &[(b, "pos.Y"), (a, "pos.Z")], c, "pos.Y", "A+B");
    let s = rig_schematic::build(&e.scene);
    let m = Metrics::default();
    let l = layout(&s, &m);
    let mut xs: Vec<i64> = l.positions.values().map(|p| p[0] as i64).collect();
    xs.sort();
    xs.dedup();
    assert_eq!(xs.len(), 5, "a, expression, b, expression, c");
    // a's pos.Z wire skips two columns: it bends through each.
    assert_eq!(l.bends.len(), 1);
    assert_eq!(l.bends.values().next().unwrap().len(), 4);
    assert_no_overlaps(&s, &m, &l);
}

#[test]
fn rg70_separate_rigs_stack_and_loops_still_lay_out() {
    let mut e = Editor::default();
    let a = item(&mut e);
    let b = item(&mut e);
    let c = item(&mut e);
    let d = item(&mut e);
    ok(
        &mut e,
        &format!("operandi.rig.constrain kind:position items:{b} targets:{a}"),
    );
    ok(
        &mut e,
        &format!("operandi.rig.constrain kind:position items:{d} targets:{c}"),
    );
    // A loop (RG-02): a also follows b.
    e.scene
        .animation
        .rig
        .pieces
        .push(rig::Piece::new(900, "position", a, vec![b]));
    let s = rig_schematic::build(&e.scene);
    let m = Metrics::default();
    let l = layout(&s, &m);
    assert_eq!(l.positions.len(), s.nodes.len());
    assert_no_overlaps(&s, &m, &l);
    // c and d's rig sits wholly below a and b's.
    let bottom = |k: &str| l.positions[k][1] + m.height(node(&s, k));
    let first = [format!("item:{a}"), format!("item:{b}")];
    let second = [format!("item:{c}"), format!("item:{d}")];
    let first_bottom = first.iter().map(|k| bottom(k)).fold(0., f32::max);
    assert!(second.iter().all(|k| l.positions[k][1] > first_bottom));
    // Nothing at all: nothing to place.
    assert_eq!(layout(&Schematic::default(), &m), Layout::default());
}

/// Writes the HC-02-shaped test rig as a scene for GUI checks of the
/// Schematic: `OPERANDI_QA_SCENE=<path> cargo test --locked --workspace
/// -- write_schematic_qa_scene --ignored`.
#[test]
#[ignore = "writes a QA scene; run by hand with OPERANDI_QA_SCENE"]
fn write_schematic_qa_scene() {
    let Some(path) = std::env::var_os("OPERANDI_QA_SCENE") else {
        return;
    };
    let mut e = hc02_shaped();
    for (k, i) in e.scene.items.iter_mut().enumerate() {
        i.name = if k < 5 {
            format!("Control {}", k + 1)
        } else {
            format!("Part {}", k - 4)
        };
    }
    std::fs::write(&path, persistence::encode(&e.scene).unwrap()).unwrap();
    // The old three columns and the new layout, for before/after pictures.
    let s = rig_schematic::build(&e.scene);
    let m = Metrics::default();
    let l = layout(&s, &m);
    let nodes: Vec<serde_json::Value> = s
        .nodes
        .iter()
        .map(|n| {
            serde_json::json!({"key": n.key, "label": n.label, "detail": n.detail,
                "piece": n.piece, "inputs": n.inputs, "outputs": n.outputs,
                "height": m.height(n), "head": m.head(n)})
        })
        .collect();
    let json = serde_json::json!({
        "nodes": nodes, "wires": s.wires, "node_w": m.node_w, "row_h": m.row_h,
        "before": old_columns(&s, &m), "after": l.positions,
        "crossings": [crossings(&s, &m, &old_columns(&s, &m), &BTreeMap::new()),
            crossings(&s, &m, &l.positions, &l.bends)],
    });
    let mut out = path.clone();
    out.push(".layouts.json");
    std::fs::write(out, json.to_string()).unwrap();
}
