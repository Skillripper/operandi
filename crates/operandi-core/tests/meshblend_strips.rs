//! MeshBlend B2: rounded strips along every join (meshblend/strips.rs, the
//! strip builder picked in B0), through the plain evaluator.
//!
//! Acceptance criteria:
//! 1. Two overlapping smooth boxes get a strip of quads along their join
//!    and the result passes ST-10 (SB-30, SB-33).
//! 2. Width 0 makes no strips (sharp joins).
//! 3. Three boxes meeting get corner patches; still ST-10 (SB-34).
//! 4. A strip too wide to fit is narrowed with a note, never broken
//!    (ST-11, SB-72); the shape keeps most of its volume.
//! 5. Design Mode gives narrow flat strips (ST-16).
//! 6. Plain polygons (Tab off) get strips too, and most of the cut faces
//!    are quads again (pairing).
//! 7. Same inputs in another order give the same result bit for bit (ST-09).
use operandi_core::Mesh;
use operandi_core::meshblend::check;
use operandi_core::meshblend::{BlendSource, Input, Recipe, Role, evaluate};
use operandi_core::meshblend_spike::corpus;

fn smooth(mut m: Mesh) -> Mesh {
    m.attributes.representations =
        vec![operandi_core::PolygonRepresentation::CatmullClark; m.faces.len()];
    m
}
fn input(item: u64, mesh: Mesh) -> Input {
    Input {
        item,
        name: format!("Mesh {item}"),
        mesh,
    }
}
fn recipe(items: &[u64], width: f64) -> Recipe {
    Recipe {
        sources: items
            .iter()
            .map(|&i| BlendSource::new(i, Role::Primary))
            .collect(),
        strip_width: width,
        ..Default::default()
    }
}
fn two_boxes() -> Vec<Input> {
    vec![
        input(1, smooth(corpus::boxed([0.; 3], [1.; 3], [2; 3], false))),
        input(2, smooth(corpus::boxed([0.5; 3], [1.; 3], [2; 3], false))),
    ]
}
fn strips(b: &operandi_core::meshblend::Blend) -> usize {
    b.origin.iter().filter(|o| o.polygon == u32::MAX).count()
}

#[test]
fn a_join_gets_a_strip_of_quads() {
    let b = evaluate(&recipe(&[1, 2], 0.1), &two_boxes()).unwrap();
    assert!(b.notes.is_empty(), "{:?}", b.notes);
    let r = check::st10(&b.mesh, 0.1, None);
    assert!(r.passes(), "{:?}", r.failures());
    assert_eq!(r.shells, 1);
    let n = strips(&b);
    assert!(n > 100, "{n} strip polygons");
    let strip_quads = b
        .mesh
        .faces
        .iter()
        .zip(&b.origin)
        .filter(|(f, o)| o.polygon == u32::MAX && f.vertices.len() == 4)
        .count();
    assert_eq!(strip_quads, n, "every strip polygon is a quad");
    // The round adds material in the concave crease.
    let sharp = evaluate(&recipe(&[1, 2], 0.), &two_boxes()).unwrap();
    assert!(check::volume(&b.mesh) > check::volume(&sharp.mesh));
}

#[test]
fn width_zero_keeps_sharp_joins() {
    let b = evaluate(&recipe(&[1, 2], 0.), &two_boxes()).unwrap();
    assert_eq!(strips(&b), 0);
    assert!(check::st10(&b.mesh, 0.1, None).passes());
}

#[test]
fn three_strips_meet_in_corner_patches() {
    let mut inputs = vec![input(
        1,
        smooth(corpus::boxed([0.; 3], [1.; 3], [2; 3], false)),
    )];
    inputs.push(input(
        2,
        smooth(corpus::boxed([0.6, 0.3, 0.], [1., 0.6, 0.6], [2; 3], false)),
    ));
    inputs.push(input(
        3,
        smooth(corpus::boxed([0., 0.3, 0.6], [0.6, 0.6, 1.], [2; 3], false)),
    ));
    let b = evaluate(&recipe(&[1, 2, 3], 0.05), &inputs).unwrap();
    let r = check::st10(&b.mesh, 0.05, None);
    assert!(r.passes(), "{:?} {:?}", r.failures(), b.notes);
    assert!(strips(&b) > 100);
}

#[test]
fn a_strip_too_wide_is_narrowed_not_broken() {
    let sharp = check::volume(&evaluate(&recipe(&[1, 2], 0.), &two_boxes()).unwrap().mesh);
    for width in [0.6, 1.0, 2.0] {
        let b = evaluate(&recipe(&[1, 2], width), &two_boxes()).unwrap();
        let r = check::st10(&b.mesh, width, None);
        assert!(r.open_edges + r.bad_edges + r.ngons == 0, "{width}: {r:?}");
        let v = check::volume(&b.mesh);
        assert!(v > 0.8 * sharp, "{width}: volume {v} vs sharp {sharp}");
        if strips(&b) == 0 || b.notes.iter().any(|n| n.text.contains("limited")) {
            assert!(!b.notes.is_empty(), "{width}: changed without a note");
        }
    }
}

#[test]
fn design_mode_makes_narrow_flat_strips() {
    let mut r = recipe(&[1, 2], 0.2);
    r.design_mode = true;
    let b = evaluate(&r, &two_boxes()).unwrap();
    assert!(strips(&b) > 0);
    assert!(check::st10(&b.mesh, 0.02, None).passes());
}

