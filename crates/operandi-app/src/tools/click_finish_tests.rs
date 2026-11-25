//! UIC-78 in the app: a plain click in the viewport applies a tool that
//! waits for Apply and the tool stays live on the result (values reshape
//! it, one undo step); a drag still drives it, a click never selects, and
//! a click that can't apply changes nothing and says why.
use super::tests::{draw, pointer};
use super::*;

const SIZE: Vec2 = vec2(1440., 1600.);

/// Two facing quads selected in Polygon mode, Bridge picked, seen from the
/// front, as the Bridge panel tests build them.
fn bridge_app(ctx: &Context) -> Operandi {
    theme::install(ctx);
    let mut a = Operandi::empty();
    a.editor.dispatch(Command::NewMesh).unwrap();
    let m = &mut a.editor.scene.active_mut().unwrap().mesh;
    m.vertices = vec![
        DVec3::new(-2., -1., 0.),
        DVec3::new(3., -1., 0.),
        DVec3::new(2., 2., 0.),
        DVec3::new(-1., 1., 0.),
        DVec3::new(-1.5, -0.5, 4.),
        DVec3::new(2., -0.5, 4.),
        DVec3::new(1.5, 1.5, 4.),
        DVec3::new(-0.5, 1., 4.),
    ];
    for vertices in [vec![0, 1, 2, 3], vec![4, 7, 6, 5]] {
        m.faces.push(operandi_core::Face {
            vertices,
            uv: None,
            material: "Default".into(),
            material_label: None,
            part: None,
            smooth: false,
        });
    }
    a.editor.scene.selection.mode = Mode::Polygon;
    a.editor.scene.selection.faces = vec![0, 1];
    a.camera.view = View::Front;
    a.editor.undo.clear();
    a.execute("Bridge");
    assert!(a.editor.bridge.is_some(), "{}", a.status);
    for _ in 0..3 {
        draw(&mut a, ctx, SIZE, vec![]);
    }
    a
}

/// Press at `from`, move by `by` in four steps, release.
fn drag(a: &mut Operandi, ctx: &Context, from: Pos2, by: Vec2, modifiers: Modifiers) {
    draw(a, ctx, SIZE, vec![Event::PointerMoved(from)]);
    draw(a, ctx, SIZE, vec![pointer(from, true, modifiers)]);
    for step in 1..=4 {
        draw(
            a,
            ctx,
            SIZE,
            vec![Event::PointerMoved(from + by * (step as f32 / 4.))],
        );
    }
    draw(a, ctx, SIZE, vec![pointer(from + by, false, modifiers)]);
}

fn click(a: &mut Operandi, ctx: &Context) {
    let at = a.viewport_rect.center();
    drag(a, ctx, at, Vec2::ZERO, Modifiers::NONE);
}

fn faces(a: &Operandi) -> usize {
    a.editor.scene.active().unwrap().mesh.faces.len()
}

/// The click bridges with the panel's values as one undo step, Bridge
/// stays on, and the click on the mesh selected nothing.
#[test]
fn a_click_applies_bridge_and_keeps_it_live() {
    let ctx = Context::default();
    let mut a = bridge_app(&ctx);
    let before = a.editor.scene.clone();
    click(&mut a, &ctx);
    assert!(a.editor.bridge.is_some(), "{}", a.status);
    assert!(a.editor.bridge_result_live(), "{}", a.status);
    assert_ne!(a.editor.scene, before, "{}", a.status);
    assert!(faces(&a) > 2);
    assert_eq!(a.editor.undo.len(), 1);
    assert_eq!(a.status, "Bridge applied · change a value to reshape it");
    // A second click makes no second bridge.
    let applied = a.editor.scene.clone();
    click(&mut a, &ctx);
    assert_eq!(a.editor.scene, applied);
    assert_eq!(a.editor.undo.len(), 1);
}

/// After the click, a drag and a typed value reshape the same bridge in
/// one undo step; Undo takes it back to the two quads.
#[test]
fn values_reshape_the_live_bridge_in_one_step() {
    let ctx = Context::default();
    let mut a = bridge_app(&ctx);
    let before = a.editor.scene.clone();
    click(&mut a, &ctx);
    let at = a.viewport_rect.center();
    drag(&mut a, &ctx, at, vec2(40., 0.), Modifiers::NONE);
    assert_eq!(a.editor.bridge.as_ref().unwrap().requested.segments, 3);
    // Four sides, three segments: twelve bridge polygons.
    assert_eq!(faces(&a), 12, "{}", a.status);
    a.execute("bridge segments 1");
    assert_eq!(faces(&a), 4, "{}", a.status);
    assert_eq!(a.editor.undo.len(), 1);
    assert!(a.editor.bridge_result_live());
    a.execute("undo");
    assert_eq!(
        a.editor.scene.active().unwrap().mesh,
        before.active().unwrap().mesh
    );
    assert!(!a.editor.bridge_result_live());
}

/// A drag before any click sets Segments and changes nothing; the click
/// then bridges with them.
#[test]
fn a_drag_before_the_click_only_sets_segments() {
    let ctx = Context::default();
    let mut a = bridge_app(&ctx);
    let before = a.editor.scene.clone();
    let at = a.viewport_rect.center();
    drag(&mut a, &ctx, at, vec2(40., 0.), Modifiers::NONE);
    assert_eq!(a.editor.bridge.as_ref().unwrap().requested.segments, 3);
    assert_eq!(a.editor.scene, before);
    assert!(a.editor.undo.is_empty());
    click(&mut a, &ctx);
    assert_eq!(faces(&a), 12);
    assert_eq!(a.editor.undo.len(), 1);
}

/// Q keeps the live bridge and ends the tool; Cancel takes the live
/// bridge back with its undo step.
#[test]
fn drop_keeps_and_cancel_removes_the_live_bridge() {
    let ctx = Context::default();
    let mut a = bridge_app(&ctx);
    let before = a.editor.scene.clone();
    click(&mut a, &ctx);
    let applied = a.editor.scene.clone();
    a.execute("tool.drop");
    assert!(a.editor.bridge.is_none());
    assert_eq!(a.editor.scene, applied);
    assert_eq!(a.editor.undo.len(), 1);

    let mut b = bridge_app(&ctx);
    click(&mut b, &ctx);
    b.run(Command::Cancel);
    assert!(b.editor.bridge.is_none());
    assert_eq!(b.editor.scene, before);
    assert!(b.editor.undo.is_empty());
}

/// With only one polygon selected the click changes nothing; Bridge stays
/// on and the status line says why.
#[test]
fn a_click_that_cannot_bridge_keeps_the_tool() {
    let ctx = Context::default();
    let mut a = bridge_app(&ctx);
    a.editor.scene.selection.faces = vec![0];
    let before = a.editor.scene.clone();
    click(&mut a, &ctx);
    assert!(a.editor.bridge.is_some());
    assert_eq!(a.editor.scene, before);
    assert!(a.editor.undo.is_empty());
    assert!(a.status.starts_with("Bridge: "), "{}", a.status);
}

/// A Shift (or other modified) click neither applies nor selects.
#[test]
fn a_modified_click_does_not_apply() {
    let ctx = Context::default();
    let mut a = bridge_app(&ctx);
    let before = a.editor.scene.clone();
    let at = a.viewport_rect.center();
    drag(&mut a, &ctx, at, Vec2::ZERO, Modifiers::SHIFT);
    assert!(a.editor.bridge.is_some());
    assert_eq!(a.editor.scene, before);
    assert!(a.editor.undo.is_empty());
}
