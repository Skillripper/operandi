//! MeshBlend B0 spike: the corpus, the ST-10 / ST-09 checker and the two
//! strip prototypes (operandi-core `meshblend_spike`).
//!
//! The comparison run that writes the pictures and numbers for Jason is
//! ignored by default (it renders 50 pictures):
//!
//! ```text
//! cargo test --release --locked --workspace -- meshblend_spike::b0_comparison --ignored --nocapture
//! ```
//!
//! It writes `development/meshblend-b0/` (git-ignored): one PNG per scene
//! and method, and `report.json`.
use operandi_core::meshblend_spike::sharp::{self, Role, Source};
use operandi_core::meshblend_spike::{Blended, check, corpus, picture, strip_bevel, strip_builder};
use std::time::Instant;

fn scene(name: &str) -> corpus::Scene {
    corpus::all()
        .into_iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no corpus scene {name}"))
}

#[test]
fn corpus_has_a_scene_for_every_listed_case() {
    let all = corpus::all();
    assert!(all.len() >= 25, "{} scenes", all.len());
    for name in corpus::PICTURED {
        scene(name);
    }
}

#[test]
fn sharp_union_of_two_cubes_is_closed_and_one_shell() {
    let s = scene("two-cubes");
    let sharp = sharp::build(&s.sources, 1).unwrap();
    let r = check::st10(&sharp.mesh(), s.width, None);
    assert!(r.passes(), "{:?}", r.failures());
    assert_eq!(r.shells, 1);
}

#[test]
fn open_primary_is_left_out_with_a_reason() {
    // BLD-C15 / SB-70: the open box is named and the other still builds.
    let s = scene("open-primary");
    let sharp = sharp::build(&s.sources, 1).unwrap();
    assert!(
        sharp
            .notes
            .iter()
            .any(|n| n.contains("A has") && n.contains("open edges")),
        "{:?}",
        sharp.notes
    );
    assert!(!sharp.tris.is_empty());
}

#[test]
fn identical_subtract_is_exactly_empty() {
    // BLD-C17 / SB-73.
    let s = scene("identical");
    let sharp = sharp::build(&s.sources, 1).unwrap();
    assert!(sharp.tris.is_empty());
    assert!(sharp.notes.iter().any(|n| n.contains("empty")));
}

#[test]
fn intersect_trim_that_misses_leaves_the_primary_whole() {
    // BLD-C23 / SB-14.
    let s = scene("intersect-miss");
    let sharp = sharp::build(&s.sources, 1).unwrap();
    assert!(!sharp.tris.is_empty());
    assert!(sharp.notes.iter().any(|n| n.contains("doesn't reach")));
}

#[test]
fn results_do_not_depend_on_source_order() {
    // BLD-C24 / ST-09: the same sources listed the other way round give
    // bit-identical sharp results and strips.
    let s = scene("two-cubes");
    let mut flipped: Vec<Source> = s.sources.clone();
    flipped.reverse();
    let a = sharp::build(&s.sources, 1).unwrap();
    let b = sharp::build(&flipped, 1).unwrap();
    assert_eq!(check::fingerprint(&a.mesh()), check::fingerprint(&b.mesh()));
    let da = strip_builder::blend(&a, 0.1, 100., 2);
    let db = strip_builder::blend(&b, 0.1, 100., 2);
    assert_eq!(check::fingerprint(&da.mesh), check::fingerprint(&db.mesh));
}

#[test]
fn trims_follow_their_listed_primary_after_reordering() {
    // A Subtractive Trim names its Primary by list index; reordering must
    // keep the relationship (ST-09 canonical order).
    let s = scene("subtract");
    let flipped = vec![
        Source {
            role: Role::Subtract(vec![1]),
            ..s.sources[1].clone()
        },
        s.sources[0].clone(),
    ];
    let a = sharp::build(&s.sources, 1).unwrap();
    let b = sharp::build(&flipped, 1).unwrap();
    assert_eq!(check::fingerprint(&a.mesh()), check::fingerprint(&b.mesh()));
}

// ── The comparison run ───────────────────────────────────────────────────

