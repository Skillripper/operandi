//! GitHub #19: Mesh Stack selections of edges and points. Edge ops pick
//! edges and Vertex ops points (Select by Index, Randomly, ...); rule-based
//! selection ops (Edges by Angle, Border, Polygon Sides, Facing) are worked
//! out again on every evaluation; selection ops can be reordered; captured
//! numbers on a changed mesh say so on the op's row.
use glam::DVec3;
use operandi_core::mesh_stack::OpStatus;
use operandi_core::mesh_stack_editor::{OpRef, SelOpEdit};
use operandi_core::meshop_selection::{Blend, SelKind, SelOp, resolve};
use operandi_core::meshop_types::SelElement;
use operandi_core::{Command, Editor, Mesh, Mode, persistence};

fn cube() -> Mesh {
    operandi_core::cube_geometry(DVec3::ZERO, DVec3::ONE).unwrap()
}
fn with_base(base: Mesh) -> (Editor, u64) {
    let mut e = Editor::default();
    e.scene.new_mesh().unwrap();
    let id = e.scene.items[0].id;
    e.scene.item_mut(id).unwrap().mesh = base.into();
    e.scene.select_item(Some(id));
    (e, id)
}
fn last_op(e: &Editor, item: u64) -> OpRef {
    let stack = e.scene.item(item).unwrap().mesh_stack.as_ref().unwrap();
    OpRef {
        item,
        op: stack.ops.last().unwrap().id,
    }
}
fn picks(kind: SelKind, mesh: &Mesh, element: SelElement) -> Vec<u32> {
    resolve(None, &[SelOp::new(kind)], mesh, element)
        .unwrap()
        .unwrap()
}

#[test]
fn edge_and_vertex_ops_pick_their_own_kind() {
    let random = SelKind::Random {
        seed: 7,
        probability: 50.,
    };
    let edges = picks(random.clone(), &cube(), SelElement::Edge);
    assert!(edges.iter().all(|&e| e < 12), "numbers the 12 edges");
    assert!(!edges.is_empty() && edges.len() < 12);
    let points = picks(random, &cube(), SelElement::Point);
    assert!(points.iter().all(|&p| p < 8));
    // An Edge op's Select by Index names edges, out of range named so.
    let e = resolve(
        None,
        &[SelOp::new(SelKind::Index { index: "20".into() })],
        &cube(),
        SelElement::Edge,
    )
    .unwrap_err();
    assert!(e.contains("12 edges"), "{e}");
}

#[test]
fn rule_ops_pick_by_shape() {
    let c = cube();
    // Every cube edge is a 90° corner.
    let sharp = SelKind::EdgeAngle {
        min: 30.,
        max: 180.,
    };
    assert_eq!(picks(sharp, &c, SelElement::Edge).len(), 12);
    let flat = SelKind::EdgeAngle { min: 0., max: 10. };
    assert!(picks(flat, &c, SelElement::Edge).is_empty());
    // A closed cube has no border; one polygon has four border edges.
    assert!(picks(SelKind::Border {}, &c, SelElement::Edge).is_empty());
    let quad = operandi_core::cube_geometry(DVec3::ZERO, DVec3::new(1., 0., 1.)).unwrap();
    assert_eq!(picks(SelKind::Border {}, &quad, SelElement::Edge).len(), 4);
    assert_eq!(picks(SelKind::Border {}, &quad, SelElement::Point).len(), 4);
    // Quads: all six; triangles: none.
    assert_eq!(
        picks(SelKind::Sides { sides: 1 }, &c, SelElement::Polygon).len(),
        6
    );
    assert!(picks(SelKind::Sides { sides: 0 }, &c, SelElement::Polygon).is_empty());
    // Facing +Y: the top polygon, its 4 edges, its 4 points.
    let up = SelKind::Facing {
        x: 0.,
        y: 1.,
        z: 0.,
        angle: 10.,
    };
    assert_eq!(picks(up.clone(), &c, SelElement::Polygon).len(), 1);
    assert_eq!(picks(up.clone(), &c, SelElement::Edge).len(), 4);
    assert_eq!(picks(up, &c, SelElement::Point).len(), 4);
    // Bad values are refused, not panics.
    let bad = SelOp::new(SelKind::Sides { sides: 9 });
    assert!(resolve(None, &[bad], &c, SelElement::Polygon).is_err());
}

