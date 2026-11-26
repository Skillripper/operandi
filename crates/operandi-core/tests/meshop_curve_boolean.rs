//! Curve Boolean as a Mesh Stack op (`curveBoolean.meshop.item`).
use crate::mesh_stack_tools::{add, curve_squares, ok, only, with_base};
use glam::DVec3;
use operandi_core::curve_boolean::{CurveBooleanOp, CurveBooleanOptions, curve_boolean};
use serde_json::json;

#[test]
fn curve_boolean_matches_the_command() {
    let base = curve_squares(&[DVec3::ZERO, DVec3::new(0.5, 0., 0.5)], false);
    for mode in 0..4u32 {
        let (mut e, id) = with_base(base.clone());
        let op = add(
            &mut e,
            id,
            "curveBoolean.meshop.item",
            json!({ "mode": mode }),
        );
        only(&mut e, op, "0");
        let o = CurveBooleanOptions {
            op: CurveBooleanOp::from_index(mode as usize),
            merge: false,
            ..Default::default()
        };
        assert_eq!(
            ok(&e, id),
            curve_boolean(&base, &[0], &o).unwrap(),
            "{mode}"
        );
    }
}

#[test]
fn curve_boolean_passes_polygons_through() {
    let (mut e, id) = with_base(crate::mesh_stack_tools::cube());
    add(&mut e, id, "curveBoolean.meshop.item", json!({}));
    assert_eq!(ok(&e, id), crate::mesh_stack_tools::cube());
}