struct Run {
    blended: Blended,
    report: check::Report,
    secs: f64,
    repeatable: bool,
}

fn run_method(
    s: &corpus::Scene,
    sharp: &sharp::Sharp,
    flipped: &sharp::Sharp,
    method: &str,
) -> Run {
    let go = |sh: &sharp::Sharp| -> Blended {
        match method {
            "B" => strip_bevel::blend(sh, s.width, s.profile, s.rows).unwrap_or_else(|e| Blended {
                problems: vec![e],
                ..Default::default()
            }),
            _ => strip_builder::blend(sh, s.width, s.profile, s.rows),
        }
    };
    let started = Instant::now();
    let blended = go(sharp);
    let secs = started.elapsed().as_secs_f64();
    let again = go(flipped);
    let report = check::st10(&blended.mesh, s.width, Some(&blended.kind));
    Run {
        repeatable: check::fingerprint(&blended.mesh) == check::fingerprint(&again.mesh),
        blended,
        report,
        secs,
    }
}

#[test]
#[ignore = "writes the B0 comparison pictures; run in release (see the file comment)"]
fn b0_comparison() {
    let out =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../development/meshblend-b0");
    std::fs::create_dir_all(&out).unwrap();
    let mut rows = Vec::new();
    for s in corpus::all() {
        let started = Instant::now();
        let sharp = sharp::build(&s.sources, s.level).unwrap();
        let sharp_secs = started.elapsed().as_secs_f64();
        let mut flipped_sources = s.sources.clone();
        flipped_sources.reverse();
        // Reversing the list breaks Trim targets; rename them.
        let n = flipped_sources.len();
        for src in &mut flipped_sources {
            src.role = match &src.role {
                Role::Primary => Role::Primary,
                Role::Subtract(of) => Role::Subtract(of.iter().map(|i| n - 1 - i).collect()),
                Role::Intersect(of) => Role::Intersect(of.iter().map(|i| n - 1 - i).collect()),
            };
        }
        let flipped = sharp::build(&flipped_sources, s.level).unwrap();
        let from = match s.name {
            "two-cubes" | "wide-0.3" | "wide-0.6" | "wide-1.0" | "subtract" => {
                glam::DVec3::new(1.2, 0.55, -1.0)
            }
            "three-boxes" => glam::DVec3::new(0.7, 1.1, 0.9),
            "sixteen-cutters" => glam::DVec3::new(0.7, 1.4, 1.1),
            _ => glam::DVec3::new(1.0, 0.85, 1.45),
        };
        let view = picture::View::fit(&sharp.mesh(), from);
        // A close look at the join point nearest the camera.
        let close = {
            let m = sharp.mesh();
            let mut on_join = std::collections::HashMap::new();
            for (t, tri) in sharp.tris.iter().enumerate() {
                for &v in tri {
                    on_join
                        .entry(v)
                        .or_insert_with(std::collections::HashSet::new)
                        .insert(sharp.source[t]);
                }
            }
            let near = on_join
                .iter()
                .filter(|(_, s)| s.len() > 1)
                .map(|(&v, _)| m.vertices[v as usize])
                .min_by(|a, b| {
                    a.dot(view.forward)
                        .total_cmp(&b.dot(view.forward))
                        .then(a.x.total_cmp(&b.x))
                        .then(a.z.total_cmp(&b.z))
                });
            near.map(|p| view.zoom(p, view.height * 0.35))
        };
        let mut methods = serde_json::Map::new();
        for method in ["B", "D"] {
            let r = run_method(&s, &sharp, &flipped, method);
            if corpus::PICTURED.contains(&s.name) || std::env::var("OPERANDI_B0_ALL").is_ok() {
                let (w, h) = (900, 680);
                let rgb = picture::render(
                    &r.blended.mesh,
                    &r.blended.kind,
                    &r.blended.source,
                    view,
                    w,
                    h,
                );
                picture::save_png(&out.join(format!("{}-{method}.png", s.name)), w, h, &rgb)
                    .unwrap();
                if let Some(close) = close {
                    let rgb = picture::render(
                        &r.blended.mesh,
                        &r.blended.kind,
                        &r.blended.source,
                        close,
                        w,
                        h,
                    );
                    picture::save_png(
                        &out.join(format!("{}-{method}-close.png", s.name)),
                        w,
                        h,
                        &rgb,
                    )
                    .unwrap();
                }
            }
            let rep = &r.report;
            println!(
                "{:<16} {method}  st10 {:<5} rep {:<5} polys {:>6} tri {:>5} quad {:>6} ngon {:>3} open {:>3} bad {:>3} dup {:>3} short {:>3} stripNQ {:>4} shells {} vol {:.4} t {:.3}+{:.3}s {:?}",
                s.name,
                rep.passes(),
                r.repeatable,
                rep.polygons,
                rep.triangles,
                rep.quads,
                rep.ngons,
                rep.open_edges,
                rep.bad_edges,
                rep.duplicate_points,
                rep.short_edges,
                rep.strip_non_quads,
                rep.shells,
                rep.volume,
                sharp_secs,
                r.secs,
                r.blended.problems
            );
            methods.insert(
                method.into(),
                serde_json::json!({
                    "st10": rep.passes(),
                    "st10_failures": rep.failures(),
                    "repeatable": r.repeatable,
                    "polygons": rep.polygons,
                    "triangles": rep.triangles,
                    "quads": rep.quads,
                    "ngons": rep.ngons,
                    "open_edges": rep.open_edges,
                    "bad_edges": rep.bad_edges,
                    "duplicate_points": rep.duplicate_points,
                    "short_edges": rep.short_edges,
                    "strip_non_quads": rep.strip_non_quads,
                    "strip_polygons": r.blended.kind.iter().filter(|&&k| k != 0).count(),
                    "shells": rep.shells,
                    "volume": rep.volume,
                    "strip_secs": r.secs,
                    "problems": r.blended.problems,
                }),
            );
        }
        rows.push(serde_json::json!({
            "scene": s.name,
            "case": s.case,
            "width": s.width,
            "profile": s.profile,
            "rows": s.rows,
            "level": s.level,
            "sharp_secs": sharp_secs,
            "smooth_secs": sharp.smooth_secs,
            "boolean_secs": sharp.boolean_secs,
            "sharp_volume": check::volume(&sharp.mesh()),
            "notes": sharp.notes,
            "methods": methods,
        }));
    }
    std::fs::write(
        out.join("report.json"),
        serde_json::to_string_pretty(&rows).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "debug one scene: OPERANDI_B0_SCENE=<name>"]
fn b0_debug_scene() {
    let s = scene(&std::env::var("OPERANDI_B0_SCENE").unwrap_or("two-cubes".into()));
    let sharp = sharp::build(&s.sources, s.level).unwrap();
    let started = Instant::now();
    let d = if std::env::var("OPERANDI_B0_METHOD").as_deref() == Ok("B") {
        strip_bevel::blend(&sharp, s.width, s.profile, s.rows).unwrap()
    } else {
        strip_builder::blend(&sharp, s.width, s.profile, s.rows)
    };
    eprintln!("strip {:.3}s", started.elapsed().as_secs_f64());
    let r = check::st10(&d.mesh, s.width, Some(&d.kind));
    eprintln!("{:?}\n{:?}", r, d.problems);
    // Where are tiny edges?
    let m = &d.mesh;
    let mut seen = std::collections::HashSet::new();
    for (f, face) in m.faces.iter().enumerate() {
        let v = &face.vertices;
        for i in 0..v.len() {
            let (a, b) = (v[i], v[(i + 1) % v.len()]);
            let l = m.vertices[a as usize].distance(m.vertices[b as usize]);
            if l < s.width / 1000. && seen.insert((a.min(b), a.max(b))) && seen.len() < 12 {
                let users: Vec<(usize, u8, usize)> = m
                    .faces
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| g.vertices.contains(&a) || g.vertices.contains(&b))
                    .map(|(g, x)| (g, d.kind[g], x.vertices.len()))
                    .collect();
                eprintln!(
                    "short {a}-{b} len {l:e} in face {f} kind {} users {users:?}",
                    d.kind[f]
                );
            }
        }
    }
}
