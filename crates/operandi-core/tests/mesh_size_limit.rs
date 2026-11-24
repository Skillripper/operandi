//! Hidden-limits B1: one mesh size limit, 100 million points and polygons
//! (was a 5 million "development safety budget"), with a plain message.
use glam::DVec3;
use operandi_core::*;

#[test]
fn the_limit_is_checked_at_its_edges_with_a_plain_message() {
    let n = MAX_MESH_ELEMENTS;
    assert!(mesh_too_big(n - 1, n - 1).is_none());
    assert!(mesh_too_big(n, n).is_none());
    let e = mesh_too_big(0, n + 1).unwrap();
    assert!(
        e.contains("100.0 million polygons") && e.contains("at most 100.0 million"),
        "{e}"
    );
    let e = mesh_too_big(n + 1, 0).unwrap();
    assert!(e.contains("points"), "{e}");
}

#[test]
fn a_mesh_past_the_old_five_million_validates() {
    // 6 million points: refused before, fine now.
    let mesh = Mesh {
        vertices: vec![DVec3::ZERO; 6_000_000],
        ..Default::default()
    };
    mesh.validate().unwrap();
}
