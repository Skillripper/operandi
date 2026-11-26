//! Arc as a Mesh Stack op (`prim.arc.item`).
use crate::mesh_stack_tools::{add, ok, with_base};
use operandi_core::*;
use serde_json::json;

#[test]
fn arc_op_matches_the_arc_primitive() {
    let (mut e, id) = with_base(Mesh::default());
    add(&mut e, id, "prim.arc.item", json!({}));
    assert_eq!(ok(&e, id), arc_geometry(&ArcParameters::default()).unwrap());
}

#[test]
fn arc_op_angle_radius_and_segments() {
    let (mut e, id) = with_base(Mesh::default());
    add(
        &mut e,
        id,
        "prim.arc.item",
        json!({ "angle": std::f64::consts::PI, "radius": 2., "segments": 4 }),
    );
    let m = ok(&e, id);
    assert_eq!(m.vertices.len(), 4 + 1 + 1);
    assert!((m.vertices[4].x + 2.).abs() < 1e-9, "{:?}", m.vertices[4]);
}