#[test]
fn plain_meshes_get_strips_and_mostly_quads() {
    let inputs = vec![
        input(1, corpus::boxed([0.; 3], [1.; 3], [4; 3], false)),
        input(2, corpus::cylinder([0.3, 0.5, 0.3], 0.3, 0.8, 24, 2)),
    ];
    let b = evaluate(&recipe(&[1, 2], 0.05), &inputs).unwrap();
    let r = check::st10(&b.mesh, 0.05, None);
    assert!(r.passes(), "{:?} {:?}", r.failures(), b.notes);
    assert!(strips(&b) > 0, "{:?}", b.notes);
    assert!(
        r.quads > 2 * r.triangles,
        "{} quads, {} triangles",
        r.quads,
        r.triangles
    );
}

#[test]
fn results_repeat_in_any_source_order() {
    let mut flipped = two_boxes();
    flipped.reverse();
    let mut r = recipe(&[2, 1], 0.1);
    let a = evaluate(&recipe(&[1, 2], 0.1), &two_boxes()).unwrap();
    let b = evaluate(&r, &flipped).unwrap();
    assert_eq!(check::fingerprint(&a.mesh), check::fingerprint(&b.mesh));
    r.sources.reverse();
    let c = evaluate(&r, &two_boxes()).unwrap();
    assert_eq!(check::fingerprint(&a.mesh), check::fingerprint(&c.mesh));
}

/// Triangles with a corner under 10° on source `item`, for the tidy pass.
fn slivers(b: &operandi_core::meshblend::Blend, item: u64) -> usize {
    let m = &b.mesh;
    m.faces
        .iter()
        .zip(&b.origin)
        .filter(|(f, o)| f.vertices.len() == 3 && o.item == item && o.polygon != u32::MAX)
        .map(|(f, _)| f)
        .filter(|f| {
            let p: Vec<glam::DVec3> = f.vertices.iter().map(|&v| m.vertices[v as usize]).collect();
            let ang = |a: glam::DVec3, b: glam::DVec3, c: glam::DVec3| (b - a).angle_between(c - a);
            ang(p[0], p[1], p[2])
                .min(ang(p[1], p[2], p[0]))
                .min(ang(p[2], p[0], p[1]))
                < 10f64.to_radians()
        })
        .count()
}

#[test]
fn the_cut_faces_beside_strips_have_few_slivers() {
    // Jason's QA case: a one-segment cube and a cylinder, Tab off.
    let inputs = vec![
        input(1, corpus::boxed([0.; 3], [1.; 3], [1; 3], false)),
        input(2, corpus::cylinder([0.3, 0.5, 0.3], 0.3, 0.8, 24, 1)),
    ];
    let b = evaluate(&recipe(&[1, 2], 0.05), &inputs).unwrap();
    assert!(strips(&b) > 0, "{:?}", b.notes);
    // The cube's faces (the cylinder's own tall side quads are thin by
    // design, so they don't count).
    let n = slivers(&b, 1);
    let tris = b
        .mesh
        .faces
        .iter()
        .zip(&b.origin)
        .filter(|(f, o)| f.vertices.len() == 3 && o.item == 1)
        .count();
    if std::env::var("OPERANDI_B0_PICTURE").is_ok() {
        use operandi_core::meshblend_spike::picture;
        let kind: Vec<u8> = b
            .origin
            .iter()
            .map(|o| u8::from(o.polygon == u32::MAX))
            .collect();
        let source: Vec<u32> = b.origin.iter().map(|o| o.item as u32).collect();
        let view = picture::View::fit(&b.mesh, glam::DVec3::new(1.0, 0.85, 1.45));
        let rgb = picture::render(&b.mesh, &kind, &source, view, 900, 680);
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../development/meshblend-b0/cube-cylinder.png");
        picture::save_png(&out, 900, 680, &rgb).unwrap();
        eprintln!(
            "{n} slivers of {tris} triangles, {} polygons",
            b.mesh.faces.len()
        );
    }
    // Fans remain where many rim points face one long cube edge (an edge
    // shared with the next face can't take new points yet).
    assert!(n * 2 <= tris.max(1), "{n} slivers of {tris} triangles");
}

#[test]
fn output_modes() {
    let with = |output: u32| {
        let mut r = recipe(&[1, 2], 0.1);
        r.output = output;
        evaluate(&r, &two_boxes()).unwrap()
    };
    // Draft: no strips.
    assert_eq!(strips(&with(0)), 0);
    // Closed keeps the triangles Closed, clean pairs into quads.
    let (closed, clean) = (with(1), with(2));
    let tris = |b: &operandi_core::meshblend::Blend| {
        b.mesh
            .faces
            .iter()
            .filter(|f| f.vertices.len() == 3)
            .count()
    };
    assert!(strips(&closed) > 0);
    assert!(
        tris(&clean) < tris(&closed),
        "{} vs {}",
        tris(&clean),
        tris(&closed)
    );
    assert!(check::st10(&closed.mesh, 0.1, None).passes());
    // Parts per source: every polygon in exactly one part, named after
    // its source or Strips (BLD-C06).
    let per_source = with(4);
    let parts: std::collections::BTreeSet<_> = per_source
        .mesh
        .faces
        .iter()
        .map(|f| f.part.clone().expect("every polygon has a part"))
        .collect();
    assert_eq!(
        parts.into_iter().collect::<Vec<_>>(),
        ["Mesh 1", "Mesh 2", "Strips"]
    );
    // Parts per patch: the two sides of the strip are two patches.
    let per_patch = with(3);
    let patches: std::collections::BTreeSet<_> = per_patch
        .mesh
        .faces
        .iter()
        .filter_map(|f| f.part.clone())
        .collect();
    assert!(
        patches.contains("Strips") && patches.contains("Patch 1") && patches.contains("Patch 2")
    );
    // Strips only: just the band, open at both edges.
    let only = with(5);
    assert!(only.origin.iter().all(|o| o.polygon == u32::MAX));
    assert!(check::st10(&only.mesh, 0.1, None).open_edges > 0);
}
