//! Bridge as a Mesh Stack op (`edge.bridge.item`): the same engine as the
//! direct tool, on the op's edges or, with no Selection, the mesh's border.
use crate::mesh_stack_tools::{add, cube, ok, with_base};
use operandi_core::mesh_stack::OpStatus;
use operandi_core::{Mesh, persistence};
use serde_json::json;

/// Only a cube's top and bottom polygons: two loose squares facing apart.
fn caps() -> Mesh {
    let c = cube();
    let caps: Vec<u32> = (0..c.faces.len() as u32)
        .filter(|&f| {
            let v = &c.faces[f as usize].vertices;
            let y = c.vertices[v[0] as usize].y;
            v.iter()
                .all(|&p| (c.vertices[p as usize].y - y).abs() < 1e-9)
        })
        .collect();
    assert_eq!(caps.len(), 2);
    let sides: Vec<u32> = (0..c.faces.len() as u32)
        .filter(|f| !caps.contains(f))
        .collect();
    operandi_core::polygon_remove::delete_geometry(&c, &sides, &[]).unwrap()
}

#[test]
fn bridge_joins_the_two_border_loops() {
    let (mut e, id) = with_base(caps());
    add(&mut e, id, "edge.bridge.item", json!({}));
    // Four new polygons span the two squares' borders: a closed box.
    assert_eq!(ok(&e, id).faces.len(), 2 + 4);
    // More segments, more rows.
    let (mut e, id) = with_base(caps());
    add(&mut e, id, "edge.bridge.item", json!({ "segments": 3 }));
    assert_eq!(ok(&e, id).faces.len(), 2 + 4 * 3);
}

#[test]
fn bridge_with_nothing_to_pair_fails_only_its_row() {
    // A closed cube has no border: the op fails, its input passes through.
    let (mut e, id) = with_base(cube());
    add(&mut e, id, "edge.bridge.item", json!({}));
    let r = e
        .mesh_stack
        .current(&e.scene, e.scene.item(id).unwrap())
        .unwrap();
    assert!(
        matches!(r.statuses[0], OpStatus::Failed(_)),
        "{:?}",
        r.statuses
    );
    assert_eq!(*r.mesh, cube());
}

#[test]
fn bridge_values_save_and_reopen() {
    let (mut e, id) = with_base(caps());
    add(
        &mut e,
        id,
        "edge.bridge.item",
        json!({ "segments": 2, "mode": 0, "twist": 1.0 }),
    );
    let before = ok(&e, id);
    let e2 = operandi_core::Editor {
        scene: persistence::decode(&persistence::encode(&e.scene).unwrap()).unwrap(),
        ..Default::default()
    };
    assert_eq!(ok(&e2, id), before);
}
