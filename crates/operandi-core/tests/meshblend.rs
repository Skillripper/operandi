//! MeshBlend B1 (meshblend-2026-11-24): the MeshBlend Item as a Mesh Stack
//! op, its recipe, evaluator and commands. Sharp joins only (strips are
//! B2).
//!
//! Acceptance criteria for B1:
//! 1. `meshblend.new` on selected meshes makes one new item whose stack is
//!    the MeshBlend op with them as Primaries; the result is their closed
//!    union, read as smooth surfaces (SB-01, SB-20, SB-42, SB-60).
//! 2. A Subtractive Trim cuts only the Primaries it's related to; Intersect
//!    keeps the overlap; one that misses leaves the Primary whole with a
//!    hint (SB-11, SB-13, SB-14).
//! 3. An open source is left out with a reason; the rest builds (SB-70).
//! 4. Every command is one undo step; undo and redo restore the recipe
//!    and the result exactly (SB-80).
//! 5. Save and reopen gives the same recipe and result (ST-02).
//! 6. The result doesn't depend on the order the sources are listed in
//!    (ST-09).
//! 7. A source that would make the item read itself is refused (SAFE-07).
//! 8. Convert freezes the result into an ordinary mesh (SB-53).
//! 9. Random command sequences never panic, never loop, and every result
//!    is a closed mesh of quads and triangles (ST-13, ST-10).
use glam::DVec3;
use operandi_core::mesh_stack::{OpStatus, Runtime};
use operandi_core::meshblend::{MeshBlendCommand, Recipe, Role, recipe_of};
use operandi_core::meshblend_spike::check;
use operandi_core::{Command, Editor, Mesh, persistence};

fn boxed(centre: [f64; 3]) -> Mesh {
    operandi_core::meshblend_spike::corpus::boxed(centre, [1., 1., 1.], [2, 2, 2], false)
}

fn mesh(e: &mut Editor, m: Mesh) -> u64 {
    e.scene.new_mesh().unwrap();
    let id = e.scene.items.last().unwrap().id;
    e.scene.item_mut(id).unwrap().mesh = m.into();
    id
}

/// Two overlapping boxes, both selected, and a MeshBlend Item of them.
fn two_boxes() -> (Editor, u64, u64, u64) {
    let mut e = Editor::default();
    let a = mesh(&mut e, boxed([0.; 3]));
    let b = mesh(&mut e, boxed([0.5; 3]));
    e.scene.select_item(Some(a));
    e.scene.toggle_item(b);
    e.dispatch(Command::MeshBlend(MeshBlendCommand::New))
        .unwrap();
    let blend = e.scene.selection.item.unwrap();
    (e, a, b, blend)
}

fn run(e: &mut Editor, c: MeshBlendCommand) -> operandi_core::Result<()> {
    e.dispatch(Command::MeshBlend(c))
}

fn result(e: &Editor, item: u64) -> operandi_core::mesh_stack::Evaluated {
    e.mesh_stack
        .current(&e.scene, e.scene.item(item).unwrap())
        .unwrap()
}

fn recipe(e: &Editor, item: u64) -> Recipe {
    recipe_of(&e.scene, item).unwrap().1
}

fn closed(m: &Mesh) -> check::Report {
    let r = check::st10(m, 0.02, None);
    assert_eq!(r.open_edges + r.bad_edges + r.ngons, 0, "{r:?}");
    r
}

#[test]
fn new_blends_the_selected_meshes_as_primaries() {
    let (e, a, b, blend) = two_boxes();
    assert_eq!(e.scene.items.len(), 3, "no helper items (SB-42)");
    assert_ne!(blend, a);
    let r = recipe(&e, blend);
    assert_eq!(
        r.sources
            .iter()
            .map(|s| (s.item, s.role))
            .collect::<Vec<_>>(),
        [(a, Role::Primary), (b, Role::Primary)]
    );
    // Q04: 2 % of the sources' 1.5 m size.
    assert!((r.strip_width - 0.03).abs() < 1e-9, "{}", r.strip_width);
    let out = result(&e, blend);
    assert_eq!(out.statuses, [OpStatus::Ok]);
    let rep = closed(&out.mesh);
    assert_eq!(rep.shells, 1);
    // Smooth sources: the union is smaller than the cages' 1.875.
    assert!(rep.volume > 1.0 && rep.volume < 1.875, "{}", rep.volume);
}

