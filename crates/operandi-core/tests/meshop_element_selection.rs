//! GitHub #19: Mesh Stack selections of edges and points. Edge ops pick
//! edges and Vertex ops points (Select by Index, Randomly, ...); rule-based
//! selection ops (Edges by Angle, Border, Vertex Count, Normal) are worked
//! out again on every evaluation; selection ops can be reordered; captured
//! numbers on a changed mesh say so on the op's row.
use glam::DVec3;
use operandi_core::mesh_stack::OpStatus;
use operandi_core::mesh_stack_editor::{OpRef, SelOpEdit};
use operandi_core::meshop_selection::{Blend, SelKind, SelOp, resolve};
use operandi_core::meshop_types::SelElement;
use operandi_core::{Command, Editor, Mesh, Mode, persistence};

fn cube() -> Mesh {
    operandi_core::cube_geometry(DVec3::ZERO, DVec3::ONE).unwrap()
}
fn with_base(base: Mesh) -> (Editor, u64) {
    let mut e = Editor::default();
    e.scene.new_mesh().unwrap();
    let id = e.scene.items[0].id;
    e.scene.item_mut(id).unwrap().mesh = base.into();
    e.scene.select_item(Some(id));
    (e, id)
}
fn last_op(e: &Editor, item: u64) -> OpRef {
    let stack = e.scene.item(item).unwrap().mesh_stack.as_ref().unwrap();
    OpRef {
        item,
        op: stack.ops.last().unwrap().id,
    }
}
fn picks(kind: SelKind, mesh: &Mesh, element: SelElement) -> Vec<u32> {
    resolve(None, &[SelOp::new(kind)], mesh, element)
        .unwrap()
        .unwrap()
}

#[test]
fn edge_and_vertex_ops_pick_their_own_kind() {
    let random = SelKind::Random {
        seed: 7,
        probability: 50.,
    };
    let edges = picks(random.clone(), &cube(), SelElement::Edge);
    assert!(edges.iter().all(|&e| e < 12), "numbers the 12 edges");
    assert!(!edges.is_empty() && edges.len() < 12);
    let points = picks(random, &cube(), SelElement::Point);
    assert!(points.iter().all(|&p| p < 8));
    // An Edge op's Select by Index names edges, out of range named so.
    let e = resolve(
        None,
        &[SelOp::new(SelKind::Index { index: "20".into() })],
        &cube(),
        SelElement::Edge,
    )
    .unwrap_err();
    assert!(e.contains("12 edges"), "{e}");
}

#[test]
fn rule_ops_pick_by_shape() {
    let c = cube();
    // Every cube edge is a 90° corner.
    let sharp = SelKind::EdgeAngle {
        min: 30.,
        max: 180.,
        convex: true,
        concave: true,
    };
    assert_eq!(picks(sharp, &c, SelElement::Edge).len(), 12);
    let flat = SelKind::EdgeAngle {
        min: 0.,
        max: 10.,
        convex: true,
        concave: true,
    };
    assert!(picks(flat, &c, SelElement::Edge).is_empty());
    // A closed cube has no border; one polygon has four border edges.
    assert!(picks(SelKind::Border {}, &c, SelElement::Edge).is_empty());
    let quad = operandi_core::cube_geometry(DVec3::ZERO, DVec3::new(1., 0., 1.)).unwrap();
    assert_eq!(picks(SelKind::Border {}, &quad, SelElement::Edge).len(), 4);
    assert_eq!(picks(SelKind::Border {}, &quad, SelElement::Point).len(), 4);
    // Quads: all six; triangles: none.
    assert_eq!(
        picks(
            SelKind::VertexCount { min: 4, max: 4 },
            &c,
            SelElement::Polygon
        )
        .len(),
        6
    );
    assert!(
        picks(
            SelKind::VertexCount { min: 3, max: 3 },
            &c,
            SelElement::Polygon
        )
        .is_empty()
    );
    // Normal +Y: each kind uses its own normal. The top polygon faces up;
    // the top edges lean 45° (their points' average), the top corners
    // 54.7°.
    let up = |max| SelKind::Normal {
        x: 0.,
        y: 1.,
        z: 0.,
        min: 0.,
        max,
    };
    assert_eq!(picks(up(10.), &c, SelElement::Polygon).len(), 1);
    assert!(picks(up(10.), &c, SelElement::Edge).is_empty());
    assert_eq!(picks(up(50.), &c, SelElement::Edge).len(), 4);
    assert_eq!(picks(up(60.), &c, SelElement::Point).len(), 4);
    // Bad values are refused, not panics.
    let bad = SelOp::new(SelKind::Normal {
        x: 0.,
        y: 0.,
        z: 0.,
        min: 0.,
        max: 10.,
    });
    assert!(resolve(None, &[bad], &c, SelElement::Polygon).is_err());
}

