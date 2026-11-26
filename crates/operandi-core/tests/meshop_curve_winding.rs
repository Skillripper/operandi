//! Curve Winding Aligner as a Mesh Stack op (`curveWinding.meshop.item`).
use crate::mesh_stack_tools::{add, curve_squares, ok, with_base};
use glam::DVec3;
use operandi_core::*;
use serde_json::json;

fn facing(m: &Mesh, f: usize) -> f64 {
    let p: Vec<DVec3> = m.faces[f]
        .vertices
        .iter()
        .map(|&v| m.vertices[v as usize])
        .collect();
    (0..p.len())
        .map(|i| p[i].cross(p[(i + 1) % p.len()]).y)
        .sum()
}

#[test]
fn winding_aligner_turns_curves_the_same_way() {
    let base = curve_squares(&[DVec3::ZERO, DVec3::X * 3.], true);
    assert!(facing(&base, 0) * facing(&base, 1) < 0., "start opposite");
    let (mut e, id) = with_base(base);
    add(&mut e, id, "curveWinding.meshop.item", json!({}));
    let m = ok(&e, id);
    assert!(facing(&m, 0) > 0. && facing(&m, 1) > 0.);
    let (mut e, id) = with_base(m);
    add(
        &mut e,
        id,
        "curveWinding.meshop.item",
        json!({ "reverse": true }),
    );
    let m = ok(&e, id);
    assert!(facing(&m, 0) < 0. && facing(&m, 1) < 0.);
}

#[test]
fn winding_aligner_passes_polygons_through() {
    let (mut e, id) = with_base(crate::mesh_stack_tools::cube());
    add(&mut e, id, "curveWinding.meshop.item", json!({}));
    assert_eq!(ok(&e, id), crate::mesh_stack_tools::cube());
}
