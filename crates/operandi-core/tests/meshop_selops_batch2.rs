//! Mesh Stack selection ops, batch 2 (meshops-gaps-2026-11-25): Select
//! Connected, by Polygon Tag, Coplanar Polygons, by Hard Edges and Vertex
//! Map Value; and the Assign Selection Set op is in its own file.
use crate::mesh_stack_tools::{add, cube, ok, with_base};
use glam::DVec3;
use operandi_core::Mesh;
use operandi_core::meshop_selection::{Blend, SelKind, SelOp, resolve};
use operandi_core::meshop_types::SelElement;
use serde_json::json;

fn pick(ops: &[SelOp], mesh: &Mesh, element: SelElement) -> Vec<u32> {
    resolve(None, ops, mesh, element).unwrap().unwrap()
}
fn index(i: &str) -> SelOp {
    SelOp::new(SelKind::Index { index: i.into() })
}
/// Two cubes side by side (two parts), made by an Array op.
fn two_cubes() -> Mesh {
    let (mut e, id) = with_base(cube());
    add(
        &mut e,
        id,
        "array.clone.item",
        json!({ "numX": 2, "offX": 2.0 }),
    );
    let m = ok(&e, id);
    assert!(m.faces.len() >= 12, "{}", m.faces.len());
    m
}

#[test]
fn select_connected_grows_to_whole_parts() {
    let m = two_cubes();
    let ops = [index("0"), SelOp::new(SelKind::Connected {})];
    let faces = pick(&ops, &m, SelElement::Polygon);
    assert_eq!(faces.len(), 6, "one cube: {faces:?}");
    assert!(faces.contains(&0));
    // As edges: one cube's 12.
    let ops = [index("0"), SelOp::new(SelKind::Connected {})];
    assert_eq!(pick(&ops, &m, SelElement::Edge).len(), 12);
    // Nothing below: nothing.
    assert!(
        pick(
            &[SelOp::new(SelKind::Connected {})],
            &m,
            SelElement::Polygon
        )
        .is_empty()
    );
}

#[test]
fn select_by_polygon_tag_reads_material_or_part() {
    let mut m = cube();
    m.faces[2].material = "Glass".into();
    m.faces[4].part = Some("Lid".into());
    let tag = |tag, name: &str| {
        SelOp::new(SelKind::PolygonTag {
            tag,
            name: name.into(),
        })
    };
    assert_eq!(pick(&[tag(0, "Glass")], &m, SelElement::Polygon), [2]);
    assert_eq!(pick(&[tag(1, "Lid")], &m, SelElement::Polygon), [4]);
    assert!(pick(&[tag(1, "Glass")], &m, SelElement::Polygon).is_empty());
    assert!(resolve(None, &[tag(5, "x")], &m, SelElement::Polygon).is_err());
}

#[test]
fn select_coplanar_grows_across_flat_neighbors_only() {
    // A 3x3-cell box: the top's nine polygons are flat, the sides turn 90°.
    let m = operandi_core::segmented_cube(DVec3::ZERO, DVec3::ONE, [3, 3, 3]).unwrap();
    let top: Vec<u32> = (0..m.faces.len() as u32)
        .filter(|&f| m.normal(f as usize).y > 0.99)
        .collect();
    assert_eq!(top.len(), 9);
    let ops = [
        index(&top[0].to_string()),
        SelOp::new(SelKind::Coplanar { angle: 1. }),
    ];
    assert_eq!(pick(&ops, &m, SelElement::Polygon), top);
    // A wide angle crosses the 90° edges, but the bottom faces 180° away
    // from the polygon it all grew from.
    let ops = [
        index(&top[0].to_string()),
        SelOp::new(SelKind::Coplanar { angle: 91. }),
    ];
    assert_eq!(pick(&ops, &m, SelElement::Polygon).len(), m.faces.len() - 9);
}

#[test]
fn select_by_hard_edges_picks_the_marked_edges() {
    let base = cube();
    let (mut e, id) = with_base(base.clone());
    let op = add(&mut e, id, "hardedge.meshop.item", json!({}));
    crate::mesh_stack_tools::only(&mut e, op, "3,5");
    let m = ok(&e, id);
    assert_eq!(
        pick(&[SelOp::new(SelKind::HardEdges {})], &m, SelElement::Edge),
        [3, 5]
    );
    assert!(
        pick(
            &[SelOp::new(SelKind::HardEdges {})],
            &base,
            SelElement::Edge
        )
        .is_empty()
    );
}

#[test]
fn select_vertex_map_value_picks_points_in_range() {
    let mut m = cube();
    let map: std::collections::BTreeMap<u32, f64> = [(0, 1.0), (1, 0.7), (2, 0.2)].into();
    m.attributes.point_weights.insert("Weight".into(), map);
    let op = |min, max| {
        SelOp::new(SelKind::VertexMapValue {
            map: "Weight".into(),
            min,
            max,
        })
    };
    assert_eq!(pick(&[op(0.5, 1.)], &m, SelElement::Point), [0, 1]);
    // Missing points count as 0.
    assert_eq!(pick(&[op(0., 0.)], &m, SelElement::Point), [3, 4, 5, 6, 7]);
    // An unknown map picks nothing.
    let none = SelOp::new(SelKind::VertexMapValue {
        map: "Other".into(),
        min: 0.,
        max: 1.,
    });
    assert!(pick(&[none], &m, SelElement::Point).is_empty());
    // Blends like any other.
    let mut add_op = op(0.1, 0.3);
    add_op.blend = Blend::Add;
    assert_eq!(pick(&[op(0.9, 1.), add_op], &m, SelElement::Point), [0, 2]);
}

#[test]
fn every_new_kind_is_on_the_menu_and_saves() {
    for label in [
        "Select Connected",
        "Select by Polygon Tag",
        "Select Coplanar Polygons",
        "Select by Hard Edges",
        "Select Vertex Map Value",
    ] {
        let (_, kind) = SelKind::menu()
            .into_iter()
            .find(|(l, _)| *l == label)
            .unwrap_or_else(|| panic!("{label} not on the menu"));
        assert_eq!(kind.label(), label);
        let op = SelOp::new(kind);
        let back: SelOp = serde_json::from_value(serde_json::to_value(&op).unwrap()).unwrap();
        assert_eq!(back, op);
    }
}
