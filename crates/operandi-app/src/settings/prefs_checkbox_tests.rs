//! Every checkbox in Preferences shows its new state after a click, and an
//! Experimental switch forced by its environment variable shows the forced
//! state, greyed with the reason (prefs-checkbox-refresh). The Python box
//! once stayed unticked: the row's group diff paired leaves by position,
//! and a field the JSON left out shifted them, so the click was undone.
use super::tests::draw;
use super::*;
use crate::preferences::Page;
use egui::accesskit::{self, Role, Toggled};

const SIZE: Vec2 = vec2(1440., 1400.);

fn context() -> Context {
    let ctx = Context::default();
    theme::install(&ctx);
    ctx.enable_accesskit();
    ctx
}

fn frame(a: &mut Operandi, ctx: &Context, events: Vec<Event>) -> Vec<accesskit::Node> {
    draw(a, ctx, SIZE, events)
        .platform_output
        .accesskit_update
        .expect("AccessKit enabled")
        .nodes
        .into_iter()
        .map(|(_, n)| n)
        .collect()
}

fn center(n: &accesskit::Node) -> Pos2 {
    let b = n.bounds().expect("bounds");
    pos2(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32)
}

fn pointer(at: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn click(a: &mut Operandi, ctx: &Context, at: Pos2) {
    frame(a, ctx, vec![Event::PointerMoved(at)]);
    frame(a, ctx, vec![pointer(at, true)]);
    frame(a, ctx, vec![pointer(at, false)]);
    frame(a, ctx, vec![Event::PointerMoved(pos2(1., 1.))]);
}

/// The Preferences window's checkboxes: (label, enabled, toggled, centre).
fn boxes(nodes: &[accesskit::Node], window: Rect) -> Vec<(String, bool, Option<Toggled>, Pos2)> {
    nodes
        .iter()
        .filter(|n| n.role() == Role::CheckBox && n.bounds().is_some())
        // Rows scrolled out of view still have nodes: keep them, so a
        // box's index stays the same as the page scrolls.
        .filter(|n| window.x_range().contains(center(n).x))
        .map(|n| {
            (
                n.label().unwrap_or_default().to_owned(),
                !n.is_disabled(),
                n.toggled(),
                center(n),
            )
        })
        .collect()
}

fn window(ctx: &Context) -> Rect {
    ctx.memory(|m| m.area_rect(Id::new("Preferences")))
        .expect("Preferences drawn")
}

fn page_boxes(a: &mut Operandi, ctx: &Context) -> Vec<(String, bool, Option<Toggled>, Pos2)> {
    for _ in 0..2 {
        frame(a, ctx, vec![]);
    }
    let nodes = frame(a, ctx, vec![]);
    boxes(&nodes, window(ctx))
}

/// The top of the window's Close button row; the page body ends above it.
fn close_top(nodes: &[accesskit::Node]) -> f32 {
    nodes
        .iter()
        .find(|n| n.role() == Role::Button && n.label() == Some("Close"))
        .and_then(|n| n.bounds())
        .map_or(f32::MAX, |b| b.y0 as f32)
}

/// Box `k` of the page, scrolled into view first when the page scrolls.
fn visible_box(
    a: &mut Operandi,
    ctx: &Context,
    k: usize,
) -> Option<(String, bool, Option<Toggled>, Pos2)> {
    let mut last = None;
    for _ in 0..200 {
        let nodes = frame(a, ctx, vec![]);
        let found = boxes(&nodes, window(ctx)).get(k).cloned()?;
        // The page body ends about 100 points above the Close row.
        let limit = close_top(&nodes) - 110.;
        // Scrolling is animated: wait until the row stops moving.
        let settled = last == Some(found.3);
        last = Some(found.3);
        if !settled {
            continue;
        }
        let top = window(ctx).top() + 50.;
        if (top..limit).contains(&found.3.y) {
            return Some(found);
        }
        let down = found.3.y >= limit;
        let over = pos2(found.3.x, window(ctx).center().y);
        frame(a, ctx, vec![Event::PointerMoved(over)]);
        frame(
            a,
            ctx,
            vec![Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: vec2(0., if down { -80. } else { 80. }),
                modifiers: Modifiers::NONE,
            }],
        );
    }
    None
}

fn flip(t: Option<Toggled>) -> Option<Toggled> {
    match t {
        Some(Toggled::True) => Some(Toggled::False),
        Some(Toggled::False) => Some(Toggled::True),
        other => other,
    }
}

