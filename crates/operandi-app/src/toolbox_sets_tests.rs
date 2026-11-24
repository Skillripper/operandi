//! The seven toolbox sets (Jason, 2026-11-23): their tabs, one-form sets
//! (UV, Animation, Workbench) without a strip, the old Paint name, and
//! tabs that open their set.
use super::tests::{box_app, draw};
use super::*;
use crate::toolbox::{Category, ToolSet};

const SIZE: Vec2 = vec2(1440., 900.);

fn shows(a: &mut Operandi, ctx: &Context) -> egui::FullOutput {
    draw(a, ctx, SIZE, vec![]);
    draw(a, ctx, SIZE, vec![])
}

#[test]
fn the_bar_lists_seven_sets_in_order_and_paint_reads_as_brush() {
    let labels: Vec<_> = ToolSet::ALL.iter().map(|s| s.label()).collect();
    assert_eq!(
        labels,
        [
            "Model",
            "UV",
            "Brush",
            "Setup",
            "Animation",
            "Workbench",
            "Game"
        ]
    );
    assert_eq!(ToolSet::named("paint"), Some(ToolSet::Brush));
    let mut a = box_app();
    a.execute("operandi.toolbox.set paint");
    assert_eq!(a.layouts.tool_set, ToolSet::Brush, "{}", a.status);
    assert_eq!(a.category, Category::Sculpt, "Brush opens at Sculpt");
}

#[test]
fn every_tab_of_every_set_draws_its_form() {
    crate::anim_switch::set_for_test(true);
    let ctx = Context::default();
    let mut a = box_app();
    for &set in ToolSet::ALL {
        a.execute(&format!("operandi.toolbox.set {}", set.label()));
        assert_eq!(a.layouts.tool_set, set, "{}", a.status);
        let tabs = set.tabs();
        for &tab in &tabs {
            a.category = tab;
            shows(&mut a, &ctx);
            assert_eq!(a.layouts.tool_set, set, "{tab:?} stays in {set:?}");
            assert_eq!(a.shown_category(), Some(tab));
            // A strip only for sets with more than one tab.
            let strip = a.toolbox_rects.keys().any(|k| k.starts_with("Category "));
            assert_eq!(strip, tabs.len() > 1, "{set:?} {tab:?}");
            if tab != Category::Workbench {
                assert!(!a.form_rects.is_empty(), "{set:?} {tab:?} drew no buttons");
            }
        }
    }
}

#[test]
fn the_new_tabs_hold_their_planned_buttons() {
    crate::anim_switch::set_for_test(true);
    let ctx = Context::default();
    let mut a = box_app();
    for (tab, lines) in [
        (
            Category::Setup,
            &[
                "Action Center origin",
                "item.create camera",
                "Duplicate Item",
                "Bezier Curve",
                "operandi.rig.parent",
                "Parent to Group Locator",
                "Quick Align",
                "Mirror",
                "Ruler",
            ][..],
        ),
        (
            Category::Constraints,
            &[
                "operandi.rig.constrain kind:position",
                "operandi.rig.dynamicParentAdd",
            ][..],
        ),
        (
            Category::InverseKinematics,
            &["operandi.ik.applyTwoBone", "operandi.ik.switch"][..],
        ),
        (
            Category::Weighting,
            &[
                "operandi.skin.bindDialog",
                "Vertex Map Paint",
                "operandi.weight.normalize",
            ][..],
        ),
        (
            Category::GameVertexMap,
            &[
                "scene.saveAs format:fbx export:true",
                "Vertex Map Paint",
                "Select Hard Edges",
                "Convert to Hard Edges",
                "Transfer Normals",
                "Morph to Normals",
            ][..],
        ),
        (Category::Baking, &["operandi.bake.curvature"][..]),
    ] {
        assert!(a.show_toolbox_tab(tab), "{tab:?}");
        shows(&mut a, &ctx);
        for line in lines {
            assert!(a.form_rects.contains_key(*line), "{tab:?}: {line}");
        }
    }
}

/// Jason, 2026-11-23: Animation is a full form with no strip, like UV.
#[test]
fn the_animation_set_is_one_form_without_a_strip() {
    crate::anim_switch::set_for_test(true);
    let ctx = Context::default();
    let mut a = box_app();
    a.execute("operandi.toolbox.set animation");
    shows(&mut a, &ctx);
    assert_eq!(a.shown_category(), Some(Category::Animation));
    assert!(!a.toolbox_rects.keys().any(|k| k.starts_with("Category ")));
    for line in [
        "Action Center origin",
        "Action Center local",
        "item.create camera",
        "Duplicate Item",
        "Instance",
        "operandi.pose.tool",
        "Key Transforms",
        "operandi.ik.switch",
        "Play/Stop",
        "operandi.anim.dialog kind:playblast",
        "Snapping Toggle",
        "Snaps and Precision",
    ] {
        assert!(a.form_rects.contains_key(line), "missing {line}");
    }
    // The shared transform row at the top stays.
    assert!(a.toolbox_rects.contains_key("Move"));
}

