//! Edges to Curves as a Mesh Stack op (`pmodel.edgeToCurve.item`).
use crate::mesh_stack_tools::{add, cube, ok, quad, with_base};
use operandi_core::*;
use serde_json::json;

#[test]
fn edges_to_curves_traces_the_border_by_default() {
    let (mut e, id) = with_base(quad());
    add(&mut e, id, "pmodel.edgeToCurve.item", json!({}));
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 2);
    assert_eq!(m.faces[1].vertices.len(), 4);
    assert_eq!(
        m.attributes.representations[1],
        PolygonRepresentation::Polyline {
            closed: true,
            curve: CurveKind::Line
        }
    );
}

#[test]
fn edges_to_curves_types_modes_and_delete() {
    let (mut e, id) = with_base(quad());
    add(
        &mut e,
        id,
        "pmodel.edgeToCurve.item",
        json!({ "crvType": 1, "remove": true }),
    );
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 1, "the quad went");
    assert!(matches!(
        m.attributes.representations[0],
        PolygonRepresentation::Polyline {
            curve: CurveKind::Curve { .. },
            ..
        }
    ));
    let (mut e, id) = with_base(quad());
    add(&mut e, id, "pmodel.edgeToCurve.item", json!({ "mode": 1 }));
    let m = ok(&e, id);
    assert_eq!(m.faces.len(), 5, "one line per edge");
    let (mut e, id) = with_base(quad());
    add(
        &mut e,
        id,
        "pmodel.edgeToCurve.item",
        json!({ "crvType": 2 }),
    );
    let m = ok(&e, id);
    assert!(matches!(
        m.attributes.representations[1],
        PolygonRepresentation::Polyline {
            curve: CurveKind::Bezier,
            ..
        }
    ));
}

#[test]
fn edges_to_curves_on_a_closed_mesh_has_no_border() {
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "pmodel.edgeToCurve.item", json!({}));
    assert_eq!(ok(&e, id), cube());
}
