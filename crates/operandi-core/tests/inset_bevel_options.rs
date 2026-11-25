//! Inset and Bevel's opt-in options from the TwistBevel notes (2026-11-25):
//! Stop Before Fold, Even Interior and Keep Cap UVs, plus maps and morphs
//! riding along to new points, SubD walls and exact undo/redo.
use glam::DVec3;
use operandi_core::*;

fn face(vertices: &[u32]) -> Face {
    Face {
        vertices: vertices.to_vec(),
        uv: None,
        material: "M".into(),
        material_label: None,
        smooth: false,
        part: None,
    }
}

/// A 3 × 3 grid of 1 m quads on z = 0, with UVs u = x / 3, v = y / 3.
fn grid() -> Mesh {
    let mut vertices = Vec::new();
    for y in 0..4 {
        for x in 0..4 {
            vertices.push(DVec3::new(x as f64, y as f64, 0.));
        }
    }
    let mut faces = Vec::new();
    for y in 0..3u32 {
        for x in 0..3u32 {
            let v = 4 * y + x;
            faces.push(face(&[v, v + 1, v + 5, v + 4]));
        }
    }
    let mut m = Mesh {
        vertices,
        faces: faces.into(),
        ..Default::default()
    };
    let rows = m
        .faces
        .iter()
        .map(|f| {
            f.vertices
                .iter()
                .map(|&v| Some(m.vertices[v as usize].truncate() / 3.))
                .collect()
        })
        .collect();
    m.uv_maps.insert("Texture".into(), rows);
    m
}

fn one_quad() -> Mesh {
    Mesh {
        vertices: vec![
            DVec3::new(0., 0., 0.),
            DVec3::new(1., 0., 0.),
            DVec3::new(1., 1., 0.),
            DVec3::new(0., 1., 0.),
        ],
        faces: vec![face(&[0, 1, 2, 3])].into(),
        ..Default::default()
    }
}

fn bevel(m: &Mesh, faces: &[u32], inset: f64, grouped: bool, p: &BevelOptions) -> Mesh {
    bevel_geometry_with_profile(m, faces, 0., inset, grouped, p, None, None)
        .unwrap()
        .mesh
}

fn cap_points(m: &Mesh, f: usize) -> Vec<DVec3> {
    m.faces[f]
        .vertices
        .iter()
        .map(|&v| m.vertices[v as usize])
        .collect()
}

fn area_z(points: &[DVec3]) -> f64 {
    (1..points.len() - 1)
        .map(|i| (points[i] - points[0]).cross(points[i + 1] - points[0]).z)
        .sum::<f64>()
        / 2.
}

#[test]
fn inset_crosses_over_by_default_and_stops_before_fold_when_asked() {
    let quad = one_quad();
    // Default (PB-C-01/04): 0.8 on a 1 m square crosses over.
    let crossed = inset_geometry(&quad, &[0], 0.8).unwrap().mesh;
    let mut corner = cap_points(&crossed, 0)[0];
    assert!((corner - DVec3::new(0.8, 0.8, 0.)).length() < 1e-9);
    // Stop Before Fold: the copy stops just short of the middle.
    let options = InsetOptions {
        stop_at_fold: true,
        ..Default::default()
    };
    let stopped = inset_geometry_with(&quad, &[0], 0.8, options).unwrap().mesh;
    corner = cap_points(&stopped, 0)[0];
    assert!(corner.x < 0.5 && corner.x > 0.5 - 1e-6, "{corner}");
    // An inset that doesn't fold is untouched by the option.
    let small = inset_geometry_with(&quad, &[0], 0.2, options).unwrap().mesh;
    assert_eq!(small, inset_geometry(&quad, &[0], 0.2).unwrap().mesh);
}

