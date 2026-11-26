//! Set Edge Weight as a Mesh Stack op (`setvmap.subd.item`).
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use serde_json::json;

#[test]
fn set_edge_weight_weights_the_chosen_edges() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "setvmap.subd.item", json!({ "MapValue": 0.5 }));
    only(&mut e, op, "0");
    let m = ok(&e, id);
    assert_eq!(m.attributes.creases.len(), 1);
    assert_eq!(m.attributes.creases[0].weight, 0.5);
    assert_eq!(m.vertices, cube().vertices, "geometry unchanged");
    // No Selection: every edge.
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "setvmap.subd.item", json!({ "MapValue": 1.0 }));
    assert_eq!(ok(&e, id).attributes.creases.len(), 12);
}

#[test]
fn set_edge_weight_zero_clears_weights_from_below() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "setvmap.subd.item", json!({ "MapValue": 1.0 }));
    let op = add(&mut e, id, "setvmap.subd.item", json!({}));
    only(&mut e, op, "0,1");
    assert_eq!(ok(&e, id).attributes.creases.len(), 10);
}
