//! Assign Selection Set as a Mesh Stack op (`assignselectionset.meshop.item`).
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use operandi_core::Command;
use operandi_core::mesh_stack_editor::SelOpEdit;
use operandi_core::meshop_selection::{SelKind, SelOp};
use serde_json::json;

#[test]
fn assign_selection_set_stores_the_chosen_polygons_for_ops_above() {
    let (mut e, id) = with_base(cube());
    let set = add(
        &mut e,
        id,
        "assignselectionset.meshop.item",
        json!({ "name": "Top" }),
    );
    only(&mut e, set, "1,3");
    let m = ok(&e, id);
    let stored: Vec<u32> = m.attributes.polygon_sets["Top"].iter().copied().collect();
    assert_eq!(stored, [1, 3]);
    // A Delete above picks the set.
    let del = add(&mut e, id, "delete.meshop.item", json!({}));
    e.dispatch(Command::MeshOpSelOp {
        op: del,
        edit: SelOpEdit::Add(SelOp::new(SelKind::SelectionSet { name: "Top".into() })),
    })
    .unwrap();
    assert_eq!(ok(&e, id).faces.len(), 4);
}

#[test]
fn assign_selection_set_with_no_name_does_nothing() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "assignselectionset.meshop.item", json!({}));
    assert_eq!(ok(&e, id), cube());
}