#[test]
fn an_edge_op_captures_the_selected_edges_and_bevels_only_them() {
    let (mut e, id) = with_base(cube());
    let edges = e.scene.item(id).unwrap().mesh.edges();
    e.scene.selection.mode = Mode::Edge;
    e.scene.selection.item = Some(id);
    e.scene.selection.edges = vec![edges[0], edges[5]];
    e.dispatch(Command::MeshOpCreate("edge.bevel.item".into()))
        .unwrap();
    let r = last_op(&e, id);
    let stack = e.scene.item(id).unwrap().mesh_stack.as_ref().unwrap();
    let sel = stack.ops[0].selection.clone().unwrap();
    assert_eq!(sel.polygons, vec![0, 5]);
    assert_eq!(sel.made_on, Some(12));
    let mut values = serde_json::Map::new();
    values.insert("inset".into(), 0.1.into());
    e.dispatch(Command::MeshOpSet { op: r, values }).unwrap();
    let two = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    assert_eq!(two.statuses, vec![OpStatus::Ok]);
    // Two bevelled edges add two strips, not one per edge of a polygon.
    assert_eq!(two.mesh.faces.len(), 8, "6 + 2 strips");
    // Saved and reopened, the edge numbers and their count come back.
    let back = persistence::decode(&persistence::encode(&e.scene).unwrap()).unwrap();
    let op = &back.items[0].mesh_stack.as_ref().unwrap().ops[0];
    assert_eq!(op.selection, Some(sel));
}

#[test]
fn numbers_on_a_changed_mesh_say_so_on_the_row() {
    let (mut e, id) = with_base(cube());
    e.scene.selection.mode = Mode::Polygon;
    e.scene.selection.item = Some(id);
    e.scene.selection.faces = vec![0];
    e.dispatch(Command::MeshOpCreate("poly.bevel.item".into()))
        .unwrap();
    let r = last_op(&e, id);
    let fine = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    assert_eq!(fine.statuses, vec![OpStatus::Ok]);
    // A Triangulate below changes the polygon count.
    e.dispatch(Command::MeshOpSelect(None)).ok();
    e.scene.selection.faces.clear();
    e.dispatch(Command::MeshOpCreate("pmodel.triangulate.item".into()))
        .unwrap();
    let tri = last_op(&e, id);
    e.dispatch(Command::MeshOpMove {
        op: tri,
        position: 0,
    })
    .unwrap();
    let r2 = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    let i = e
        .scene
        .item(id)
        .unwrap()
        .mesh_stack
        .as_ref()
        .unwrap()
        .index_of(r.op)
        .unwrap();
    let OpStatus::Noted(notes) = &r2.statuses[i] else {
        panic!("{:?}", r2.statuses)
    };
    assert!(notes[0].text.contains("may point at other polygons"));
    assert!(!notes[0].problem);
}

#[test]
fn reordering_selection_ops_changes_a_subtract_and_undoes() {
    let (mut e, id) = with_base(cube());
    e.dispatch(Command::MeshOpCreate("poly.bevel.item".into()))
        .unwrap();
    let r = last_op(&e, id);
    let top = SelOp::new(SelKind::Facing {
        x: 0.,
        y: 1.,
        z: 0.,
        angle: 10.,
    });
    let all_but_top = SelOp {
        blend: Blend::Subtract,
        ..top.clone()
    };
    let quads = SelOp::new(SelKind::Sides { sides: 1 });
    for s in [quads, all_but_top] {
        e.dispatch(Command::MeshOpSelOp {
            op: r,
            edit: SelOpEdit::Add(s),
        })
        .unwrap();
    }
    let selops = |e: &Editor| {
        e.scene.item(id).unwrap().mesh_stack.as_ref().unwrap().ops[0]
            .selops
            .clone()
    };
    let cube_faces = cube();
    let resolved = |e: &Editor| {
        resolve(None, &selops(e), &cube_faces, SelElement::Polygon)
            .unwrap()
            .unwrap()
    };
    assert_eq!(resolved(&e).len(), 5, "quads less the top");
    let before = selops(&e);
    e.dispatch(Command::MeshOpSelOp {
        op: r,
        edit: SelOpEdit::Move(1, 0),
    })
    .unwrap();
    // Subtract first (from nothing), then Override with all quads.
    assert_eq!(resolved(&e).len(), 6);
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(selops(&e), before);
    assert!(
        e.dispatch(Command::MeshOpSelOp {
            op: r,
            edit: SelOpEdit::Move(5, 0),
        })
        .is_err()
    );
}

#[test]
fn a_vertex_op_captures_points_from_vertex_mode() {
    let (mut e, id) = with_base(cube());
    e.scene.selection.mode = Mode::Vertex;
    e.scene.selection.item = Some(id);
    e.scene.selection.vertices = vec![3, 1];
    e.dispatch(Command::MeshOpCreate("vert.bevel.item".into()))
        .unwrap();
    let stack = e.scene.item(id).unwrap().mesh_stack.as_ref().unwrap();
    let sel = stack.ops[0].selection.as_ref().unwrap();
    assert_eq!(sel.polygons, vec![1, 3]);
    assert_eq!(sel.made_on, Some(8));
}
