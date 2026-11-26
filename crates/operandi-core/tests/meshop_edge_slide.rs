//! Edge Slide as a Mesh Stack op (`edge.slide.item`): the direct tool's
//! engine on the op's edges.
use crate::mesh_stack_tools::{add, ok, only, with_base};
use glam::DVec3;
use operandi_core::Mesh;
use serde_json::json;

/// A box two cells long in X: its middle loop of edges sits at x = 0.
fn long_box() -> Mesh {
    operandi_core::segmented_cube(DVec3::ZERO, DVec3::ONE, [2, 1, 1]).unwrap()
}
/// Index numbers (as Select by Index takes them) of the middle loop.
fn middle(m: &Mesh) -> String {
    m.edges()
        .iter()
        .enumerate()
        .filter(|(_, (a, b))| {
            m.vertices[*a as usize].x.abs() < 1e-9 && m.vertices[*b as usize].x.abs() < 1e-9
        })
        .map(|(i, _)| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn edge_slide_moves_the_chosen_loop_like_the_direct_tool() {
    let base = long_box();
    let (mut e, id) = with_base(base.clone());
    let op = add(&mut e, id, "edge.slide.item", json!({ "percent": 0.25 }));
    only(&mut e, op, &middle(&base));
    let m = ok(&e, id);
    assert_ne!(m, base, "the loop moved");
    let loop_edges: Vec<(u32, u32)> = base
        .edges()
        .into_iter()
        .filter(|(a, b)| {
            base.vertices[*a as usize].x.abs() < 1e-9 && base.vertices[*b as usize].x.abs() < 1e-9
        })
        .collect();
    let direct = operandi_core::edge_slide_geometry(&base, &loop_edges, 0.25).unwrap();
    assert_eq!(m, direct);
}

#[test]
fn edge_slide_by_distance_and_zero_does_nothing() {
    let base = long_box();
    let (mut e, id) = with_base(base.clone());
    let op = add(
        &mut e,
        id,
        "edge.slide.item",
        json!({ "interpolation": 1, "dist": 0.1 }),
    );
    only(&mut e, op, &middle(&base));
    let m = ok(&e, id);
    let xs: Vec<f64> = m.vertices.iter().map(|p| p.x).collect();
    assert!(xs.iter().any(|x| (x.abs() - 0.1).abs() < 1e-9), "{xs:?}");
    let (mut e, id) = with_base(base.clone());
    add(&mut e, id, "edge.slide.item", json!({}));
    assert_eq!(ok(&e, id), base, "Percent 0 does nothing");
}