#[test]
fn an_edge_op_captures_the_selected_edges_and_bevels_only_them() {
    let (mut e, id) = with_base(cube());
    let edges = e.scene.item(id).unwrap().mesh.edges();
    e.scene.selection.mode = Mode::Edge;
    e.scene.selection.item = Some(id);
    e.scene.selection.edges = vec![edges[0], edges[5]];
    e.dispatch(Command::MeshOpCreate("edge.bevel.item".into()))
        .unwrap();
    let r = last_op(&e, id);
    let stack = e.scene.item(id).unwrap().mesh_stack.as_ref().unwrap();
    let sel = stack.ops[0].selection.clone().unwrap();
    assert_eq!(sel.polygons, vec![0, 5]);
    assert_eq!(sel.made_on, Some(12));
    let mut values = serde_json::Map::new();
    values.insert("inset".into(), 0.1.into());
    e.dispatch(Command::MeshOpSet { op: r, values }).unwrap();
    let two = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    assert_eq!(two.statuses, vec![OpStatus::Ok]);
    // Two bevelled edges add two strips, not one per edge of a polygon.
    assert_eq!(two.mesh.faces.len(), 8, "6 + 2 strips");
    // Saved and reopened, the edge numbers and their count come back.
    let back = persistence::decode(&persistence::encode(&e.scene).unwrap()).unwrap();
    let op = &back.items[0].mesh_stack.as_ref().unwrap().ops[0];
    assert_eq!(op.selection, Some(sel));
}

#[test]
fn numbers_on_a_changed_mesh_say_so_on_the_row() {
    let (mut e, id) = with_base(cube());
    e.scene.selection.mode = Mode::Polygon;
    e.scene.selection.item = Some(id);
    e.scene.selection.faces = vec![0];
    e.dispatch(Command::MeshOpCreate("poly.bevel.item".into()))
        .unwrap();
    let r = last_op(&e, id);
    let fine = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    assert_eq!(fine.statuses, vec![OpStatus::Ok]);
    // A Triangulate below changes the polygon count.
    e.dispatch(Command::MeshOpSelect(None)).ok();
    e.scene.selection.faces.clear();
    e.dispatch(Command::MeshOpCreate("pmodel.triangulate.item".into()))
        .unwrap();
    let tri = last_op(&e, id);
    e.dispatch(Command::MeshOpMove {
        op: tri,
        position: 0,
    })
    .unwrap();
    let r2 = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    let i = e
        .scene
        .item(id)
        .unwrap()
        .mesh_stack
        .as_ref()
        .unwrap()
        .index_of(r.op)
        .unwrap();
    let OpStatus::Noted(notes) = &r2.statuses[i] else {
        panic!("{:?}", r2.statuses)
    };
    assert!(notes[0].text.contains("may point at other polygons"));
    assert!(!notes[0].problem);
}