/// Jason, 2026-11-23: UV is the UV form's buttons with no strip, whichever
/// layout is showing.
#[test]
fn the_uv_set_shows_the_uv_form_without_a_strip_in_every_layout() {
    let root = crate::layouts_tests::data_root();
    let ctx = Context::default();
    let mut a = crate::layouts_tests::launch(&root, &ctx);
    for layout in ["Model", "UV"] {
        a.execute(&format!("layout.restore {layout}"));
        a.execute("operandi.toolbox.set uv");
        shows(&mut a, &ctx);
        assert_eq!(a.layouts.current, layout);
        assert_eq!(a.shown_category(), Some(Category::Uv), "{layout}");
        assert!(
            !a.toolbox_rects.keys().any(|k| k.starts_with("Category ")),
            "{layout}: no strip"
        );
        for key in ["Unwrap Tool", "UV Unwrap", "Pack UVs"] {
            assert!(a.form_rects.contains_key(key), "{layout}: missing {key}");
        }
    }
}

#[test]
fn toolbox_tab_opens_the_first_set_that_has_it() {
    crate::anim_switch::set_for_test(true);
    let mut a = box_app();
    for (tab, set) in [
        ("uv", ToolSet::Uv),
        ("sculpt", ToolSet::Brush),
        ("weighting", ToolSet::Setup),
        ("rig", ToolSet::Setup),
        ("baking", ToolSet::Game),
        ("workbench", ToolSet::Workbench),
        ("polygon", ToolSet::Model),
    ] {
        a.execute(&format!("operandi.toolbox.tab {tab}"));
        assert_eq!(a.layouts.tool_set, set, "{tab}: {}", a.status);
        assert_eq!(a.category.label().to_lowercase(), tab);
    }
    // Rig is already in Game: from there it stays.
    a.execute("operandi.toolbox.set game");
    a.execute("operandi.toolbox.tab rig");
    assert_eq!(a.layouts.tool_set, ToolSet::Game);
}

#[test]
fn a_saved_tab_outside_its_set_opens_the_sets_first_tab() {
    let root = crate::layouts_tests::data_root();
    let ctx = Context::default();
    let mut a = crate::layouts_tests::launch(&root, &ctx);
    let mut doc = a.capture_layout().unwrap();
    doc.toolbox = Some(operandi_layout::ToolboxState {
        set: "Game".into(),
        tab: "Polygon".into(),
        uv_tab: false,
    });
    a.apply_layout_doc(&doc).unwrap();
    assert_eq!(a.layouts.tool_set, ToolSet::Game);
    assert_eq!(a.category, Category::GameVertexMap);
}

/// Jason, 2026-11-24: the toolbox's shared top row holds Move, Rotate,
/// Scale and Transform; no toolbox form repeats them (they ran the same
/// tools, so the UV form's copy went too).
#[test]
fn no_toolbox_form_repeats_the_shared_transform_row() {
    fn lines(v: &serde_json::Value, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(m) => {
                if let Some(serde_json::Value::String(s)) = m.get("command") {
                    out.push(s.clone());
                }
                m.values().for_each(|v| lines(v, out));
            }
            serde_json::Value::Array(a) => a.iter().for_each(|v| lines(v, out)),
            _ => {}
        }
    }
    let mut checked = 0;
    for (file, text) in crate::toolbox_form::SHIPPED_FORMS {
        if !file.starts_with("toolbox.") {
            continue;
        }
        let form: serde_json::Value = serde_json::from_str(text).unwrap();
        let mut all = Vec::new();
        lines(&form, &mut all);
        for top in ["Move", "Rotate", "Scale", "Transform"] {
            assert!(
                !all.iter().any(|l| l.trim().eq_ignore_ascii_case(top)),
                "{file} repeats {top}"
            );
        }
        checked += 1;
    }
    assert!(checked >= 15, "{checked} toolbox forms");
    // Drawn: the UV and Setup sets show the top row only.
    let ctx = Context::default();
    let mut a = box_app();
    for set in ["uv", "setup", "animation"] {
        a.execute(&format!("operandi.toolbox.set {set}"));
        shows(&mut a, &ctx);
        assert!(a.toolbox_rects.contains_key("Move"), "{set}: top row");
        for top in ["Move", "Rotate", "Scale", "Transform"] {
            assert!(!a.form_rects.contains_key(top), "{set}: form repeats {top}");
        }
    }
    assert!(a.form_rects.contains_key("Action Center origin"), "World");
    a.execute("operandi.toolbox.set setup");
    shows(&mut a, &ctx);
    assert!(
        a.form_rects.contains_key("Action Center local"),
        "Setup's Local"
    );
}

