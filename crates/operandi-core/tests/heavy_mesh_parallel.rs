//! Checks that run on every core for big meshes (heavy-mesh-speed) give
//! the answers they gave on one.
use glam::DVec3;
use operandi_core::*;

/// A `side` × `side` quad grid in XZ.
fn grid(side: u32) -> Mesh {
    let row = side + 1;
    let mut mesh = Mesh {
        vertices: (0..row * row)
            .map(|i| DVec3::new(f64::from(i % row), 0., f64::from(i / row)))
            .collect(),
        ..Default::default()
    };
    *mesh.faces = (0..side * side)
        .map(|k| {
            let (x, y) = (k % side, k / side);
            Face {
                vertices: vec![
                    y * row + x,
                    (y + 1) * row + x,
                    (y + 1) * row + x + 1,
                    y * row + x + 1,
                ],
                uv: None,
                material: String::new(),
                smooth: false,
                part: None,
                material_label: None,
            }
        })
        .collect();
    mesh
}

#[test]
fn a_big_mesh_names_its_first_bad_polygon_as_before() {
    let mut mesh = grid(200); // 40,000 polygons, checked on every core
    assert_eq!(mesh.validate(), Ok(()));
    // Two bad polygons: the error names the first one.
    let late = mesh.faces[35_000].vertices.clone();
    mesh.faces[35_000].vertices = vec![late[0], late[1], 999_999];
    let early = mesh.faces[30_000].vertices.clone();
    mesh.faces[30_000].vertices = vec![early[0], early[1], early[0]];
    assert_eq!(mesh.validate(), Err("Invalid face 30000".to_string()));
    // A flattened polygon is degenerate.
    let mut flat = grid(200);
    let corners = flat.faces[33_333].vertices.clone();
    for &v in &corners {
        flat.vertices[v as usize] = DVec3::splat(5.);
    }
    assert!(flat.validate().unwrap_err().starts_with("Degenerate face"));
    // A point out of the coordinate budget.
    let mut far = grid(200);
    far.vertices[20_000].x = f64::NAN;
    assert_eq!(far.validate(), Err("Invalid mesh coordinates".to_string()));
}
