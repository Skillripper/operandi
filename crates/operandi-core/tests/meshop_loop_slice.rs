//! Loop Slice as a Mesh Stack op (`poly.loopSlice.item`): the direct
//! tool's engine across the rings of the op's edges.
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use operandi_core::{LoopSlice, Mesh, slice};
use serde_json::json;

#[test]
fn loop_slice_cuts_the_ring_of_the_chosen_edge() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "poly.loopSlice.item", json!({}));
    only(&mut e, op, "0");
    let m = ok(&e, id);
    // One loop round the cube: four polygons cut in two.
    assert_eq!(m.faces.len(), 10);
    // Matches the direct tool on the same ring.
    let edge = cube().edges()[0];
    let t = LoopSlice {
        rings: vec![edge],
        ..LoopSlice::default()
    };
    let (direct, _, _): (Mesh, _, _) = slice::loop_slice_geometry(&cube(), &t, &[]).unwrap();
    assert_eq!(m, direct);
}

#[test]
fn loop_slice_count_adds_evenly_spaced_loops() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "poly.loopSlice.item", json!({ "count": 3 }));
    only(&mut e, op, "0");
    assert_eq!(ok(&e, id).faces.len(), 6 + 4 * 3);
}
