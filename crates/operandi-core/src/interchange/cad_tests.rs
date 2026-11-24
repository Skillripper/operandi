//! STEP and IGES export (cad.rs, step.rs, iges.rs).
use super::*;
use crate::PolygonRepresentation;
use std::fs;

fn scene_of(meshes: Vec<(&str, Mesh)>) -> Scene {
    let mut scene = Scene::default();
    for (name, mesh) in meshes {
        scene.new_mesh().unwrap();
        let item = scene.items.last_mut().unwrap();
        item.name = name.into();
        item.mesh = mesh.into();
    }
    scene
}

fn cube() -> Mesh {
    Mesh::box_mesh(DVec3::ZERO, DVec3::ONE, 1).unwrap()
}

fn tab(mut mesh: Mesh) -> Mesh {
    mesh.attributes.representations = vec![PolygonRepresentation::Subdivs; mesh.faces.len()];
    mesh
}

/// A one-quad plane, open on every side.
fn plane() -> Mesh {
    let mut mesh = cube();
    mesh.faces.truncate(1);
    mesh
}

fn export_text(scene: &Scene, format: ExportFormat) -> (String, ExportReport) {
    let dir = std::env::temp_dir().join(format!("operandi-cad-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = dir.join(format!("out{n}.{}", format.extension()));
    let report = export(&path, scene, format).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    fs::remove_file(&path).unwrap();
    (text, report)
}

fn count(text: &str, entity: &str) -> usize {
    text.matches(&format!("={entity}(")).count()
}

/// Every `#n` a STEP file refers to is defined, once.
fn references_resolve(text: &str) {
    let mut defined = std::collections::BTreeSet::new();
    for line in text.lines().filter(|l| l.starts_with('#')) {
        let id: u64 = line[1..line.find('=').unwrap()].parse().unwrap();
        assert!(defined.insert(id), "#{id} defined twice");
    }
    for line in text.lines().filter(|l| l.starts_with('#')) {
        let body = &line[line.find('=').unwrap()..];
        for part in body.split('#').skip(1) {
            let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
            let id: u64 = digits.parse().unwrap();
            assert!(defined.contains(&id), "#{id} used but never defined");
        }
    }
}

#[test]
fn a_tab_cube_is_a_closed_solid_of_exact_patches() {
    let (text, report) = export_text(&scene_of(vec![("Box", tab(cube()))]), ExportFormat::Step);
    assert!(text.starts_with("ISO-10303-21;"));
    assert!(text.contains("AUTOMOTIVE_DESIGN"));
    assert!(text.contains("SI_UNIT(.MILLI.,.METRE.)"));
    // One patch per cage corner: 6 quads × 4.
    assert_eq!(count(&text, "B_SPLINE_SURFACE_WITH_KNOTS"), 24);
    assert_eq!(count(&text, "ADVANCED_FACE"), 24);
    assert_eq!(report.polygons, 24);
    // Each patch side is shared: 24 × 4 / 2 edges, all quadratic curves.
    assert_eq!(count(&text, "EDGE_CURVE"), 48);
    assert_eq!(count(&text, "B_SPLINE_CURVE_WITH_KNOTS"), 48);
    assert_eq!(count(&text, "CLOSED_SHELL"), 1);
    assert_eq!(count(&text, "MANIFOLD_SOLID_BREP"), 1);
    assert_eq!(count(&text, "OPEN_SHELL"), 0);
    assert!(text.contains("MANIFOLD_SOLID_BREP('Box'"));
    references_resolve(&text);
    assert!(report.notes.is_empty(), "{:?}", report.notes);
}

#[test]
fn a_plain_cube_is_a_solid_of_planes_in_millimetres_z_up() {
    let mut mesh = cube();
    // 1 m up in the scene (Y) is 1000 mm up in the file (Z).
    for p in &mut mesh.vertices {
        p.y += 1.0;
    }
    let (text, _) = export_text(&scene_of(vec![("Box", mesh)]), ExportFormat::Step);
    assert_eq!(count(&text, "PLANE"), 6);
    assert_eq!(count(&text, "EDGE_CURVE"), 12);
    assert_eq!(count(&text, "LINE"), 12);
    assert_eq!(count(&text, "VERTEX_POINT"), 8);
    assert_eq!(count(&text, "MANIFOLD_SOLID_BREP"), 1);
    assert!(
        text.contains("CARTESIAN_POINT('',(500.0,-500.0,1500.0))"),
        "{text}"
    );
    references_resolve(&text);
}

#[test]
fn open_pieces_are_surfaces_beside_the_solids() {
    let scene = scene_of(vec![("Box", tab(cube())), ("Sheet", tab(plane()))]);
    let (text, report) = export_text(&scene, ExportFormat::Step);
    assert_eq!(count(&text, "MANIFOLD_SOLID_BREP"), 1);
    assert_eq!(count(&text, "OPEN_SHELL"), 1);
    assert_eq!(count(&text, "SHELL_BASED_SURFACE_MODEL"), 1);
    assert_eq!(count(&text, "SHAPE_REPRESENTATION_RELATIONSHIP"), 1);
    assert_eq!(count(&text, "B_SPLINE_SURFACE_WITH_KNOTS"), 28);
    assert!(report.notes.iter().any(|n| n.contains("1 open piece")));
    references_resolve(&text);
}

#[test]
fn bent_quads_split_and_catmull_clark_polygons_become_facets() {
    let mut bent = plane();
    let n = bent.normal(0);
    bent.vertices[bent.faces[0].vertices[0] as usize] += n * 0.25;
    let (text, report) = export_text(&scene_of(vec![("Bent", bent)]), ExportFormat::Step);
    assert_eq!(count(&text, "PLANE"), 2);
    assert!(report.notes.iter().any(|n| n.contains("1 non-planar")));

    let mut smooth = cube();
    smooth.attributes.representations =
        vec![PolygonRepresentation::CatmullClark; smooth.faces.len()];
    let (text, report) = export_text(&scene_of(vec![("CC", smooth)]), ExportFormat::Step);
    // Render level 2: 6 × 4 × 4 facets, each slightly bent, so two
    // triangles each; still one closed solid.
    assert_eq!(count(&text, "ADVANCED_FACE"), 192);
    assert_eq!(count(&text, "MANIFOLD_SOLID_BREP"), 1);
    assert!(report.notes.iter().any(|n| n.contains("6 Catmull-Clark")));
}

#[test]
fn nothing_visible_is_refused_and_hidden_items_are_left_out() {
    let mut scene = scene_of(vec![("Box", cube())]);
    scene.items[0].visible = false;
    let path = std::env::temp_dir().join(format!("operandi-cad-none-{}.step", std::process::id()));
    assert!(export(&path, &scene, ExportFormat::Step).is_err());
    assert!(!path.exists(), "a refused export writes nothing");
    let mut scene = scene_of(vec![("Box", cube()), ("Hidden", cube())]);
    scene.items[1].visible = false;
    let (text, report) = export_text(&scene, ExportFormat::Step);
    assert_eq!(count(&text, "MANIFOLD_SOLID_BREP"), 1);
    assert!(report.notes.iter().any(|n| n.contains("1 hidden")));
}

#[test]
fn iges_holds_one_bspline_surface_per_face_in_fixed_columns() {
    let scene = scene_of(vec![("Box", tab(cube())), ("Plain", cube())]);
    let (text, _) = export_text(&scene, ExportFormat::Iges);
    for line in text.lines() {
        assert_eq!(line.len(), 80, "{line:?}");
    }
    let letters: String = text.lines().map(|l| &l[72..73]).collect();
    assert!(letters.starts_with('S') && letters.ends_with('T'));
    // 24 patches and 6 convex quads: two directory lines each.
    let directory = text.lines().filter(|l| &l[72..73] == "D").count();
    assert_eq!(directory, 2 * 30);
    let tail = text.lines().last().unwrap();
    assert!(tail.contains(&format!("D{:>7}", directory)), "{tail}");
    assert!(text.contains("2HMM"));
    // Every entity is a 128 (rational B-spline surface).
    for line in text.lines().filter(|l| &l[72..73] == "D").step_by(2) {
        assert_eq!(line[..8].trim(), "128");
    }
}

#[test]
fn short_extensions_and_script_names_pick_the_cad_formats() {
    assert_eq!(export_format_of_path("a.stp"), Some(ExportFormat::Step));
    assert_eq!(export_format_of_path("a.STEP"), Some(ExportFormat::Step));
    assert_eq!(export_format_of_path("a.igs"), Some(ExportFormat::Iges));
    assert_eq!(script_export_format("step"), Some(ExportFormat::Step));
    assert_eq!(script_export_format("IGES"), Some(ExportFormat::Iges));
}

#[test]
fn step_reals_and_strings_follow_part_21() {
    assert_eq!(step::real(1.0), "1.0");
    assert_eq!(step::real(-0.0), "0.0");
    assert_eq!(step::real(1e-7), "1.E-7");
    assert_eq!(step::real(2.5e21), "2.5E21");
    assert_eq!(step::text("it's"), "'it''s'");
    assert_eq!(step::text("Würfel"), "'W\\X2\\00FC\\X0\\rfel'");
}

/// Writes sample files for checking in a CAD reader:
/// `OPERANDI_CAD_SAMPLES=<folder> cargo test --locked --workspace -- cad_tests::write_samples --ignored`.
#[test]
#[ignore]
fn write_samples() {
    let dir = std::path::PathBuf::from(std::env::var("OPERANDI_CAD_SAMPLES").unwrap());
    fs::create_dir_all(&dir).unwrap();
    let mut creased = tab(cube());
    creased.attributes.creases = vec![crate::MeshCrease {
        a: creased.faces[0].vertices[0],
        b: creased.faces[0].vertices[1],
        weight: 1.0,
    }];
    // A cup: the cube with one side left open.
    let mut cup = cube();
    cup.faces.remove(0);
    let cup = tab(cup);
    let samples = [
        ("tab-cube", scene_of(vec![("Box", tab(cube()))])),
        ("tab-creased", scene_of(vec![("Creased", creased)])),
        ("tab-open", scene_of(vec![("Cup", cup)])),
        ("plain-cube", scene_of(vec![("Plain", cube())])),
        (
            "mixed",
            scene_of(vec![("Box", tab(cube())), ("Sheet", tab(plane()))]),
        ),
    ];
    for (name, scene) in samples {
        // The closed samples' exact volume in mm³: a third of the flux of
        // the position through the surface, by Gauss quadrature (exact for
        // these polynomial patches), plus the flat facets'.
        let item = &scene.items[0];
        let pieces = crate::subdivision::nurbs::cad_pieces(
            &item.mesh,
            &item.subdivision.clone().unwrap_or_default(),
        )
        .unwrap();
        let gauss = [
            (0.5 - 0.6f64.sqrt() / 2.0, 5.0 / 18.0),
            (0.5, 8.0 / 18.0),
            (0.5 + 0.6f64.sqrt() / 2.0, 5.0 / 18.0),
        ];
        let bern = |t: f64| [(1.0 - t) * (1.0 - t), 2.0 * t * (1.0 - t), t * t];
        let slope = |t: f64| [-2.0 * (1.0 - t), 2.0 - 4.0 * t, 2.0 * t];
        let mut volume = 0.0;
        for p in &pieces.patches {
            for &(u, wu) in &gauss {
                for &(v, wv) in &gauss {
                    let (bu, bv, du, dv) = (bern(u), bern(v), slope(u), slope(v));
                    let (mut at, mut tu, mut tv) = (DVec3::ZERO, DVec3::ZERO, DVec3::ZERO);
                    for a in 0..3 {
                        for b in 0..3 {
                            at += p.grid[a][b] * bu[a] * bv[b];
                            tu += p.grid[a][b] * du[a] * bv[b];
                            tv += p.grid[a][b] * bu[a] * dv[b];
                        }
                    }
                    volume += wu * wv * at.dot(tu.cross(tv)) / 3.0;
                }
            }
        }
        let facets = &pieces.facets;
        for f in 0..facets.faces.len() {
            for t in facets.surface_triangles(f).unwrap() {
                let v = &facets.faces[f].vertices;
                let p = |c: usize| facets.vertices[v[t[c]] as usize];
                volume += p(0).dot(p(1).cross(p(2))) / 6.0;
            }
        }
        fs::write(
            dir.join(format!("{name}.volume")),
            format!("{}", volume * 1e9),
        )
        .unwrap();
        for format in [ExportFormat::Step, ExportFormat::Iges] {
            let path = dir.join(format!("{name}.{}", format.extension()));
            export(&path, &scene, format).unwrap();
        }
    }
}
