//! No tooltip shows a raw command line ("poly.bevel", "operandi.paint.tip
//! kind:text"): a button without a label or tooltip of its own names its
//! command in words.
use super::*;

/// A word like `poly.bevel` or `operandi.paint.tip`: lower-case words
/// joined by dots, the way command lines are written.
fn looks_like_a_command(tip: &str) -> Option<&str> {
    tip.split(|c: char| c.is_whitespace() || matches!(c, '·' | ',' | '(' | ')' | ':'))
        .find(|w| {
            let w = w.trim_matches(|c: char| !c.is_alphanumeric());
            let parts: Vec<&str> = w.split('.').collect();
            parts.len() >= 2
                && parts.iter().all(|p| {
                    !p.is_empty()
                        && p.chars().next().is_some_and(|c| c.is_ascii_lowercase())
                        && p.chars().all(|c| c.is_ascii_alphanumeric())
                })
        })
}

#[test]
fn no_form_button_tooltip_shows_a_raw_command() {
    let a = Operandi::empty();
    let bad: Vec<String> = a
        .every_button_tooltip_for_test()
        .into_iter()
        // Menus show no tooltips (their items are native menu items).
        .filter(|(form, _, _)| !form.starts_with("menu."))
        .filter_map(|(form, line, tip)| {
            looks_like_a_command(&tip).map(|w| format!("{form}: {line} -> {tip:?} ({w})"))
        })
        .collect();
    assert!(
        bad.is_empty(),
        "{} tooltips:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// Preferences rows (and every other form row) show their hover text
/// without spec codes or command lines.
#[test]
fn a_preferences_row_tooltip_drops_spec_codes_and_commands() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    ctx.style_mut(|s| {
        s.interaction.tooltip_delay = 0.;
        s.interaction.show_tooltips_only_when_still = false;
    });
    let tip = "Time samples for every frame (TL-83) · poly.align";
    let mut label = None;
    let mut texts: Vec<String> = Vec::new();
    for _ in 0..4 {
        let events = label
            .map(|r: Rect| vec![Event::PointerMoved(r.right_center() - vec2(8., 0.))])
            .unwrap_or_default();
        let out = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(600., 400.))),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let (rects, _) = crate::form::rows::frame(ui, Some("Animation"), tip);
                    label = Some(rects.label);
                });
            },
        );
        texts = out
            .platform_output
            .accesskit_update
            .expect("AccessKit enabled")
            .nodes
            .into_iter()
            .filter_map(|(_, n)| n.label().or(n.value()).map(str::to_owned))
            .collect();
    }
    assert!(
        texts.iter().any(|t| t == "Time samples for every frame"),
        "{texts:?}"
    );
    assert!(
        !texts
            .iter()
            .any(|t| t.contains("TL-83") || t.contains("poly.align"))
    );
}
