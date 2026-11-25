//! Schematic workspaces (BEHAVIOR-SCHEMATIC SC-01..17, SC-41, SC-60,
//! SC-73..75, SC-91): pages show only what is placed on them, at saved
//! places that survive save and reopen, and nodes arrive arranged only
//! when they come without a place.
use operandi_core::schematic_layout::Metrics;
use operandi_core::schematic_pages::{PageId, page, pages};
use operandi_core::*;

fn line(e: &mut Editor, text: &str) -> Result<Option<String>> {
    command_line::run_line(e, text).expect("headless")
}

fn ok(e: &mut Editor, text: &str) {
    if let Err(err) = line(e, text) {
        panic!("{text}: {err}");
    }
}

fn item(e: &mut Editor) -> u64 {
    e.scene.new_mesh().unwrap();
    e.scene.items.last().unwrap().id
}

/// a drives an expression on b; c is plain.
fn rig() -> (Editor, u64, u64, u64, u64) {
    let mut e = Editor::default();
    let a = item(&mut e);
    let b = item(&mut e);
    let c = item(&mut e);
    ok(
        &mut e,
        &format!("operandi.rig.modifierAdd kind:expression item:{b}"),
    );
    let m = e.scene.animation.rig.pieces[0].id;
    ok(
        &mut e,
        &format!("operandi.rig.modifierSet piece:{m} inputs:A={a}:pos.X output:{b}:rot.Y expr:A"),
    );
    (e, a, b, c, m)
}

#[test]
fn sc74_a_rig_with_no_workspace_shows_one_arranged_page() {
    let (e, ..) = rig();
    let m = Metrics::default();
    assert_eq!(pages(&e.scene), [(PageId::All, "Workspace".to_owned())]);
    let v = page(&e.scene, PageId::All, &m);
    assert_eq!(v.schematic.nodes.len(), 3);
    assert_eq!(v.positions.len(), 3);
    assert!(v.saved.is_empty(), "shown, not saved");
    assert!(pages(&Scene::default()).is_empty());
}

#[test]
fn sc10_sc91_placed_nodes_are_saved_and_reopen_where_they_were() {
    let (mut e, a, b, c, m) = rig();
    ok(&mut e, "operandi.rig.workspaceAdd name:Arms");
    let ws = e.scene.animation.rig.workspaces[0].id;
    assert_eq!(pages(&e.scene)[0], (PageId::Saved(ws), "Arms".to_owned()));
    // Only what's placed shows; a wire to a node elsewhere is a stub (SC-14).
    ok(
        &mut e,
        &format!(
            "operandi.rig.schematicPlace workspace:{ws} place:{{item:{a}@10/20,item:{c}@300/-40}}"
        ),
    );
    let v = page(&e.scene, PageId::Saved(ws), &Metrics::default());
    let mut keys: Vec<&String> = v.positions.keys().collect();
    keys.sort();
    // The expression touches a: it shows beside a (SC-16), not saved.
    assert!(v.positions.contains_key(&format!("piece:{m}")));
    assert!(!v.saved.contains(&format!("piece:{m}")));
    assert!(!v.positions.contains_key(&format!("item:{b}")));
    assert!(v.stubs.iter().any(|s| s.other == format!("item:{b}")));
    assert_eq!(v.positions[&format!("item:{a}")], [10., 20.]);
    // Nodes without a place arrive arranged, below what's there (SC-60).
    ok(
        &mut e,
        &format!("operandi.rig.schematicPlace workspace:{ws} nodes:{{item:{b},piece:{m}}}"),
    );
    let w = &e.scene.animation.rig.workspaces[0];
    let placed = |k: String| w.nodes.iter().find(|n| n.key == k).cloned().unwrap();
    assert!(placed(format!("item:{b}")).y > 20.);
    assert!(!placed(format!("item:{b}")).pinned);
    assert!(placed(format!("item:{a}")).pinned);
    // Save and reopen.
    let bytes = persistence::encode(&e.scene).unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("\"version\": 67"));
    let back = persistence::decode(&bytes).unwrap();
    assert_eq!(
        back.animation.rig.workspaces,
        e.scene.animation.rig.workspaces
    );
    // Undo takes the last place back off.
    e.dispatch(Command::Undo).unwrap();
    assert_eq!(e.scene.animation.rig.workspaces[0].nodes.len(), 2);
}

