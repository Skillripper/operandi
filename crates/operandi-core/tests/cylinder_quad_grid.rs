//! Cylinder Quad Grid caps (GitHub #17): the same grid as the Cone's base,
//! all quads for even Sides, one triangle for odd, both ends facing out.
use glam::DVec3;
use operandi_core::*;

fn cylinder(sides: u32, caps: [CylinderCap; 2]) -> Mesh {
    cylinder_geometry_with(
        DVec3::ZERO,
        RadialParameters {
            radius: DVec3::new(0.5, 0.75, 0.5),
            sides,
            segments: 1,
        },
        caps,
        1,
        true,
    )
    .unwrap()
}

#[test]
fn even_sides_make_all_quad_caps_that_face_out() {
    for sides in [8, 12, 24, 32] {
        let m = cylinder(sides, [CylinderCap::Quads; 2]);
        m.validate().unwrap();
        assert!(
            m.faces.iter().all(|f| f.vertices.len() == 4),
            "{sides}: every polygon is a quad"
        );
        // Every cap polygon faces away from the middle along Y.
        for f in m.faces.iter().skip(sides as usize) {
            let p: Vec<DVec3> = f.vertices.iter().map(|&v| m.vertices[v as usize]).collect();
            let normal = (p[1] - p[0]).cross(p[2] - p[0]);
            assert!(normal.y * p[0].y > 0., "{sides}: a cap polygon faces in");
        }
    }
}

#[test]
fn the_cap_matches_the_cones_base() {
    for sides in [5, 8, 12, 24] {
        let cone = cone_geometry(&ConeParameters {
            sides,
            bottom_cap: ConeCap::QuadGrid,
            make_uvs: false,
            ..Default::default()
        })
        .unwrap();
        let cyl = cylinder(sides, [CylinderCap::Quads, CylinderCap::None]);
        let count = |m: &Mesh, n: usize| m.faces.iter().filter(|f| f.vertices.len() == n).count();
        // Cone: n apex triangles plus the cap; Cylinder: n wall quads plus the cap.
        assert_eq!(count(&cyl, 3), count(&cone, 3) - sides as usize, "{sides}");
        assert_eq!(count(&cyl, 4) - sides as usize, count(&cone, 4), "{sides}");
    }
}

#[test]
fn odd_sides_leave_one_triangle_per_cap_and_big_grids_are_refused() {
    let m = cylinder(7, [CylinderCap::Quads; 2]);
    assert_eq!(m.faces.iter().filter(|f| f.vertices.len() == 3).count(), 2);
    assert!(
        cylinder_geometry_with(
            DVec3::ZERO,
            RadialParameters {
                radius: DVec3::ONE,
                sides: 4096,
                segments: 1,
            },
            [CylinderCap::Quads; 2],
            1,
            false,
        )
        .is_err()
    );
}
