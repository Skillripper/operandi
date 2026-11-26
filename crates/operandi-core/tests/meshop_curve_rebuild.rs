//! Curve Rebuild as a Mesh Stack op (`cRebuild.mOp`).
use crate::mesh_stack_tools::{add, curve_squares, ok, with_base};
use glam::DVec3;
use operandi_core::*;
use serde_json::json;

fn line(n: usize) -> Mesh {
    let mut m = Mesh {
        vertices: (0..n).map(|i| DVec3::X * i as f64).collect(),
        ..Default::default()
    };
    let ids: Vec<u32> = (0..n as u32).collect();
    m = curve_commands::make_curve(&m, &ids, false).unwrap().0;
    m.attributes.representations[0] = PolygonRepresentation::Polyline {
        closed: false,
        curve: CurveKind::Line,
    };
    m
}

#[test]
fn curve_rebuild_spaces_points_evenly() {
    let (mut e, id) = with_base(line(3));
    add(&mut e, id, "cRebuild.mOp", json!({ "ptCount": 5 }));
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 1);
    let xs: Vec<f64> = m.faces[0]
        .vertices
        .iter()
        .map(|&v| m.vertices[v as usize].x)
        .collect();
    for (x, want) in xs.iter().zip([0., 0.5, 1., 1.5, 2.]) {
        assert!((x - want).abs() < 1e-9, "{xs:?}");
    }
    assert!(m.is_polyline(0));
}

#[test]
fn curve_rebuild_by_spacing_and_closed_curves() {
    let (mut e, id) = with_base(line(3));
    add(
        &mut e,
        id,
        "cRebuild.mOp",
        json!({ "ptMode": 1, "ptDistance": 0.25 }),
    );
    assert_eq!(ok(&e, id).faces[0].vertices.len(), 9);
    let (mut e, id) = with_base(curve_squares(&[DVec3::ZERO], false));
    add(&mut e, id, "cRebuild.mOp", json!({ "ptCount": 12 }));
    let m = ok(&e, id);
    assert_eq!(m.faces[0].vertices.len(), 12);
    assert_eq!(m.vertices.len(), 12, "the old points go");
}
