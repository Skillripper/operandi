//! GitHub #25 and #29: a Mesh Stack Polygon Bevel with Group Polygons on
//! failed (and passed its input through) when its selection ops picked
//! regions a grouped bevel can't take in one go: every polygon of a closed
//! cylinder (Vertex Count 3..25), or random polygons that touch only at a
//! corner (Select N Randomly past about a dozen). The selection was right;
//! the bevel now does each region on its own (MOQ-31, provisional).
use operandi_core::mesh_stack::OpStatus;
use operandi_core::mesh_stack_editor::{OpRef, SelOpEdit};
use operandi_core::meshop_selection::{SelKind, SelOp};
use operandi_core::{Command, Editor, Mesh};

fn top(e: &Editor, item: u64) -> OpRef {
    let stack = e.scene.item(item).unwrap().mesh_stack.as_ref().unwrap();
    OpRef {
        item,
        op: stack.ops.last().unwrap().id,
    }
}

/// Steve's setup: a 12-sided, 3-segment cylinder op with a radial top cap,
/// then a Polygon Bevel (Group Polygons on) driven by `kind`.
fn bevelled(kind: SelKind) -> (Vec<OpStatus>, Mesh, Mesh) {
    let mut e = Editor::default();
    e.scene.new_mesh().unwrap();
    let id = e.scene.items[0].id;
    e.scene.select_item(Some(id));
    e.dispatch(Command::MeshOpCreate("prim.cylinder.item".into()))
        .unwrap();
    let mut v = serde_json::Map::new();
    v.insert("sides".into(), 12.into());
    v.insert("segments".into(), 3.into());
    v.insert("capTop".into(), 3.into()); // Radial
    e.dispatch(Command::MeshOpSet {
        op: top(&e, id),
        values: v,
    })
    .unwrap();
    let input = (*e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap()
        .mesh)
        .clone();
    e.dispatch(Command::MeshOpCreate("poly.bevel.item".into()))
        .unwrap();
    let bevel = top(&e, id);
    let mut v = serde_json::Map::new();
    v.insert("shift".into(), 0.05.into());
    v.insert("inset".into(), 0.02.into());
    e.dispatch(Command::MeshOpSet {
        op: bevel,
        values: v,
    })
    .unwrap();
    e.dispatch(Command::MeshOpSelOp {
        op: bevel,
        edit: SelOpEdit::Add(SelOp::new(kind)),
    })
    .unwrap();
    let ev = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    (ev.statuses.clone(), (*ev.mesh).clone(), input)
}

/// How many of the input's polygons moved: the bevel's fronts (Bevel
/// keeps the input's polygon numbers and gives fronts their own points).
fn moved(result: &Mesh, input: &Mesh) -> usize {
    let centre = |m: &Mesh, f: usize| {
        let v = &m.faces[f].vertices;
        v.iter()
            .map(|&p| m.vertices[p as usize])
            .sum::<glam::DVec3>()
            / v.len() as f64
    };
    (0..input.faces.len())
        .filter(|&f| centre(result, f).distance(centre(input, f)) > 1e-6)
        .count()
}

#[test]
fn vertex_count_three_to_twenty_five_bevels_every_polygon() {
    let (statuses, mesh, input) = bevelled(SelKind::VertexCount { min: 3, max: 25 });
    assert_eq!(statuses, vec![OpStatus::Ok, OpStatus::Ok], "{statuses:?}");
    // 12 fan triangles, 36 side quads and the 12-sided bottom cap.
    assert_eq!(input.faces.len(), 49);
    assert_eq!(moved(&mesh, &input), 49, "every polygon is bevelled");
    assert!(mesh.validate().is_ok());
}

#[test]
fn select_n_randomly_bevels_for_every_n() {
    for number in 1..=49 {
        let (statuses, mesh, input) = bevelled(SelKind::RandomCount {
            seed: 0,
            number,
            percent: 50.,
        });
        assert_eq!(
            statuses,
            vec![OpStatus::Ok, OpStatus::Ok],
            "N = {number}: {statuses:?}"
        );
        assert_eq!(moved(&mesh, &input), number as usize, "N = {number}");
        assert!(mesh.validate().is_ok(), "N = {number}");
    }
}

#[test]
fn a_selection_that_picks_nothing_still_changes_nothing() {
    // PG-04 (and Steve's note on #29): no selection means no selection.
    let (statuses, mesh, input) = bevelled(SelKind::VertexCount { min: 30, max: 40 });
    assert_eq!(statuses, vec![OpStatus::Ok, OpStatus::Ok]);
    assert_eq!(mesh.faces.len(), input.faces.len());
    assert_eq!(moved(&mesh, &input), 0);
}
