//! Older Modo scenes (format 4.x and older, old-lxo-2026-11-25, GitHub #20):
//! tests T1-T9, built in memory from the LG-01 fixture with its first
//! layer's geometry replaced by an N x N quad grid (as the spec's
//! make_big_legacy_grid.py does). Acceptance criteria:
//!
//! 1. OLD-02: UV and polygon tag entries never count as metadata, so big
//!    grids with UVs and tags open (T1, T2) and match the same bytes read as
//!    format 16.0 (T3) point for point, corner for corner, UV for UV.
//! 2. Files that already opened still do (T4-T6).
//! 3. OLD-04: an unknown plug-in item with 1.1 M sub-records is skipped with
//!    one note; the meshes open (T8).
//! 4. OLD-05: format 1.x gets the plain Modo 202 message (T9).
//! 5. OLD-06: a 1.21 M-polygon old grid opens within the app's memory
//!    allowance (T7, an ignored probe: it takes a while in debug builds).
use operandi_core::{Mesh, lxo};
use std::path::PathBuf;

fn fixture() -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/lxo-legacy/LG-01-v40-appv9-tail70.lxo"),
    )
    .unwrap()
}

fn chunk(out: &mut Vec<u8>, id: &[u8], payload: &[u8]) {
    out.extend_from_slice(id);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    if payload.len() % 2 == 1 {
        out.push(0);
    }
}
fn vx(out: &mut Vec<u8>, i: u32) {
    if i < 0xFF00 {
        out.extend_from_slice(&(i as u16).to_be_bytes());
    } else {
        out.extend_from_slice(&(0xFF00_0000 | i).to_be_bytes());
    }
}
fn f32s(out: &mut Vec<u8>, v: &[f32]) {
    for x in v {
        out.extend_from_slice(&x.to_be_bytes());
    }
}

struct Grid {
    n: u32,
    uv: bool,
    part: bool,
    seams: bool,
    /// An unknown plug-in ITEM with this many tiny sub-records (T8).
    plug_in_records: usize,
}

/// The fixture with its first layer's geometry replaced by the grid.
fn grid(g: &Grid) -> Vec<u8> {
    let d = fixture();
    let mut body = Vec::new();
    let (n, w) = (g.n, g.n + 1);
    let mut pts = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            f32s(&mut pts, &[i as f32 / n as f32, 0., j as f32 / n as f32]);
        }
    }
    let quads: Vec<[u32; 4]> = (0..n)
        .flat_map(|j| (0..n).map(move |i| j * w + i))
        .map(|a| [a, a + 1, a + 1 + w, a + w])
        .collect();
    let mut pols = b"FACE".to_vec();
    for q in &quads {
        pols.extend_from_slice(&4u16.to_be_bytes());
        for &k in q {
            vx(&mut pols, k);
        }
    }
    let mut vm = b"TXUV".to_vec();
    vm.extend_from_slice(&2u16.to_be_bytes());
    vm.extend_from_slice(b"Texture\0");
    for k in 0..w * w {
        vx(&mut vm, k);
        f32s(
            &mut vm,
            &[(k % w) as f32 / n as f32, (k / w) as f32 / n as f32],
        );
    }
    let tag = |kind: &[u8], t: u16| {
        let mut p = kind.to_vec();
        for k in 0..quads.len() as u32 {
            vx(&mut p, k);
            p.extend_from_slice(&t.to_be_bytes());
        }
        p
    };
    let mut layer = 0;
    let mut p = 12;
    while p < d.len() {
        let id = &d[p..p + 4];
        let len = u32::from_be_bytes(d[p + 4..p + 8].try_into().unwrap()) as usize;
        let payload = &d[p + 8..p + 8 + len];
        p += 8 + len + (len & 1);
        if id == b"LAYR" {
            layer += 1;
        }
        if layer == 1 && [&b"PNTS"[..], b"VMAP", b"POLS", b"VMAD", b"PTAG"].contains(&id) {
            match id {
                b"PNTS" => chunk(&mut body, b"PNTS", &pts),
                b"VMAP" if g.uv => chunk(&mut body, b"VMAP", &vm),
                b"POLS" => {
                    chunk(&mut body, b"POLS", &pols);
                    chunk(&mut body, b"PTAG", &tag(b"MATR", 0));
                    if g.part {
                        chunk(&mut body, b"PTAG", &tag(b"PART", 1));
                    }
                    if g.seams {
                        let mut v = b"TXUV".to_vec();
                        v.extend_from_slice(&2u16.to_be_bytes());
                        v.extend_from_slice(b"Texture\0");
                        for (f, q) in quads.iter().enumerate() {
                            for &k in q {
                                vx(&mut v, k);
                                vx(&mut v, f as u32);
                                f32s(&mut v, &[0.5, 0.5]);
                            }
                        }
                        chunk(&mut body, b"VMAD", &v);
                    }
                }
                _ => {}
            }
            continue;
        }
        chunk(&mut body, id, payload);
    }
    if g.plug_in_records > 0 {
        let mut item = b"octaneRenderTarget\0\0".to_vec();
        item.extend_from_slice(b"Octane\0\0");
        item.extend_from_slice(&900_001u32.to_be_bytes());
        for _ in 0..g.plug_in_records {
            item.extend_from_slice(b"ZZZZ\0\x02\0\0");
        }
        chunk(&mut body, b"ITEM", &item);
    }
    let mut out = b"FORM".to_vec();
    out.extend_from_slice(&(body.len() as u32 + 4).to_be_bytes());
    out.extend_from_slice(b"LXOB");
    out.extend_from_slice(&body);
    out
}

