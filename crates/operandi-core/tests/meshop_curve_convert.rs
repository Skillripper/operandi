//! Curve Convert as a Mesh Stack op (`curveconvert.meshop.item`).
use crate::mesh_stack_tools::{add, curve_squares, ok, with_base};
use glam::DVec3;
use operandi_core::curve_convert::{CurveType, convert_points};
use operandi_core::*;
use serde_json::json;

fn points(m: &Mesh, f: usize) -> Vec<DVec3> {
    m.faces[f]
        .vertices
        .iter()
        .map(|&v| m.vertices[v as usize])
        .collect()
}

#[test]
fn curve_convert_turns_curves_into_beziers_in_place() {
    let base = curve_squares(&[DVec3::ZERO], false);
    let (mut e, id) = with_base(base.clone());
    add(&mut e, id, "curveconvert.meshop.item", json!({}));
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 1);
    assert_eq!(
        m.attributes.representations[0],
        PolygonRepresentation::Polyline {
            closed: true,
            curve: CurveKind::Bezier
        }
    );
    let want = convert_points(
        &points(&base, 0),
        CurveKind::Curve {
            start_control: false,
            end_control: false,
        },
        true,
        CurveType::Bezier,
    )
    .unwrap();
    assert_eq!(points(&m, 0), want);
    assert_eq!(m.vertices.len(), want.len(), "the old points go");
}

#[test]
fn curve_convert_passes_other_types_through() {
    let base = curve_squares(&[DVec3::ZERO], false);
    let (mut e, id) = with_base(base.clone());
    add(
        &mut e,
        id,
        "curveconvert.meshop.item",
        json!({ "from": 1, "to": 2 }),
    );
    assert_eq!(ok(&e, id), base);
}