#[test]
fn new_needs_a_selected_mesh() {
    let mut e = Editor::default();
    let before = e.scene.clone();
    assert!(run(&mut e, MeshBlendCommand::New).is_err());
    assert_eq!(e.scene, before);
}

#[test]
fn trims_cut_only_their_primaries() {
    let (mut e, a, b, blend) = two_boxes();
    let union = check::volume(&result(&e, blend).mesh);
    run(
        &mut e,
        MeshBlendCommand::Role {
            blend: Some(blend),
            source: b,
            role: Role::Subtract,
        },
    )
    .unwrap();
    assert_eq!(recipe(&e, blend).source(b).unwrap().trims, [a]);
    let cut = result(&e, blend);
    let v = closed(&cut.mesh).volume;
    assert!(v > 0.3 && v < union * 0.6, "{v} vs union {union}");
    // Untrim: B is a Trim related to nothing, so A is whole again, with a hint.
    run(
        &mut e,
        MeshBlendCommand::Trim {
            blend: Some(blend),
            trims: vec![b],
            of: vec![a],
            untrim: true,
        },
    )
    .unwrap();
    let whole = result(&e, blend);
    assert!(
        matches!(&whole.statuses[0], OpStatus::Noted(n) if n.iter().any(|n| n.item == b && !n.problem))
    );
    assert!(check::volume(&whole.mesh) > v);
    // Intersect: only the overlap.
    run(
        &mut e,
        MeshBlendCommand::Trim {
            blend: Some(blend),
            trims: vec![b],
            of: vec![a],
            untrim: false,
        },
    )
    .unwrap();
    run(
        &mut e,
        MeshBlendCommand::Role {
            blend: Some(blend),
            source: b,
            role: Role::Intersect,
        },
    )
    .unwrap();
    let both = check::volume(&result(&e, blend).mesh);
    assert!(both > 0. && both < 0.2, "{both}");
}

#[test]
fn an_intersect_trim_that_misses_leaves_the_primary_whole() {
    let mut e = Editor::default();
    let a = mesh(&mut e, boxed([0.; 3]));
    let far = mesh(&mut e, boxed([3., 0., 0.]));
    e.scene.select_item(Some(a));
    run(&mut e, MeshBlendCommand::New).unwrap();
    let blend = e.scene.selection.item.unwrap();
    let alone = check::volume(&result(&e, blend).mesh);
    run(
        &mut e,
        MeshBlendCommand::AddSource {
            blend: Some(blend),
            source: far,
            role: Role::Intersect,
            of: vec![],
        },
    )
    .unwrap();
    let out = result(&e, blend);
    assert!((check::volume(&out.mesh) - alone).abs() < 1e-9);
    assert!(
        matches!(&out.statuses[0], OpStatus::Noted(n) if n.iter().any(|n| n.text.contains("doesn't reach")))
    );
}

#[test]
fn an_open_source_is_left_out_with_a_reason() {
    let mut e = Editor::default();
    let open = mesh(
        &mut e,
        operandi_core::meshblend_spike::corpus::boxed([0.; 3], [1.; 3], [2; 3], true),
    );
    let b = mesh(&mut e, boxed([0.5, 0., 0.5]));
    e.scene.select_item(Some(open));
    e.scene.toggle_item(b);
    run(&mut e, MeshBlendCommand::New).unwrap();
    let blend = e.scene.selection.item.unwrap();
    let out = result(&e, blend);
    match &out.statuses[0] {
        OpStatus::Noted(n) => assert!(
            n.iter()
                .any(|n| n.item == open && n.problem && n.text.contains("open edges")),
            "{n:?}"
        ),
        s => panic!("{s:?}"),
    }
    assert_eq!(closed(&out.mesh).shells, 1, "B alone still builds");
}