#[test]
fn sc41_sc06_removing_and_clearing_never_touch_the_scene() {
    let (mut e, a, _, c, m) = rig();
    ok(
        &mut e,
        &format!(
            "operandi.rig.workspaceAdd name:Arms place:{{item:{a}@0/0,item:{c}@0/80,piece:{m}@200/0}}"
        ),
    );
    let ws = e.scene.animation.rig.workspaces[0].id;
    ok(
        &mut e,
        &format!("operandi.rig.schematicRemove workspace:{ws} nodes:{{piece:{m}}}"),
    );
    assert_eq!(e.scene.animation.rig.pieces.len(), 1, "the piece stays");
    assert_eq!(e.scene.animation.rig.workspaces[0].nodes.len(), 2);
    ok(
        &mut e,
        &format!("operandi.rig.workspaceRename workspace:{ws} name:Legs"),
    );
    assert_eq!(e.scene.animation.rig.workspaces[0].name, "Legs");
    ok(
        &mut e,
        &format!("operandi.rig.workspaceClear workspace:{ws}"),
    );
    assert!(e.scene.animation.rig.workspaces[0].nodes.is_empty());
    assert_eq!(e.scene.items.len(), 3);
    // A piece on no page now shows on "Unplaced" (SC-73).
    assert!(pages(&e.scene).iter().any(|p| p.0 == PageId::Unplaced));
    ok(
        &mut e,
        &format!("operandi.rig.workspaceRemove workspace:{ws}"),
    );
    assert!(e.scene.animation.rig.workspaces.is_empty());
    assert_eq!(pages(&e.scene)[0].0, PageId::All);
}

#[test]
fn sc60_arrange_leaves_hand_placed_nodes_alone_unless_named() {
    let (mut e, a, b, _, m) = rig();
    ok(
        &mut e,
        &format!("operandi.rig.workspaceAdd name:Arms place:{{item:{a}@0/0}}"),
    );
    let ws = e.scene.animation.rig.workspaces[0].id;
    ok(
        &mut e,
        &format!("operandi.rig.schematicPlace workspace:{ws} place:{{item:{a}@500/500}}"),
    );
    ok(
        &mut e,
        &format!(
            "operandi.rig.schematicPlace workspace:{ws} place:{{item:{b}@0/0,piece:{m}@0/0}} pinned:false"
        ),
    );
    ok(
        &mut e,
        &format!("operandi.rig.schematicArrange workspace:{ws}"),
    );
    let w = &e.scene.animation.rig.workspaces[0];
    let at = |k: String| {
        let n = w.nodes.iter().find(|n| n.key == k).unwrap();
        [n.x, n.y]
    };
    assert_eq!(at(format!("item:{a}")), [500., 500.], "pinned");
    assert!(
        at(format!("piece:{m}"))[0] < at(format!("item:{b}"))[0],
        "flow left to right"
    );
    assert!(
        at(format!("item:{b}"))[1] > 500.,
        "below the hand-placed node"
    );
    ok(
        &mut e,
        &format!(
            "operandi.rig.schematicArrange workspace:{ws} nodes:{{item:{a},item:{b},piece:{m}}}"
        ),
    );
    let w = &e.scene.animation.rig.workspaces[0];
    let a_at = w
        .nodes
        .iter()
        .find(|n| n.key == format!("item:{a}"))
        .unwrap();
    assert_ne!([a_at.x, a_at.y], [500., 500.], "named, so arranged");
}

#[test]
fn bad_workspace_lines_are_refused() {
    let (mut e, a, ..) = rig();
    assert!(
        line(
            &mut e,
            "operandi.rig.schematicPlace workspace:999 nodes:{item:1}"
        )
        .is_err()
    );
    ok(&mut e, "operandi.rig.workspaceAdd");
    let ws = e.scene.animation.rig.workspaces[0].id;
    assert_eq!(e.scene.animation.rig.workspaces[0].name, "Workspace");
    for bad in [
        format!("operandi.rig.schematicPlace workspace:{ws} place:{{item:{a}@1}}"),
        format!("operandi.rig.schematicPlace workspace:{ws} place:{{item:9999@1/2}}"),
        format!("operandi.rig.schematicPlace workspace:{ws}"),
        format!("operandi.rig.workspaceRename workspace:{ws}"),
    ] {
        assert!(line(&mut e, &bad).is_err(), "{bad}");
    }
    let json = command_exec::answer(&e, "operandi.rig.workspaces", &Default::default())
        .unwrap()
        .unwrap();
    assert!(json.contains("\"Workspace\""));
}
