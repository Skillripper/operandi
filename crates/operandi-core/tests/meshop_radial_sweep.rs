//! Radial Sweep as a Mesh Stack op (`radial.sweep.item`).
use crate::mesh_stack_tools::{add, ok, only, with_base};
use glam::DVec3;
use operandi_core::*;
use serde_json::json;

/// An open line from (1, 0, 0) to (1, 1, 0).
fn line() -> Mesh {
    let m = Mesh {
        vertices: vec![DVec3::X, DVec3::new(1., 1., 0.)],
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
fn radial_sweep_turns_a_line_into_a_tube() {
    let (mut e, id) = with_base(line());
    add(&mut e, id, "radial.sweep.item", json!({}));
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 8);
    assert!(!m.has_polylines(), "the line is used up");
    for v in &m.vertices {
        assert!((DVec3::new(v.x, 0., v.z).length() - 1.).abs() < 1e-9);
    }
    // Half a turn with caps off: an open strip of four.
    let (mut e, id) = with_base(line());
    add(
        &mut e,
        id,
        "radial.sweep.item",
        json!({ "sides": 4, "end": std::f64::consts::PI, "cap0": false, "cap1": false }),
    );
    assert_eq!(ok(&e, id).faces.len(), 4);
}

#[test]
fn radial_sweep_sweeps_the_chosen_edges() {
    let cube = cube_geometry(DVec3::new(3., 0., 0.), DVec3::ONE).unwrap();
    let (mut e, id) = with_base(cube);
    let op = add(&mut e, id, "radial.sweep.item", json!({}));
    only(&mut e, op, "0");
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 6 + 8);
}

#[test]
fn radial_sweep_refuses_a_zero_axis() {
    let (mut e, id) = with_base(line());
    add(
        &mut e,
        id,
        "radial.sweep.item",
        json!({ "vecX": 0., "vecY": 0., "vecZ": 0. }),
    );
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    assert!(matches!(
        r.statuses[0],
        operandi_core::mesh_stack::OpStatus::Failed(_)
    ));
}
