//! The viewport grid is depth-tested against the scene: it shows in front
//! of geometry lying behind or below its plane, and stays hidden behind
//! geometry in front of it, in Perspective and in the orthographic views.
use super::*;
use crate::camera::View;

/// A 40 m square floor at height `y`, filling the view.
fn floor(y: f64) -> operandi_core::Mesh {
    let mut mesh = operandi_core::Mesh::default();
    for (x, z) in [(-20., -20.), (20., -20.), (20., 20.), (-20., 20.)] {
        mesh.vertices.push(DVec3::new(x, y, z));
    }
    mesh.faces.push(Face {
        vertices: vec![0, 3, 2, 1],
        uv: None,
        material: String::new(),
        smooth: false,
        part: None,
        material_label: None,
    });
    mesh
}

/// The first view's picture with the floor at `y`, the grid shown or not.
fn picture(view: View, y: f64, grid: bool) -> Option<Vec<u8>> {
    let state = super::performance_tests::headless_gpu()?;
    let ctx = Context::default();
    theme::install(&ctx);
    let mut a = Operandi::empty();
    a.viewport_gpu = Some(gpu::ViewportGpu::new(state));
    a.run(Command::NewMesh);
    a.editor.scene.active_mut().unwrap().mesh = floor(y).into();
    a.display.settings.drawing.grid = grid;
    a.display.settings.drawing.grid_axes = grid;
    a.camera.view = view;
    a.camera.target = DVec3::ZERO;
    a.camera.distance = 6.;
    if view == View::Perspective {
        a.camera.yaw = 0.3;
        a.camera.pitch = 0.9;
    }
    for _ in 0..3 {
        super::tests::draw(&mut a, &ctx, vec2(600., 400.), vec![]);
    }
    let views = a.viewport_gpu.as_ref().unwrap().read_views();
    views.into_iter().next().map(|(_, _, rgba)| rgba)
}

/// How many pixels differ between two pictures.
fn changed(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4)
        .zip(b.chunks(4))
        .filter(|(p, q)| p.iter().zip(*q).any(|(x, y)| x.abs_diff(*y) > 2))
        .count()
}

#[test]
fn grid_shows_over_geometry_below_it_and_not_through_geometry_above() {
    for view in [View::Perspective, View::Top] {
        let Some(below_off) = picture(view, -1., false) else {
            eprintln!("skipped: no graphics device");
            return;
        };
        let below_on = picture(view, -1., true).unwrap();
        let pixels = below_on.len() / 4;
        // The floor fills the view; the grid's lines cross it.
        assert!(
            changed(&below_off, &below_on) > pixels / 200,
            "{view:?}: the grid should draw over the floor below it"
        );
        let above_off = picture(view, 1., false).unwrap();
        let above_on = picture(view, 1., true).unwrap();
        assert_eq!(
            changed(&above_off, &above_on),
            0,
            "{view:?}: the floor above the grid should hide it"
        );
    }
}
