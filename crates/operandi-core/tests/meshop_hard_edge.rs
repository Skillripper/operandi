//! Hard Edge and Hard Edge Convert as Mesh Stack ops.
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use operandi_core::vertex_normals::hard_edges;
use serde_json::json;

#[test]
fn hard_edge_hardens_the_chosen_edges_and_soft_takes_them_back() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "hardedge.meshop.item", json!({}));
    only(&mut e, op, "0,1");
    assert_eq!(hard_edges(&ok(&e, id)).len(), 2);
    // Soft on one of them, above.
    let op = add(&mut e, id, "hardedge.meshop.item", json!({ "type": 1 }));
    only(&mut e, op, "0");
    assert_eq!(hard_edges(&ok(&e, id)).len(), 1);
}

#[test]
fn hard_edge_by_angle_picks_only_sharp_edges() {
    // Every cube edge is 90°: all sharper than 45°, none sharper than 100°.
    let (mut e, id) = with_base(cube());
    add(
        &mut e,
        id,
        "hardedge.meshop.item",
        json!({ "mode": 1, "angle": 45f64.to_radians() }),
    );
    assert_eq!(hard_edges(&ok(&e, id)).len(), 12);
    let (mut e, id) = with_base(cube());
    add(
        &mut e,
        id,
        "hardedge.meshop.item",
        json!({ "mode": 1, "angle": 100f64.to_radians() }),
    );
    assert!(hard_edges(&ok(&e, id)).is_empty());
}

#[test]
fn hard_edge_convert_turns_shading_splits_into_hard_edges() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "hardedgeconvert.meshop.item", json!({}));
    // The default smoothing angle splits a cube's 90° edges.
    assert_eq!(hard_edges(&ok(&e, id)).len(), 12);
}