/// The same bytes with another VRSN number.
fn with_version(mut bytes: Vec<u8>, major: u32, minor: u32) -> Vec<u8> {
    let at = bytes.windows(4).position(|w| w == b"VRSN").unwrap() + 8;
    bytes[at..at + 4].copy_from_slice(&major.to_be_bytes());
    bytes[at + 4..at + 8].copy_from_slice(&minor.to_be_bytes());
    bytes
}

fn open(bytes: &[u8]) -> lxo::Result<lxo::Candidate> {
    let mut go = |_: &str| true;
    lxo::parse(bytes, &mut lxo::Control::new(&mut go))
}

fn polygons(c: &lxo::Candidate) -> usize {
    c.scene.items.iter().map(|i| i.mesh.faces.len()).sum()
}

/// The grid's mesh: the one with the most polygons.
fn grid_mesh(c: &lxo::Candidate) -> &Mesh {
    &c.scene
        .items
        .iter()
        .max_by_key(|i| i.mesh.faces.len())
        .unwrap()
        .mesh
}

const T1: Grid = Grid {
    n: 400,
    uv: true,
    part: true,
    seams: true,
    plug_in_records: 0,
};

#[test]
fn t1_t3_a_big_grid_with_seam_uvs_opens_like_format_16() {
    let bytes = grid(&T1);
    let old = open(&bytes).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(polygons(&old), 160_001);
    // T3: the control, the same bytes as format 16.0.
    let control = open(&with_version(bytes, 16, 0)).unwrap_or_else(|e| panic!("{e}"));
    let (a, b) = (grid_mesh(&old), grid_mesh(&control));
    assert_eq!(a.vertices, b.vertices);
    let corners = |m: &Mesh| {
        m.faces
            .iter()
            .map(|f| f.vertices.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(corners(a), corners(b));
    let uvs = |m: &Mesh| m.faces.iter().map(|f| f.uv.clone()).collect::<Vec<_>>();
    assert_eq!(uvs(a), uvs(b));
    assert_eq!(a.uv_maps, b.uv_maps);
}

#[test]
fn t2_a_bigger_grid_with_uvs_and_tags_opens() {
    let c = open(&grid(&Grid {
        n: 600,
        seams: false,
        ..T1
    }))
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(polygons(&c), 360_001);
}

#[test]
fn t4_t5_t6_files_that_opened_still_open() {
    for (g, polys) in [
        (Grid { seams: false, ..T1 }, 160_001),
        (Grid { n: 300, ..T1 }, 90_001),
        (
            Grid {
                n: 600,
                uv: false,
                part: false,
                seams: false,
                plug_in_records: 0,
            },
            360_001,
        ),
    ] {
        let c = open(&grid(&g)).unwrap_or_else(|e| panic!("grid {}: {e}", g.n));
        assert_eq!(polygons(&c), polys, "grid {}", g.n);
    }
}

#[test]
fn t8_plug_in_data_is_skipped_with_one_note() {
    let c = open(&grid(&Grid {
        seams: false,
        plug_in_records: 1_100_000,
        ..T1
    }))
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(polygons(&c), 160_001);
    let note = "'octaneRenderTarget' items not imported (not modelling, or plug-in data)";
    assert_eq!(
        c.report.omissions.get(note),
        Some(&1),
        "{:?}",
        c.report.omissions
    );
}

#[test]
fn t9_format_1_is_refused_in_plain_words() {
    let bytes = with_version(grid(&Grid { seams: false, ..T1 }), 1, 2);
    let Err(e) = open(&bytes) else {
        panic!("format 1.2 is refused")
    };
    assert_eq!(
        e.to_string(),
        "This scene was saved by Modo 202 or earlier. Operandi can't read it yet. Re-save it in a newer Modo and try again."
    );
    assert!(e.detail.contains("[LXO format 1.2]"));
}

#[test]
fn old_02_metadata_over_the_limit_is_cut_short_not_refused() {
    // OLD-03: an item of a known kind (a camera) with more sub-records than
    // the limit: the file still opens, with a note.
    let mut bytes = grid(&Grid {
        n: 2,
        uv: false,
        part: false,
        seams: false,
        plug_in_records: 0,
    });
    let mut item = b"camera\0\0".to_vec();
    item.extend_from_slice(b"Busy\0\0");
    item.extend_from_slice(&900_002u32.to_be_bytes());
    for _ in 0..1_000_010 {
        item.extend_from_slice(b"ZZZZ\0\x02\0\0");
    }
    let mut extra = Vec::new();
    chunk(&mut extra, b"ITEM", &item);
    bytes.extend_from_slice(&extra);
    let len = bytes.len() as u32 - 8;
    bytes[4..8].copy_from_slice(&len.to_be_bytes());
    let c = open(&bytes).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(polygons(&c), 5);
    assert!(
        c.report.notes.iter().any(|n| n.contains("over a million")),
        "{:?}",
        c.report.notes
    );
}

/// T7 / OLD-06: a 1,100 x 1,100 grid (1.21 M polygons) opens with the
/// app's allowance (half the free memory, at least 1 GiB; 4 GiB here).
/// Run with `cargo test --release --workspace -- t7_ --ignored`.
#[test]
#[ignore]
fn t7_a_1_2_million_polygon_old_grid_opens() {
    let bytes = grid(&Grid {
        n: 1100,
        uv: false,
        part: false,
        seams: false,
        plug_in_records: 0,
    });
    let mut go = |_: &str| true;
    let mut control = lxo::Control::new(&mut go);
    control.limits.working_bytes = 4 << 30;
    let c = lxo::parse(&bytes, &mut control).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(polygons(&c), 1_210_001);
    eprintln!("T7 working memory charged: {} MiB", control.used >> 20);
    let mut go = |_: &str| true;
    let mut new = lxo::Control::new(&mut go);
    new.limits.working_bytes = 4 << 30;
    lxo::parse(&with_version(bytes, 16, 0), &mut new).unwrap();
    eprintln!("T7 as format 16.0: {} MiB", new.used >> 20);
}
