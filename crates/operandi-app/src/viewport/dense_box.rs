//! Dense meshes drawn as boxes: when a mesh is too dense for its wireframe,
//! and the box or box-corner strokes drawn instead.
use super::*;

/// The 12 edges of `mesh`'s bounding box through `matrix`, as wire strokes
/// (Bounding Box Threshold), in `color` (the item's plain wire: boxes
/// draw in the shared overlay, so the inactive colour is baked here).
/// Only heavy meshes get the dense-mesh box corners (Operandi's choice,
/// provisional): a crowd of small objects, each a few points across, would
/// otherwise all be boxed.
pub(super) const DENSE_BOX_POLYGONS: usize = 10_000;

/// Whether a mesh's polygons draw smaller on screen than Fade Wireframe on
/// Small Polygons keeps full wire for: its box's screen area shared out
/// among its polygons, as squares. Not when the box reaches behind the
/// eye.
pub(super) fn too_dense_for_wire(
    projector: &crate::camera::Projector,
    lo: DVec3,
    hi: DVec3,
    matrix: glam::DMat4,
    polygons: usize,
) -> bool {
    if polygons == 0 {
        return false;
    }
    let mut screen = Rect::NOTHING;
    for i in 0..8 {
        let corner = DVec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        );
        let Some((p, _)) = projector.project(matrix.transform_point3(corner)) else {
            return false;
        };
        screen.extend_with(p);
    }
    let side = (f64::from(screen.area()) / polygons as f64).sqrt();
    side < f64::from(crate::gpu::WIRE_FADE_FULL)
}

/// The corners of a box (`lo`..`hi` placed by `matrix`): a fifth of each
/// edge from each end, in `color`.
pub(super) fn box_corner_strokes(
    lo: DVec3,
    hi: DVec3,
    matrix: glam::DMat4,
    color: Color32,
    out: &mut Vec<crate::gpu_scene::EdgeVertex>,
) {
    let corner = |i: usize| {
        matrix.transform_point3(DVec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        ))
    };
    for i in 0..8 {
        for bit in [1, 2, 4] {
            // Each box edge once, from its lower-numbered corner.
            if i & bit != 0 {
                continue;
            }
            let (a, b) = (corner(i), corner(i | bit));
            // Width 1 point (not the plain wire's 0), so the overlay keeps
            // the colour instead of taking the view's wire colour.
            crate::gpu_scene::stroke_width(out, a, a.lerp(b, 0.2), 1., color);
            crate::gpu_scene::stroke_width(out, b, b.lerp(a, 0.2), 1., color);
        }
    }
}

pub(super) fn box_strokes(
    mesh: &operandi_core::Mesh,
    matrix: glam::DMat4,
    color: Color32,
    out: &mut Vec<crate::gpu_scene::EdgeVertex>,
) {
    let Some(first) = mesh.vertices.first() else {
        return;
    };
    let (lo, hi) = mesh
        .vertices
        .iter()
        .fold((*first, *first), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
    let corner = |i: usize| {
        matrix.transform_point3(DVec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        ))
    };
    for (a, b) in [
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (0, 2),
        (1, 3),
        (4, 6),
        (5, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        crate::gpu_scene::stroke(out, corner(a), corner(b), false, color);
    }
}

#[cfg(test)]
mod dense_mesh_box_tests {
    use super::*;

    /// A box's 8 corners, each with its 3 edges' first fifths: 24 strokes
    /// of 6 corners, in the given colour, none longer than a fifth.
    #[test]
    fn box_corners_are_a_fifth_of_each_edge() {
        let mut out = Vec::new();
        let colour = Color32::from_rgb(1, 2, 3);
        box_corner_strokes(
            DVec3::ZERO,
            DVec3::splat(5.),
            glam::DMat4::IDENTITY,
            colour,
            &mut out,
        );
        assert_eq!(out.len(), 24 * 6);
        assert!(out.iter().all(|v| v.color == colour && v.width == 1.));
        for v in &out {
            let len = (glam::Vec3::from(v.a) - glam::Vec3::from(v.b)).length();
            assert!((len - 1.).abs() < 1e-5, "{len}");
        }
    }

    /// Dense is about polygons per screen area, not polygon count alone.
    #[test]
    fn density_is_screen_area_per_polygon() {
        let camera = crate::camera::Camera::default();
        let r = Rect::from_min_size(Pos2::ZERO, vec2(800., 600.));
        let p = camera.projector(r);
        let (lo, hi) = (DVec3::splat(-1.), DVec3::splat(1.));
        assert!(!too_dense_for_wire(&p, lo, hi, glam::DMat4::IDENTITY, 100));
        assert!(too_dense_for_wire(
            &p,
            lo,
            hi,
            glam::DMat4::IDENTITY,
            10_000_000
        ));
        assert!(!too_dense_for_wire(&p, lo, hi, glam::DMat4::IDENTITY, 0));
    }
}
