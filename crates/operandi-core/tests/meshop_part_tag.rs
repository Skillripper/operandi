//! Part Tag as a Mesh Stack op (`pmodel.partTag.item`).
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use operandi_core::mesh_stack::OpStatus;
use serde_json::json;

#[test]
fn part_tag_names_the_chosen_polygons() {
    let (mut e, id) = with_base(cube());
    let op = add(
        &mut e,
        id,
        "pmodel.partTag.item",
        json!({ "partName": "Lid" }),
    );
    only(&mut e, op, "0,1");
    let m = ok(&e, id);
    let named: Vec<_> = m.faces.iter().map(|f| f.part.as_deref()).collect();
    assert_eq!(named[0], Some("Lid"));
    assert_eq!(named[1], Some("Lid"));
    assert!(named[2..].iter().all(|p| p.is_none()));
}

#[test]
fn part_tag_refuses_a_blank_name() {
    let (mut e, id) = with_base(cube());
    add(
        &mut e,
        id,
        "pmodel.partTag.item",
        json!({ "partName": " " }),
    );
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    assert!(matches!(r.statuses[0], OpStatus::Failed(_)));
    assert_eq!(*r.mesh, cube());
}
