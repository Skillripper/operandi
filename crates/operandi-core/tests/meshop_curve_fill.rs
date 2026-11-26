//! Curve Fill as a Mesh Stack op (`curvefill`).
use crate::mesh_stack_tools::{add, curve_squares, ok, with_base};
use glam::DVec3;
use operandi_core::*;
use serde_json::json;

#[test]
fn curve_fill_fills_closed_curves_and_removes_them() {
    let (mut e, id) = with_base(curve_squares(&[DVec3::ZERO, DVec3::X * 3.], false));
    add(&mut e, id, "curvefill", json!({}));
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 2);
    assert!(!m.has_polylines());
    assert!(m.faces.iter().all(|f| f.vertices.len() == 4));
}

#[test]
fn curve_fill_keeps_the_curves_and_sets_the_type() {
    let (mut e, id) = with_base(curve_squares(&[DVec3::ZERO], false));
    add(
        &mut e,
        id,
        "curvefill",
        json!({ "delete": false, "polygon_type": 1 }),
    );
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 2);
    assert!(m.is_polyline(0));
    assert_eq!(
        m.attributes.representations[1],
        PolygonRepresentation::Subdivs
    );
}

#[test]
fn curve_fill_flip_turns_the_fill_over() {
    let fill = |flip: bool| {
        let (mut e, id) = with_base(curve_squares(&[DVec3::ZERO], false));
        add(&mut e, id, "curvefill", json!({ "flip": flip }));
        ok(&e, id).normal(0)
    };
    assert!(fill(false).dot(fill(true)) < -0.99);
}