#[test]
fn every_edit_is_one_undo_step() {
    let (mut e, a, b, blend) = two_boxes();
    let start = (recipe(&e, blend), result(&e, blend).mesh.to_owned_value());
    let edits = [
        MeshBlendCommand::Role {
            blend: Some(blend),
            source: b,
            role: Role::Subtract,
        },
        MeshBlendCommand::Source {
            blend: Some(blend),
            source: a,
            enabled: None,
            subdivision: Some(1),
            keep_cage: None,
        },
        MeshBlendCommand::Clear {
            blend: Some(blend),
            source: b,
        },
    ];
    for c in edits {
        let before = (recipe(&e, blend), result(&e, blend).mesh.to_owned_value());
        run(&mut e, c.clone()).unwrap();
        let after = (recipe(&e, blend), result(&e, blend).mesh.to_owned_value());
        assert_ne!(before.0, after.0, "{c:?} changed nothing");
        e.dispatch(Command::Undo).unwrap();
        assert_eq!(recipe(&e, blend), before.0, "{c:?} undo");
        assert_eq!(*result(&e, blend).mesh, before.1, "{c:?} undo result");
        e.dispatch(Command::Redo).unwrap();
        assert_eq!(recipe(&e, blend), after.0, "{c:?} redo");
        assert_eq!(*result(&e, blend).mesh, after.1, "{c:?} redo result");
    }
    // Back past the creation: the item is gone, the sources untouched.
    for _ in 0..4 {
        e.dispatch(Command::Undo).unwrap();
    }
    assert!(e.scene.item(blend).is_none());
    assert_eq!(e.scene.items.len(), 2);
    let _ = start;
}

#[test]
fn save_and_reopen_give_the_same_result() {
    let (mut e, _a, b, blend) = two_boxes();
    run(
        &mut e,
        MeshBlendCommand::Role {
            blend: Some(blend),
            source: b,
            role: Role::Subtract,
        },
    )
    .unwrap();
    let bytes = persistence::encode(&e.scene).unwrap();
    let back = persistence::decode(&bytes).unwrap();
    assert_eq!(back, e.scene);
    let fresh = Runtime::default();
    let theirs = fresh.current(&back, back.item(blend).unwrap()).unwrap();
    assert_eq!(*theirs.mesh, *result(&e, blend).mesh);
}

#[test]
fn the_result_does_not_depend_on_source_order() {
    let (mut e, a, b, blend) = two_boxes();
    let first = result(&e, blend).mesh.to_owned_value();
    // The same sources listed the other way round, as a script could.
    let (op, mut r) = recipe_of(&e.scene, blend).unwrap();
    r.sources.reverse();
    assert_eq!(r.sources[0].item, b);
    let mut values = serde_json::Map::new();
    values.insert("sources".into(), serde_json::to_value(&r.sources).unwrap());
    e.dispatch(Command::MeshOpSet {
        op: operandi_core::mesh_stack_editor::OpRef { item: blend, op },
        values,
    })
    .unwrap();
    assert_eq!(recipe(&e, blend).sources[0].item, b);
    let second = result(&e, blend).mesh.to_owned_value();
    assert_eq!(check::fingerprint(&first), check::fingerprint(&second));
    let _ = a;
}

#[test]
fn a_source_that_reads_the_blend_is_refused() {
    let (mut e, _a, _b, blend) = two_boxes();
    let before = e.scene.clone();
    let err = run(
        &mut e,
        MeshBlendCommand::AddSource {
            blend: Some(blend),
            source: blend,
            role: Role::Primary,
            of: vec![],
        },
    )
    .unwrap_err();
    assert!(err.contains("own source"), "{err}");
    assert_eq!(e.scene, before);
    // A second MeshBlend of the first, then the first reading the second.
    e.scene.select_item(Some(blend));
    run(&mut e, MeshBlendCommand::New).unwrap();
    let outer = e.scene.selection.item.unwrap();
    let before = e.scene.clone();
    let err = run(
        &mut e,
        MeshBlendCommand::AddSource {
            blend: Some(blend),
            source: outer,
            role: Role::Primary,
            of: vec![],
        },
    )
    .unwrap_err();
    assert!(err.contains("loop"), "{err}");
    assert_eq!(e.scene, before);
}

#[test]
fn convert_freezes_the_result() {
    let (mut e, _a, _b, blend) = two_boxes();
    let shown = result(&e, blend).mesh.to_owned_value();
    run(
        &mut e,
        MeshBlendCommand::Convert {
            blend: Some(blend),
            duplicate: false,
        },
    )
    .unwrap();
    let item = e.scene.item(blend).unwrap();
    assert!(item.mesh_stack.is_none());
    assert_eq!(item.mesh.faces.len(), shown.faces.len());
}

