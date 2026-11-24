//! CAD export (STEP, IGES): the scene's visible meshes as exact surfaces.
//!
//! Subdivision (Tab) polygons are exact biquadratic patches
//! (`subdivision::nurbs`), so they leave as the very surface the viewport
//! draws. Face polygons leave as flat faces, Catmull-Clark polygons as the
//! flat facets of their render-level surface, and lines are left out.
//! Files are written in millimetres with Z up, the CAD convention, whatever
//! the scene's File I/O preferences say.
use super::{ExportFormat, ExportReport, world_mesh, write_atomic};
use crate::subdivision::nurbs::{TabPatch, cad_pieces};
use crate::{Mesh, Result, Scene};
use glam::DVec3;
use std::collections::HashMap;
use std::path::Path;

/// A face of a CAD shell.
#[derive(Clone, Debug)]
pub(super) enum CadFace {
    /// An exact Tab patch.
    Patch(TabPatch),
    /// A flat polygon: its corners, their point numbers in the shell's
    /// space, and its unit normal (the corners run counter-clockwise
    /// around it).
    Polygon {
        points: Vec<DVec3>,
        keys: Vec<u32>,
        normal: DVec3,
    },
}

/// Faces joined by shared edges. Vertex numbers are unique within `space`.
#[derive(Clone, Debug)]
pub(super) struct CadShell {
    pub name: String,
    pub space: usize,
    pub faces: Vec<CadFace>,
    /// Every edge has exactly two faces, running it opposite ways: the
    /// shell bounds a solid.
    pub closed: bool,
}

/// What a CAD file holds.
#[derive(Clone, Debug, Default)]
pub(super) struct CadModel {
    pub name: String,
    pub shells: Vec<CadShell>,
    /// Distance below which points are the same, in millimetres.
    pub tolerance: f64,
}
impl CadModel {
    pub fn faces(&self) -> usize {
        self.shells.iter().map(|s| s.faces.len()).sum()
    }
}

/// Scene metres (Y up) to CAD millimetres (Z up).
fn to_cad(p: DVec3) -> DVec3 {
    DVec3::new(p.x, -p.z, p.y) * 1000.0
}

/// The CAD model of `scene`'s visible meshes.
pub(super) fn model(scene: &Scene, name: &str, notes: &mut Vec<String>) -> Result<CadModel> {
    let mut out = CadModel {
        name: name.to_owned(),
        shells: Vec::new(),
        tolerance: 1e-6,
    };
    let (mut catmull_clark, mut skipped, mut split) = (0, 0, 0);
    for (index, item) in scene
        .items
        .iter()
        .filter(|i| i.visible && i.is_mesh())
        .enumerate()
    {
        let world = world_mesh(scene, item)?;
        let settings = item.subdivision.clone().unwrap_or_default();
        let pieces = cad_pieces(&world.mesh, &settings)?;
        catmull_clark += pieces.catmull_clark_polygons;
        // Patches: corners keyed by step point.
        let patches: Vec<(Vec<(u32, u32)>, CadFace)> = pieces
            .patches
            .into_iter()
            .map(|mut p| {
                for row in &mut p.grid {
                    for c in row {
                        *c = to_cad(*c);
                    }
                }
                let sides = (0..4).map(|i| directed(&p.corners, i)).collect();
                (sides, CadFace::Patch(p))
            })
            .collect();
        out.shells.extend(shells(&item.name, 2 * index, patches));
        let facets = flat_faces(&pieces.facets, &mut skipped, &mut split);
        out.shells.extend(shells(&item.name, 2 * index + 1, facets));
    }
    if catmull_clark > 0 {
        notes.push(format!(
            "{catmull_clark} Catmull-Clark polygon(s) written as flat facets of their render-level surface; only Subdivision (Tab) polygons convert exactly"
        ));
    }
    if split > 0 {
        notes.push(format!(
            "{split} non-planar polygon(s) split into flat triangles"
        ));
    }
    if skipped > 0 {
        notes.push(format!(
            "{skipped} degenerate polygon(s) left out (no area, or a repeated point)"
        ));
    }
    let open = out.shells.iter().filter(|s| !s.closed).count();
    if open > 0 {
        notes.push(format!(
            "{open} open piece(s) written as surfaces; closed pieces are solids"
        ));
    }
    Ok(out)
}

/// Side `i` of a polygon with corner numbers `keys`, as (from, to).
fn directed(keys: &[u32], i: usize) -> (u32, u32) {
    (keys[i], keys[(i + 1) % keys.len()])
}