#[test]
fn bevel_stop_before_fold_keeps_the_cap_facing_up() {
    let quad = one_quad();
    let off = bevel(&quad, &[0], 0.8, false, &BevelOptions::default());
    // Past the middle the cap has passed through itself: its first edge
    // now runs backwards.
    let cap = cap_points(&off, 0);
    assert!((cap[1] - cap[0]).x < 0.);
    let p = BevelOptions {
        stop_at_fold: true,
        ..Default::default()
    };
    let on = bevel(&quad, &[0], 0.8, false, &p);
    let cap = cap_points(&on, 0);
    assert!(area_z(&cap) > 0.);
    assert!(
        cap.iter()
            .all(|c| (c.x - 0.5).abs() < 1e-5 && (c.y - 0.5).abs() < 1e-5)
    );
    // Grouped grid: the guard stops before the grouped cap folds too.
    let g = grid();
    let all: Vec<u32> = (0..9).collect();
    let on = bevel(&g, &all, 2.5, true, &p);
    for f in 0..9 {
        assert!(
            area_z(&cap_points(&on, f)) >= 0.,
            "cap polygon {f} turned over"
        );
    }
}

#[test]
fn even_interior_shrinks_a_grouped_cap_evenly() {
    let g = grid();
    let all: Vec<u32> = (0..9).collect();
    let plain = bevel(&g, &all, 0.3, true, &BevelOptions::default());
    let p = BevelOptions {
        even_interior: true,
        ..Default::default()
    };
    let even = bevel(&g, &all, 0.3, true, &p);
    // Cap polygon 0 is the bottom-left quad: corners 0, 1, 5, 4.
    let at = |m: &Mesh, c: usize| cap_points(m, 0)[c];
    // Default: only the outline moves; the interior point stays.
    assert!((at(&plain, 1) - DVec3::new(1., 0.3, 0.)).length() < 1e-9);
    assert!((at(&plain, 2) - DVec3::new(1., 1., 0.)).length() < 1e-9);
    // Even Interior: the straight-run point keeps its third of the moved
    // edge (0.3 + 2.4 / 3) and the interior point follows.
    assert!((at(&even, 0) - DVec3::new(0.3, 0.3, 0.)).length() < 1e-9);
    assert!((at(&even, 1) - DVec3::new(1.1, 0.3, 0.)).length() < 1e-9);
    assert!((at(&even, 2) - DVec3::new(1.1, 1.1, 0.)).length() < 1e-6);
    // Every cap quad is the same size.
    let sizes: Vec<f64> = (0..9).map(|f| area_z(&cap_points(&even, f))).collect();
    assert!(sizes.iter().all(|s| (s - 0.64).abs() < 1e-6), "{sizes:?}");
}

#[test]
fn keep_cap_uvs_leaves_the_caps_uvs_exact() {
    let g = grid();
    let uv = |m: &Mesh, f: usize| m.uv_maps["Texture"][f].clone();
    // Bevel, grouped and not.
    for grouped in [false, true] {
        let off = bevel(&g, &[4], 0.2, grouped, &BevelOptions::default());
        assert_ne!(
            uv(&off, 4),
            uv(&g, 4),
            "Connected UVs inset the cap by default"
        );
        let p = BevelOptions {
            keep_cap_uvs: true,
            ..Default::default()
        };
        let on = bevel(&g, &[4], 0.2, grouped, &p);
        assert_eq!(uv(&on, 4), uv(&g, 4));
        // The walls still have UVs, and their cap-side corners moved.
        for f in g.faces.len()..on.faces.len() {
            assert!(uv(&on, f).iter().all(Option::is_some));
        }
        assert_eq!(on.validate(), Ok(()));
    }
    // Inset.
    let options = InsetOptions {
        keep_cap_uvs: true,
        ..Default::default()
    };
    let off = inset_geometry(&g, &[4], 0.2).unwrap().mesh;
    assert_ne!(uv(&off, 4), uv(&g, 4));
    let on = inset_geometry_with(&g, &[4], 0.2, options).unwrap().mesh;
    assert_eq!(uv(&on, 4), uv(&g, 4));
    let wall = uv(&on, g.faces.len());
    assert!(wall.iter().all(Option::is_some));
    assert_ne!(
        wall[2], wall[1],
        "the wall's inner corners have their own UVs"
    );
}

