//! Handles for the selected primitive MeshOp: they show with the op's row
//! selected and no tool on, and dragging one edits the op live, one undo
//! step per drag.
use super::*;
use crate::viewport::meshop_handles::OpHandle;

fn rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, vec2(800., 600.))
}
fn cylinder_op() -> Operandi {
    let mut a = Operandi::empty();
    a.run(Command::NewMesh);
    a.run(Command::MeshOpCreate("prim.cylinder.item".into()));
    a
}
fn op_values(a: &Operandi) -> serde_json::Value {
    let (_, v) = a
        .selected_primitive_op()
        .expect("a primitive op is selected");
    v
}

#[test]
fn a_selected_primitive_op_shows_its_handles() {
    let a = cylinder_op();
    let handles = a.meshop_handles();
    let kinds: Vec<OpHandle> = handles.iter().map(|h| h.1).collect();
    assert_eq!(
        kinds,
        [
            OpHandle::Centre,
            OpHandle::Size(0),
            OpHandle::Size(1),
            OpHandle::Size(2)
        ]
    );
    // The Y square sits on top: the cylinder's half height, 0.75 m.
    assert!((handles[2].0 - DVec3::new(0., 0.75, 0.)).length() < 1e-9);
}

#[test]
fn no_handles_while_a_tool_is_on_or_for_other_ops() {
    let mut a = cylinder_op();
    a.activate(Tool::Box);
    assert!(a.editor.live.is_some(), "{}", a.status);
    assert!(a.meshop_handles().is_empty());
    let mut b = Operandi::empty();
    b.run(Command::NewMesh);
    b.run(Command::MeshOpCreate("pmodel.triangulate.item".into()));
    assert!(b.meshop_handles().is_empty());
}

#[test]
fn dragging_a_size_square_edits_the_op_live_in_one_undo_step() {
    let mut a = cylinder_op();
    let r = rect();
    let (at, _) = a.meshop_handles()[1];
    let q = a.camera.project(at, r).unwrap().0;
    let grab = a
        .meshop_handle_hit(q, r)
        .expect("the X square is under the pointer");
    assert_eq!(grab.handle, OpHandle::Size(0));
    let history = a.editor.undo.len();
    for step in 1..=4 {
        a.drag_meshop_handle(&grab, q, q + vec2(10. * step as f32, 0.), r);
    }
    let size = op_values(&a)["sizeX"].as_f64().unwrap();
    assert!(size > 0.5, "wider: {size}");
    assert_eq!(a.editor.undo.len(), history + 1, "one step for the drag");
    a.run(Command::Undo);
    assert_eq!(op_values(&a)["sizeX"].as_f64().unwrap(), 0.5);
}

#[test]
fn the_ops_falloff_points_are_handles_too() {
    let mut a = cylinder_op();
    let (r, _) = a.selected_primitive_op().unwrap();
    let mut values = serde_json::Map::new();
    values.insert(
        "falloff".into(),
        serde_json::json!({"first": {"Linear": {"start": [0., 0.75, 0.], "end": [0., -0.75, 0.]}}}),
    );
    values.insert("falloffScale".into(), 0.into());
    a.run(Command::MeshOpSet { op: r, values });
    let handles = a.meshop_handles();
    assert_eq!(handles.len(), 6, "centre, 3 sizes, Start, End");
    let (end, h) = handles[5];
    assert_eq!(h, OpHandle::Falloff(1));
    let rr = rect();
    let q = a.camera.project(end, rr).unwrap().0;
    let grab = crate::viewport::meshop_handles::OpGrab {
        op: r,
        handle: h,
        start: op_values(&a),
        gesture: crate::colour_field::next_gesture(),
    };
    a.drag_meshop_handle(&grab, q, q + vec2(0., -60.), rr);
    let f = &op_values(&a)["falloff"]["first"]["Linear"]["end"];
    assert!(f[1].as_f64().unwrap() > -0.75, "End moved up: {f}");
}