/// The flat faces of `mesh` in CAD space. A polygon that isn't flat
/// becomes its triangles; one with no area or a repeated point is left
/// out.
fn flat_faces(
    mesh: &Mesh,
    skipped: &mut usize,
    split: &mut usize,
) -> Vec<(Vec<(u32, u32)>, CadFace)> {
    let mut out = Vec::new();
    let push = |keys: Vec<u32>, out: &mut Vec<_>| -> bool {
        let points: Vec<DVec3> = keys
            .iter()
            .map(|&k| to_cad(mesh.vertices[k as usize]))
            .collect();
        let normal = newell(&points);
        let repeated = (0..keys.len()).any(|i| {
            keys[i] == keys[(i + 1) % keys.len()] || points[i] == points[(i + 1) % keys.len()]
        });
        if keys.len() < 3 || normal == DVec3::ZERO || repeated {
            return false;
        }
        let sides = (0..keys.len()).map(|i| directed(&keys, i)).collect();
        out.push((
            sides,
            CadFace::Polygon {
                points,
                keys,
                normal,
            },
        ));
        true
    };
    for (f, face) in mesh.faces.iter().enumerate() {
        let keys = face.vertices.clone();
        let points: Vec<DVec3> = keys
            .iter()
            .map(|&k| to_cad(mesh.vertices[k as usize]))
            .collect();
        if planar(&points) {
            if !push(keys, &mut out) {
                *skipped += 1;
            }
            continue;
        }
        *split += 1;
        let triangles = mesh
            .surface_triangles(f)
            .unwrap_or_else(|_| (1..keys.len() - 1).map(|i| [0, i, i + 1]).collect());
        let mut any = false;
        for t in triangles {
            any |= push(t.iter().map(|&c| keys[c]).collect(), &mut out);
        }
        if !any {
            *skipped += 1;
        }
    }
    out
}

/// Unit normal of a polygon (Newell's method), or zero.
fn newell(points: &[DVec3]) -> DVec3 {
    let mut n = DVec3::ZERO;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        n += DVec3::new(
            (a.y - b.y) * (a.z + b.z),
            (a.z - b.z) * (a.x + b.x),
            (a.x - b.x) * (a.y + b.y),
        );
    }
    n.normalize_or_zero()
}

/// Whether every corner lies on the polygon's plane, to a millionth of its
/// size (a quad off by more than that is drawn bent and splits).
fn planar(points: &[DVec3]) -> bool {
    if points.len() <= 3 {
        return true;
    }
    let n = newell(points);
    let centre = points.iter().copied().sum::<DVec3>() / points.len() as f64;
    let size = points
        .iter()
        .map(|p| p.distance(centre))
        .fold(0.0, f64::max);
    n != DVec3::ZERO
        && points
            .iter()
            .all(|p| (*p - centre).dot(n).abs() <= 1e-6 * size)
}

/// `faces` (each with its directed sides) grouped into shells by shared
/// edges.
fn shells(name: &str, space: usize, faces: Vec<(Vec<(u32, u32)>, CadFace)>) -> Vec<CadShell> {
    if faces.is_empty() {
        return Vec::new();
    }
    // Union-find over faces sharing an edge.
    let mut parent: Vec<usize> = (0..faces.len()).collect();
    fn root(parent: &mut [usize], mut f: usize) -> usize {
        while parent[f] != f {
            parent[f] = parent[parent[f]];
            f = parent[f];
        }
        f
    }
    // Per undirected edge: first face, uses one way, uses the other way.
    let mut uses: HashMap<(u32, u32), (usize, u32, u32)> = HashMap::new();
    for (f, (sides, _)) in faces.iter().enumerate() {
        for &(a, b) in sides {
            let key = (a.min(b), a.max(b));
            let entry = uses.entry(key).or_insert((f, 0, 0));
            if a < b {
                entry.1 += 1;
            } else {
                entry.2 += 1;
            }
            let first = entry.0;
            let (x, y) = (root(&mut parent, f), root(&mut parent, first));
            parent[x] = y;
        }
    }
    let mut open = vec![false; faces.len()];
    for &(first, up, down) in uses.values() {
        if (up, down) != (1, 1) {
            let r = root(&mut parent, first);
            open[r] = true;
        }
    }
    let mut index: HashMap<usize, usize> = HashMap::new();
    let mut out: Vec<CadShell> = Vec::new();
    for (f, (_, face)) in faces.into_iter().enumerate() {
        let r = root(&mut parent, f);
        let k = *index.entry(r).or_insert_with(|| {
            out.push(CadShell {
                name: name.to_owned(),
                space,
                faces: Vec::new(),
                closed: !open[r],
            });
            out.len() - 1
        });
        out[k].faces.push(face);
    }
    out
}

/// The current time as an ISO 8601 UTC stamp (for file headers).
pub(super) fn utc_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (days, rest) = ((secs / 86_400) as i64, secs % 86_400);
    // Civil date from days since 1970-01-01 (Howard Hinnant's method).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}",
        rest / 3600,
        rest / 60 % 60,
        rest % 60
    )
}

/// Write `scene` as STEP or IGES.
pub(super) fn export(path: &Path, scene: &Scene, format: ExportFormat) -> Result<ExportReport> {
    let mut notes = Vec::new();
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Operandi".into());
    let model = model(scene, &name, &mut notes)?;
    if model.shells.is_empty() {
        return Err("Nothing to export: no visible item has polygons".into());
    }
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stamp = utc_stamp();
    let data = match format {
        ExportFormat::Step => super::step::write(&model, &file_name, &stamp)?,
        ExportFormat::Iges => super::iges::write(&model, &file_name, &stamp)?,
        _ => unreachable!("cad::export writes STEP and IGES only"),
    };
    write_atomic(path, &data)?;
    let hidden = scene
        .items
        .iter()
        .filter(|i| !i.visible && i.is_mesh())
        .count();
    if hidden > 0 {
        notes.push(format!("{hidden} hidden item(s) not exported"));
    }
    Ok(ExportReport {
        items: scene
            .items
            .iter()
            .filter(|i| i.visible && i.is_mesh() && !i.mesh.faces.is_empty())
            .count(),
        polygons: model.faces(),
        notes,
    })
}
