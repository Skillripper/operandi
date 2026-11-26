//! Set Polygon Type as a Mesh Stack op (`poly.setType.meshop.item`).
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use operandi_core::PolygonRepresentation as Rep;
use serde_json::json;

#[test]
fn set_polygon_type_sets_rather_than_toggles() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "poly.setType.meshop.item", json!({ "type": 1 }));
    // Twice in a row still gives subdivision polygons.
    add(&mut e, id, "poly.setType.meshop.item", json!({ "type": 1 }));
    let m = ok(&e, id);
    assert_eq!(m.attributes.representations, vec![Rep::Subdivs; 6]);
    // Back to faces: the compact empty list.
    add(&mut e, id, "poly.setType.meshop.item", json!({}));
    assert!(ok(&e, id).attributes.representations.is_empty());
}

#[test]
fn set_polygon_type_on_the_chosen_polygons_only() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "poly.setType.meshop.item", json!({ "type": 2 }));
    only(&mut e, op, "0");
    let reps = ok(&e, id).attributes.representations.clone();
    assert_eq!(reps[0], Rep::CatmullClark);
    assert!(reps[1..].iter().all(|r| *r == Rep::Face));
}