#[test]
fn reordering_selection_ops_changes_a_subtract_and_undoes() {
    let (mut e, id) = with_base(cube());
    e.dispatch(Command::MeshOpCreate("poly.bevel.item".into()))
        .unwrap();
    let r = last_op(&e, id);
    let top = SelOp::new(SelKind::Normal {
        x: 0.,
        y: 1.,
        z: 0.,
        min: 0.,
        max: 10.,
    });
    let all_but_top = SelOp {
        blend: Blend::Subtract,
        ..top.clone()
    };
    let quads = SelOp::new(SelKind::VertexCount { min: 4, max: 4 });
    for s in [quads, all_but_top] {
        e.dispatch(Command::MeshOpSelOp {
            op: r,
            edit: SelOpEdit::Add(s),
        })
        .unwrap();
    }
    let selops = |e: &Editor| {
        e.scene.item(id).unwrap().mesh_stack.as_ref().unwrap().ops[0]
            .selops
            .clone()
    };
    let cube_faces = cube();
    let resolved = |e: &Editor| {
        resolve(None, &selops(e), &cube_faces, SelElement::Polygon)
            .unwrap()
            .unwrap()
    };
    assert_eq!(resolved(&e).len(), 5, "quads less the top");
    let before = selops(&e);
    e.dispatch(Command::MeshOpSelOp {
        op: r,
        edit: SelOpEdit::Move(1, 0),
    })
    .unwrap();
    // Subtract first (from nothing), then Override with all quads.
    assert_eq!(resolved(&e).len(), 6);
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(selops(&e), before);
    assert!(
        e.dispatch(Command::MeshOpSelOp {
            op: r,
            edit: SelOpEdit::Move(5, 0),
        })
        .is_err()
    );
}

#[test]
fn a_vertex_op_captures_points_from_vertex_mode() {
    let (mut e, id) = with_base(cube());
    e.scene.selection.mode = Mode::Vertex;
    e.scene.selection.item = Some(id);
    e.scene.selection.vertices = vec![3, 1];
    e.dispatch(Command::MeshOpCreate("vert.bevel.item".into()))
        .unwrap();
    let stack = e.scene.item(id).unwrap().mesh_stack.as_ref().unwrap();
    let sel = stack.ops[0].selection.as_ref().unwrap();
    assert_eq!(sel.polygons, vec![1, 3]);
    assert_eq!(sel.made_on, Some(8));
}

// ---- Batch 1 rules from Steve Hill's list (#19) ----

fn cylinder() -> Mesh {
    operandi_core::CylinderShape {
        sides: 8,
        ..Default::default()
    }
    .mesh()
    .unwrap()
}
fn n(kind: SelKind, mesh: &Mesh, element: SelElement) -> usize {
    picks(kind, mesh, element).len()
}

#[test]
fn shape_rules_count_corners_area_and_length() {
    let c = cube();
    assert_eq!(
        n(
            SelKind::VertexCount { min: 4, max: 4 },
            &c,
            SelElement::Polygon
        ),
        6
    );
    // N-gons: 5 or more corners, no maximum; the cylinder's two caps.
    let ngons = SelKind::VertexCount { min: 5, max: 0 };
    assert_eq!(n(ngons, &cylinder(), SelElement::Polygon), 2);
    // Each cube side is 1 m²: a 1 m square.
    assert_eq!(
        n(
            SelKind::Area { min: 0.9, max: 1.1 },
            &c,
            SelElement::Polygon
        ),
        6
    );
    assert_eq!(
        n(SelKind::Area { min: 2., max: 0. }, &c, SelElement::Polygon),
        0
    );
    assert_eq!(
        n(
            SelKind::EdgeLength { min: 0.9, max: 1.1 },
            &c,
            SelElement::Edge
        ),
        12
    );
    assert_eq!(
        n(
            SelKind::EdgeLength { min: 1.5, max: 9. },
            &c,
            SelElement::Edge
        ),
        0
    );
    // A cube's corners all bend outward.
    let angle = |convex, concave| SelKind::EdgeAngle {
        min: 30.,
        max: 180.,
        convex,
        concave,
    };
    assert_eq!(n(angle(true, false), &c, SelElement::Edge), 12);
    assert_eq!(n(angle(false, true), &c, SelElement::Edge), 0);
    // Every cube corner has three edges.
    assert_eq!(
        n(
            SelKind::VertexEdges { min: 3, max: 3 },
            &c,
            SelElement::Point
        ),
        8
    );
    assert_eq!(
        n(
            SelKind::VertexEdges { min: 4, max: 0 },
            &c,
            SelElement::Point
        ),
        0
    );
}

