//! Chamfer Edit as a Mesh Stack op (`chamfer.edit.item`): reworks a
//! chamfer made lower in the stack.
use crate::mesh_stack_tools::{add, cube, ok, only, with_base};
use serde_json::json;

#[test]
fn chamfer_edit_adds_segments_to_a_chamfer_below() {
    let (mut e, id) = with_base(cube());
    let op = add(&mut e, id, "edge.chamfer.item", json!({ "offset": 0.1 }));
    only(&mut e, op, "0");
    let chamfered = ok(&e, id);
    let strip = chamfered.faces.len() - 1;
    // The new strip is the last polygon: three rows instead of one.
    let op = add(&mut e, id, "chamfer.edit.item", json!({ "segments": 3 }));
    only(&mut e, op, &strip.to_string());
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), chamfered.faces.len() + 2);
    // Matches the direct tool's engine.
    let p = operandi_core::ChamferParameters {
        segments: 3,
        ..Default::default()
    };
    let direct = operandi_core::chamfer::chamfer_geometry(&chamfered, &[strip as u32], p).unwrap();
    assert_eq!(m, direct);
}