#[test]
fn every_preference_checkbox_shows_its_new_state_after_a_click() {
    let ctx = context();
    let mut a = Operandi::empty();
    a.open_preferences();
    let mut clicked = 0;
    let mut broken = Vec::new();
    for page in Page::ALL {
        a.preferences_ui.page = page;
        let found = page_boxes(&mut a, &ctx);
        for (k, (label, enabled, was, _)) in found.iter().enumerate() {
            if !enabled {
                continue;
            }
            // Positions can move as rows grey or show; find it again.
            let Some((_, _, _, at)) = visible_box(&mut a, &ctx, k) else {
                broken.push(format!("{page:?} / {label}: can't scroll to it"));
                continue;
            };
            click(&mut a, &ctx, at);
            if !a.show_preferences {
                a.open_preferences();
                a.preferences_ui.page = page;
            }
            let now = page_boxes(&mut a, &ctx).get(k).map(|b| b.2);
            if now != Some(flip(*was)) {
                let close = close_top(&frame(&mut a, &ctx, vec![]));
                broken.push(format!(
                    "{page:?} / {label}: {was:?} -> {now:?} ({}; at {at:?}, close {close:?})",
                    a.status
                ));
                continue;
            }
            clicked += 1;
            // Back as it was.
            if let Some((_, _, _, at)) = visible_box(&mut a, &ctx, k) {
                click(&mut a, &ctx, at);
            }
            let back = page_boxes(&mut a, &ctx).get(k).map(|b| b.2);
            if back != Some(*was) {
                broken.push(format!("{page:?} / {label}: back {back:?}, want {was:?}"));
            }
        }
    }
    assert!(
        broken.is_empty(),
        "boxes that didn't redraw:\n{}",
        broken.join("\n")
    );
    assert!(clicked >= 20, "only {clicked} checkboxes clicked");
}

#[test]
fn python_box_ticks_and_saves_and_animation_has_none() {
    let ctx = context();
    let mut a = Operandi::empty();
    a.open_preferences();
    a.preferences_ui.page = Page::Application;
    let find = |a: &mut Operandi, label: &str| {
        page_boxes(a, &ctx)
            .into_iter()
            .find(|b| b.0 == label)
            .unwrap_or_else(|| panic!("{label} box"))
    };
    let label = "Python scripting (preview)";
    let before = a.undo_autosave.python_preview;
    let (_, _, shown, at) = find(&mut a, label);
    assert_eq!(shown, Some(before.into()));
    click(&mut a, &ctx, at);
    assert_eq!(
        a.undo_autosave.python_preview, !before,
        "the preference changed"
    );
    assert_eq!(find(&mut a, label).2, Some((!before).into()), "drawn");
    // The same path as the command (UIC-164).
    a.execute("pref.value application.python_preview ?");
    assert_eq!(a.status, (!before).to_string());
    // Animation is always on now (anim_switch.rs): no box for it.
    assert!(page_boxes(&mut a, &ctx).iter().all(|b| b.0 != "Animation"));
}

#[test]
fn a_switch_forced_by_its_environment_shows_the_forced_state_greyed() {
    let ctx = context();
    let mut a = Operandi::empty();
    a.open_preferences();
    a.preferences_ui.page = Page::Application;
    crate::python_switch::force_for_test(Some(true));
    let found = page_boxes(&mut a, &ctx);
    let python = found
        .iter()
        .find(|b| b.0 == "Python scripting (preview)")
        .unwrap();
    crate::python_switch::force_for_test(None);
    assert!(!a.undo_autosave.python_preview, "the preference is off");
    assert_eq!((python.1, python.2), (false, Some(Toggled::True)));
}

/// The cause, for every group: a field the JSON leaves out at one value
/// (`skip_serializing_if`) vanishes from `pref.value` and the row diff.
/// Flip each on/off leaf through `pref.value`: the group keeps every leaf.
#[test]
fn every_group_keeps_all_its_fields_when_a_switch_flips() {
    let mut a = Operandi::empty();
    let paths = |v: &serde_json::Value| -> Vec<String> {
        let mut out = Vec::new();
        crate::prefs_commands::leaves_of(v, &mut out);
        out.into_iter().map(|(p, _)| p).collect()
    };
    let mut flipped = 0;
    for group in crate::prefs_commands::GROUPS {
        let start = (group.get)(&a);
        let mut leaves = Vec::new();
        crate::prefs_commands::leaves_of(&start, &mut leaves);
        for (path, leaf) in leaves {
            let Some(on) = leaf.as_bool() else {
                continue;
            };
            let key = format!("{}.{path}", group.name);
            a.execute(&format!("pref.value {key} {}", !on));
            let now = (group.get)(&a);
            assert_eq!(paths(&now), paths(&start), "{key}: {}", a.status);
            a.execute(&format!("pref.value {key} {on}"));
            flipped += 1;
        }
    }
    assert!(flipped >= 20, "only {flipped} switches");
}