#[test]
fn scripts_reach_every_command() {
    let mut e = Editor::default();
    let a = mesh(&mut e, boxed([0.; 3]));
    let b = mesh(&mut e, boxed([0.5; 3]));
    e.scene.select_item(Some(a));
    e.scene.toggle_item(b);
    let call = |e: &mut Editor, name: &str, args: serde_json::Value| {
        operandi_core::command_exec::run(e, name, args.as_object().unwrap())
            .unwrap_or_else(|err| panic!("{name}: {err}"))
    };
    use serde_json::json;
    call(&mut e, "meshblend.new", json!({}));
    let blend = e.scene.selection.item.unwrap();
    call(
        &mut e,
        "meshblend.role",
        json!({"source": b, "role": "subtract"}),
    );
    call(&mut e, "meshblend.untrim", json!({"source": b, "of": a}));
    call(&mut e, "meshblend.trim", json!({"source": [b], "of": [a]}));
    call(
        &mut e,
        "meshblend.source",
        json!({"source": a, "subdivision": 1, "keepCage": true}),
    );
    call(&mut e, "meshblend.clearRole", json!({"source": b}));
    call(
        &mut e,
        "meshblend.addSource",
        json!({"source": b, "role": "intersect"}),
    );
    let r = recipe(&e, blend);
    assert_eq!(r.source(b).unwrap().role, Role::Intersect);
    assert!(r.source(a).unwrap().keep_cage);
    call(&mut e, "meshblend.convert", json!({"duplicate": true}));
    assert_eq!(e.scene.items.len(), 4);
}

#[test]
fn random_sequences_stay_valid() {
    for seed in 1..=3u64 {
        let mut rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut below = |n: usize| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            (rng % n.max(1) as u64) as usize
        };
        let mut e = Editor::default();
        let ids: Vec<u64> = [[0.; 3], [0.5; 3], [-0.4, 0.3, 0.2]]
            .into_iter()
            .map(|c| mesh(&mut e, boxed(c)))
            .collect();
        e.scene.select_item(Some(ids[0]));
        e.scene.toggle_item(ids[1]);
        run(&mut e, MeshBlendCommand::New).unwrap();
        let blend = e.scene.selection.item.unwrap();
        for step in 0..40 {
            let pick = |n: usize| ids[n % ids.len()];
            let s = pick(below(3));
            let role = Role::ALL[below(3)];
            let cmd = match below(9) {
                0 => Command::MeshBlend(MeshBlendCommand::AddSource {
                    blend: Some(blend),
                    source: s,
                    role,
                    of: vec![],
                }),
                1 => Command::MeshBlend(MeshBlendCommand::Role {
                    blend: Some(blend),
                    source: s,
                    role,
                }),
                2 | 3 => Command::MeshBlend(MeshBlendCommand::Trim {
                    blend: Some(blend),
                    trims: vec![s],
                    of: vec![pick(below(3))],
                    untrim: below(2) == 0,
                }),
                4 => Command::MeshBlend(MeshBlendCommand::Clear {
                    blend: Some(blend),
                    source: s,
                }),
                5 => Command::MeshBlend(MeshBlendCommand::Source {
                    blend: Some(blend),
                    source: s,
                    enabled: Some(below(4) != 0),
                    subdivision: Some(below(3) as u8),
                    keep_cage: Some(below(4) == 0),
                }),
                6 => {
                    let p = e.scene.item_mut(s).unwrap();
                    p.transform.translation += DVec3::new(0.1, 0., -0.05);
                    continue;
                }
                7 => Command::Undo,
                _ => Command::Redo,
            };
            let before = e.scene.clone();
            if e.dispatch(cmd).is_err() {
                assert_eq!(e.scene, before, "seed {seed} step {step}");
            }
            if let Some(item) = e.scene.item(blend) {
                let out = e.mesh_stack.current(&e.scene, item).unwrap();
                assert!(
                    !matches!(out.statuses[0], OpStatus::Failed(_)),
                    "seed {seed} step {step}: {:?}",
                    out.statuses
                );
                if !out.mesh.faces.is_empty() {
                    closed(&out.mesh);
                }
            }
        }
        // Replaying the recipe from scratch equals the live result (ST-13).
        if let Some(item) = e.scene.item(blend) {
            let live = e.mesh_stack.current(&e.scene, item).unwrap();
            let fresh = Runtime::default().current(&e.scene, item).unwrap();
            assert_eq!(*live.mesh, *fresh.mesh, "seed {seed}");
        }
    }
}
