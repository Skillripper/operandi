//! Mini Properties' size (MP-03, issue #8).
use super::tests::{box_app, draw};
use super::*;

/// Issue #8: with an item selected and no tool, Mini Properties showed
/// only about two rows (Name and part of Item Transform) in a pop-over too
/// short to read. It's as tall as its rows, up to `MAX_HEIGHT`, wherever
/// it opens on screen.
#[test]
fn mini_properties_is_tall_enough_for_the_item_rows() {
    for at in [pos2(700., 300.), pos2(700., 880.), pos2(400., 150.)] {
        let ctx = Context::default();
        theme::install(&ctx);
        let mut a = box_app();
        a.run(Command::Mode(Mode::Item));
        assert!(a.editor.scene.active().is_some());
        a.mini_properties = Some(mini_properties::MiniProperties {
            at,
            moved: Vec2::ZERO,
        });
        let size = vec2(1265., 1016.);
        // Opened with nothing selected (one short line), then an item is
        // picked: the pop-over has to grow to its rows.
        let item = a.editor.scene.active().unwrap().id;
        a.editor.scene.select_item(None);
        for _ in 0..2 {
            let _ = draw(&mut a, &ctx, size, vec![]);
        }
        a.editor.scene.select_item(Some(item));
        for _ in 0..4 {
            let _ = draw(&mut a, &ctx, size, vec![]);
        }
        let rect = ctx
            .memory(|m| m.area_rect(Id::new("mini-properties")))
            .expect("Mini Properties drew");
        // Name plus the Item Transform rows: well over 64 points.
        assert!(
            rect.height() > 250.,
            "Mini Properties at {at:?} is only {} tall",
            rect.height()
        );
        assert!(
            rect.top() >= 0. && rect.bottom() <= size.y + 0.5,
            "Mini Properties at {at:?} is off screen: {rect:?}"
        );
    }
}