#[test]
fn new_points_carry_weights_colours_and_morphs() {
    let mut m = one_quad();
    let a = &mut m.attributes;
    a.relative_morphs.insert(
        "Bulge".into(),
        (0..4).map(|v| (v, DVec3::Z * v as f64)).collect(),
    );
    a.point_weights.insert(
        "Weight".into(),
        (0..4).map(|v| (v, v as f64 / 4.)).collect(),
    );
    a.corner_weights.insert(
        "Corner".into(),
        vec![vec![Some(0.1), Some(0.2), Some(0.3), Some(0.4)]],
    );
    let check = |out: &Mesh| {
        let at = &out.attributes;
        for &v in &out.faces[0].vertices {
            let origin = (0..4u32)
                .min_by(|&a, &b| {
                    let d = |x: u32| out.vertices[x as usize].distance(out.vertices[v as usize]);
                    d(a).total_cmp(&d(b))
                })
                .unwrap();
            assert_eq!(at.relative_morphs["Bulge"][&v], DVec3::Z * origin as f64);
            assert_eq!(at.point_weights["Weight"][&v], origin as f64 / 4.);
        }
        assert_eq!(
            at.corner_weights["Corner"][0],
            vec![Some(0.1), Some(0.2), Some(0.3), Some(0.4)]
        );
        assert_eq!(out.validate(), Ok(()));
    };
    check(&inset_geometry(&m, &[0], 0.2).unwrap().mesh);
    check(&bevel(&m, &[0], 0.2, false, &BevelOptions::default()));
}

fn subd_editor() -> Editor {
    let mut m = grid();
    m.attributes.representations = vec![PolygonRepresentation::Subdivs; m.faces.len()];
    let mut e = Editor::default();
    e.scene.new_mesh().unwrap();
    e.scene.active_mut().unwrap().mesh = m.into();
    e.scene.selection.mode = Mode::Polygon;
    e.scene.selection.faces = vec![4];
    e.synchronize_identities().unwrap();
    e
}

fn mesh(e: &Editor) -> Mesh {
    e.scene.active().unwrap().mesh.to_owned_value()
}

#[test]
fn subd_walls_and_exact_undo_redo_on_a_uv_mapped_subd_mesh() {
    for tool in [Tool::Inset, Tool::Bevel] {
        let mut e = subd_editor();
        let before = mesh(&e);
        e.dispatch(Command::Activate(tool)).unwrap();
        let mut p = e.live.as_ref().unwrap().params.clone();
        p.inset = 0.2;
        if tool == Tool::Bevel {
            p.shift = 0.3;
            p.bevel.keep_cap_uvs = true;
            p.bevel.even_interior = true;
            p.bevel.stop_at_fold = true;
        } else {
            p.inset_options.keep_cap_uvs = true;
            p.inset_options.stop_at_fold = true;
        }
        e.dispatch(Command::Preview(p)).unwrap();
        e.dispatch(Command::Drop).unwrap();
        let after = mesh(&e);
        assert!(
            after.faces.len() > before.faces.len(),
            "{tool:?} built walls"
        );
        // Walls match their polygon's type: SubD stays SubD.
        assert!(
            after
                .attributes
                .representations
                .iter()
                .all(|&r| r == PolygonRepresentation::Subdivs),
            "{tool:?}"
        );
        assert_eq!(
            after.uv_maps["Texture"][4], before.uv_maps["Texture"][4],
            "{tool:?} kept the cap's UVs"
        );
        e.dispatch(Command::Undo).unwrap();
        assert!(mesh(&e) == before, "{tool:?} undo");
        e.dispatch(Command::Redo).unwrap();
        assert!(mesh(&e) == after, "{tool:?} redo");
    }
}

#[test]
fn bevel_distances_are_world_units_on_a_scaled_item() {
    let mut e = subd_editor();
    let id = e.scene.active().unwrap().id;
    e.scene.item_mut(id).unwrap().transform.scale = DVec3::splat(2.);
    e.dispatch(Command::Activate(Tool::Bevel)).unwrap();
    let mut p = e.live.as_ref().unwrap().params.clone();
    p.inset = 0.2;
    e.dispatch(Command::Preview(p)).unwrap();
    e.dispatch(Command::Drop).unwrap();
    // 0.2 m in the world is 0.1 in the item's own units.
    let m = mesh(&e);
    let corner = m.vertices[m.faces[4].vertices[0] as usize];
    assert!(
        (corner - DVec3::new(1.1, 1.1, 0.)).length() < 1e-9,
        "{corner}"
    );
}
