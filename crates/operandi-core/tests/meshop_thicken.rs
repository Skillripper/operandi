//! Thicken as a Mesh Stack op (`poly.thicken.item`): Smooth Shift with
//! Thicken on, as its own entry.
use crate::mesh_stack_tools::{add, cube, ok, with_base};
use operandi_core::meshop_types::op_def;
use serde_json::json;

#[test]
fn thicken_gives_a_sheet_a_back_and_walls() {
    // One quad of the cube, on its own.
    let quad =
        operandi_core::polygon_remove::delete_geometry(&cube(), &[1, 2, 3, 4, 5], &[]).unwrap();
    let (mut e, id) = with_base(quad);
    add(&mut e, id, "poly.thicken.item", json!({ "shift": 0.1 }));
    let m = ok(&e, id);
    // Front, back and four walls.
    assert_eq!(m.faces.len(), 6, "{:?}", m.faces.len());
}

#[test]
fn thicken_defaults_are_smooth_shift_with_thicken_on() {
    let thicken = op_def("poly.thicken.item").unwrap().defaults();
    let mut shift = op_def("poly.smshift.item").unwrap().defaults();
    assert_eq!(thicken["thicken"], true);
    shift["thicken"] = true.into();
    assert_eq!(thicken, shift, "only Thicken differs");
}
