//! Curve Sweep as a Mesh Stack op (`curve.sweep`).
use crate::mesh_stack_tools::{add, curve_squares, ok, with_base};
use glam::DVec3;
use operandi_core::*;
use serde_json::json;

/// An open line along X, 2 m long.
fn line() -> Mesh {
    let m = Mesh {
        vertices: vec![DVec3::ZERO, DVec3::X * 2.],
        ..Default::default()
    };
    let mut m = curve_commands::make_curve(&m, &[0, 1], false).unwrap().0;
    m.attributes.representations[0] = PolygonRepresentation::Polyline {
        closed: false,
        curve: CurveKind::Line,
    };
    m
}

#[test]
fn curve_sweep_puts_a_tube_along_a_line() {
    let (mut e, id) = with_base(line());
    add(&mut e, id, "curve.sweep", json!({}));
    let m = ok(&e, id);
    // The line stays; a pen path has one section per point: one ring of 8.
    assert!(m.is_polyline(0));
    assert_eq!(m.faces.len(), 1 + 8);
    for f in 1..m.faces.len() {
        for &v in &m.faces[f].vertices {
            let p = m.vertices[v as usize];
            assert!((DVec3::new(0., p.y, p.z).length() - 0.125).abs() < 1e-9);
        }
    }
    let (mut e, id) = with_base(line());
    add(&mut e, id, "curve.sweep", json!({ "capMode": 3 }));
    assert_eq!(ok(&e, id).faces.len(), 1 + 8 + 2);
}

#[test]
fn curve_sweep_follows_curves_by_steps_and_index() {
    let base = curve_squares(&[DVec3::ZERO, DVec3::X * 3.], false);
    let (mut e, id) = with_base(base.clone());
    add(&mut e, id, "curve.sweep", json!({ "steps": 6 }));
    assert_eq!(ok(&e, id).faces.len(), 2 + 2 * 6 * 8);
    let (mut e, id) = with_base(base);
    add(
        &mut e,
        id,
        "curve.sweep",
        json!({ "steps": 6, "index": "1" }),
    );
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 2 + 6 * 8);
    assert!(m.vertices[m.faces[2].vertices[0] as usize].x > 2.);
}

#[test]
fn curve_sweep_off_or_on_polygons_passes_through() {
    let (mut e, id) = with_base(line());
    add(&mut e, id, "curve.sweep", json!({ "makePolys": false }));
    assert_eq!(ok(&e, id), line());
    let (mut e, id) = with_base(crate::mesh_stack_tools::cube());
    add(&mut e, id, "curve.sweep", json!({}));
    assert_eq!(ok(&e, id), crate::mesh_stack_tools::cube());
}