#[test]
fn space_rules_use_normals_planes_and_distance() {
    let c = cube();
    // Corner normals lean 54.7° from Y: within 60° picks the top four.
    let up = SelKind::Normal {
        x: 0.,
        y: 1.,
        z: 0.,
        min: 0.,
        max: 60.,
    };
    assert_eq!(n(up, &c, SelElement::Point), 4);
    let above = |include_zero| SelKind::Plane {
        x: 0.,
        y: 0.,
        z: 0.,
        nx: 0.,
        ny: 1.,
        nz: 0.,
        negative: false,
        include_zero,
    };
    assert_eq!(n(above(false), &c, SelElement::Point), 4);
    // The four side polygons' centres lie on the plane.
    assert_eq!(n(above(true), &c, SelElement::Polygon), 5);
    assert_eq!(n(above(false), &c, SelElement::Polygon), 1);
    let corners = SelKind::Distance {
        x: 0.,
        y: 0.,
        z: 0.,
        min: 0.8,
        max: 0.9,
    };
    assert_eq!(n(corners, &c, SelElement::Point), 8);
}

#[test]
fn random_rules_pick_counts_and_whole_parts() {
    let c = cube();
    let count = |number, percent| SelKind::RandomCount {
        seed: 3,
        number,
        percent,
    };
    assert_eq!(n(count(3, 0.), &c, SelElement::Edge), 3);
    assert_eq!(n(count(0, 50.), &c, SelElement::Edge), 6);
    assert_eq!(n(count(99, 0.), &c, SelElement::Edge), 12);
    // Two cubes: parts are picked whole or not at all.
    let mut two = cube();
    let moved = operandi_core::cube_geometry(DVec3::new(3., 0., 0.), DVec3::ONE).unwrap();
    operandi_core::append_mesh(&mut two, moved).unwrap();
    let parts = |seed, chance| SelKind::RandomParts { seed, chance };
    assert_eq!(n(parts(1, 100.), &two, SelElement::Polygon), 12);
    assert_eq!(n(parts(1, 0.), &two, SelElement::Polygon), 0);
    for seed in 0..20 {
        let k = n(parts(seed, 50.), &two, SelElement::Polygon);
        assert!(k.is_multiple_of(6), "seed {seed}: {k}");
    }
}

#[test]
fn symmetric_mirrors_the_selection_below() {
    let c = cube();
    let right = SelOp::new(SelKind::Plane {
        x: 0.,
        y: 0.,
        z: 0.,
        nx: 1.,
        ny: 0.,
        nz: 0.,
        negative: false,
        include_zero: false,
    });
    let mirror = SelOp::new(SelKind::Symmetric {
        x: true,
        y: false,
        z: false,
        tolerance: 1e-4,
    });
    let left = resolve(
        None,
        &[right.clone(), mirror.clone()],
        &c,
        SelElement::Point,
    )
    .unwrap()
    .unwrap();
    assert_eq!(left.len(), 4);
    assert!(left.iter().all(|&v| c.vertices[v as usize].x < 0.));
    // Add keeps both halves.
    let both = SelOp {
        blend: Blend::Add,
        ..mirror
    };
    let all = resolve(None, &[right, both], &c, SelElement::Point)
        .unwrap()
        .unwrap();
    assert_eq!(all.len(), 8);
}

#[test]
fn uv_seams_follow_the_map() {
    let cyl = cylinder();
    let seams = n(
        SelKind::UvSeam {
            map: "Texture".into(),
        },
        &cyl,
        SelElement::Edge,
    );
    assert!(seams > 0 && seams < cyl.edges().len(), "{seams}");
    let missing = SelOp::new(SelKind::UvSeam { map: "Nope".into() });
    assert!(resolve(None, &[missing], &cyl, SelElement::Edge).is_err());
}
