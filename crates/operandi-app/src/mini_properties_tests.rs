//! Mini Properties' size (MP-03, UIC-66, issue #8).
use super::tests::{box_app, draw, pointer};
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
            top_left: None,
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

/// UIC-66 (the maintainer, 2026-11-24): its bottom-right corner drags to
/// resize it; the top-left corner stays put, and the next one opens at the
/// new size.
#[test]
fn mini_properties_corner_drags_to_resize() {
    let ctx = Context::default();
    theme::install(&ctx);
    let mut a = box_app();
    a.run(Command::Mode(Mode::Item));
    let size = vec2(1265., 1016.);
    a.mini_properties = Some(mini_properties::MiniProperties {
        at: pos2(700., 700.),
        moved: Vec2::ZERO,
        top_left: None,
    });
    let rect = |ctx: &Context| {
        ctx.memory(|m| m.area_rect(Id::new("mini-properties")))
            .unwrap()
    };
    for _ in 0..3 {
        let _ = draw(&mut a, &ctx, size, vec![]);
    }
    let before = rect(&ctx);
    let start = before.max - vec2(4., 4.);
    let end = start + vec2(120., -150.);
    let mut events = vec![
        Event::PointerMoved(start),
        pointer(start, true, Modifiers::NONE),
    ];
    for k in 1..=5 {
        events.push(Event::PointerMoved(start + (end - start) * k as f32 / 5.));
    }
    for e in events {
        let _ = draw(&mut a, &ctx, size, vec![e]);
    }
    let _ = draw(
        &mut a,
        &ctx,
        size,
        vec![pointer(end, false, Modifiers::NONE)],
    );
    for _ in 0..2 {
        let _ = draw(&mut a, &ctx, size, vec![]);
    }
    let after = rect(&ctx);
    assert!(
        (after.min - before.min).length() < 1.,
        "{before:?} -> {after:?}"
    );
    assert!(
        (after.width() - before.width() - 120.).abs() < 4.,
        "{before:?} -> {after:?}"
    );
    assert!(
        (after.height() - before.height() + 150.).abs() < 4.,
        "{before:?} -> {after:?}"
    );
    // Closed and opened again: the same size.
    let wanted = after.size();
    a.mini_properties = None;
    let _ = draw(&mut a, &ctx, size, vec![]);
    a.mini_properties = Some(mini_properties::MiniProperties {
        at: pos2(900., 800.),
        moved: Vec2::ZERO,
        top_left: None,
    });
    for _ in 0..3 {
        let _ = draw(&mut a, &ctx, size, vec![]);
    }
    assert!((rect(&ctx).size() - wanted).length() < 2.);
}