/// Every `icon` a toolbox form names resolves to an icon file (a studio
/// render or a generator SVG), so no button silently falls back to text.
#[test]
fn every_toolbox_form_icon_slug_resolves() {
    fn icons(v: &serde_json::Value, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(m) => {
                if let Some(serde_json::Value::String(s)) = m.get("icon")
                    && !s.is_empty()
                {
                    out.push(s.clone());
                }
                m.values().for_each(|v| icons(v, out));
            }
            serde_json::Value::Array(a) => a.iter().for_each(|v| icons(v, out)),
            _ => {}
        }
    }
    let mut files: Vec<(&str, &str)> = crate::toolbox_form::SHIPPED_FORMS.to_vec();
    #[cfg(feature = "painting")]
    files.extend_from_slice(crate::toolbox_form::PAINTING_FORMS);
    let mut checked = 0;
    for (file, text) in files {
        if !file.starts_with("toolbox.") {
            continue;
        }
        let form: serde_json::Value = serde_json::from_str(text).unwrap();
        let mut all = Vec::new();
        icons(&form, &mut all);
        for slug in all {
            assert!(
                icons::Icon::from_slug(&slug).is_some(),
                "{file}: icon {slug:?} has no file"
            );
            checked += 1;
        }
    }
    assert!(checked > 100, "{checked} icons");
}

/// Jason, 2026-11-24: every toolbox button shows a tooltip. A control with
/// none of its own falls back to its label, its command's help and its key.
#[test]
fn every_toolbox_form_button_has_hover_text() {
    use crate::form::Proficiency;
    use crate::form::command_form::{self as cf, Item};
    crate::anim_switch::set_for_test(true);
    let a = box_app();
    let mut checked = 0;
    let ids: Vec<String> = a
        .forms
        .ids()
        .into_iter()
        .filter(|id| id.starts_with("toolbox.") && *id != "toolbox.workbench")
        .map(str::to_owned)
        .collect();
    for id in ids {
        for item in cf::plan(&a.forms, &id, Proficiency::Advanced, &|_| true) {
            let controls = match item {
                Item::Button(c, _) => vec![*c],
                Item::Row(cs, _, _) => cs,
                _ => continue,
            };
            for c in controls {
                let Some(line) = c.command.as_deref() else {
                    continue;
                };
                let tip = a.form_button_tooltip(&c, line).unwrap_or_default();
                assert!(!tip.trim().is_empty(), "{id}: {line} has no tooltip");
                // Users never see spec codes (Jason, 2026-11-24).
                assert!(
                    !crate::ui_text::has_spec_code(&tip),
                    "{id}: {line} shows a spec code: {tip}"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 300, "{checked} buttons");
    // The fallback reads like the written ones: label, what it does, key.
    let setup = cf::plan(&a.forms, "toolbox.setup", Proficiency::Advanced, &|_| true);
    let parent = setup
        .into_iter()
        .flat_map(|i| match i {
            Item::Row(cs, _, _) => cs,
            Item::Button(c, _) => vec![*c],
            _ => vec![],
        })
        .find(|c| c.command.as_deref() == Some("operandi.rig.parent"))
        .unwrap();
    let tip = a
        .form_button_tooltip(&parent, "operandi.rig.parent")
        .unwrap();
    assert!(tip.starts_with("Parent · "), "{tip}");
}

/// Menu items' written tooltips carry no spec codes either; every registry
/// help line, as a tooltip shows it, is free of them too.
#[test]
fn menu_tooltips_and_shown_help_have_no_spec_codes() {
    fn tooltips(v: &serde_json::Value, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(m) => {
                if let Some(serde_json::Value::String(s)) = m.get("tooltip") {
                    out.push(s.clone());
                }
                m.values().for_each(|v| tooltips(v, out));
            }
            serde_json::Value::Array(a) => a.iter().for_each(|v| tooltips(v, out)),
            _ => {}
        }
    }
    for (file, text) in crate::toolbox_form::SHIPPED_FORMS {
        if !file.starts_with("menu.") {
            continue;
        }
        let form: serde_json::Value = serde_json::from_str(text).unwrap();
        let mut all = Vec::new();
        tooltips(&form, &mut all);
        for tip in all {
            assert!(!crate::ui_text::has_spec_code(&tip), "{file}: {tip}");
        }
    }
    for command in crate::command_handlers::registry().iter() {
        let shown = crate::ui_text::without_spec_codes(command.entry.help);
        assert!(
            !crate::ui_text::has_spec_code(&shown),
            "{}: {shown}",
            command.entry.name
        );
    }
}
