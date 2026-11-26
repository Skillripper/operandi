//! A mesh as exact CAD surfaces, for STEP and IGES export
//! (`interchange/step.rs`, `interchange/iges.rs`).
//!
//! Subdivision (Tab) polygons are, by their definition (SDV-09 v2), one
//! Catmull-Clark step and then a biquadratic Bézier patch per quad. A
//! Bézier patch is a NURBS patch, so each Tab patch becomes a degree-2
//! B-spline surface with no approximation: the file holds the very surface
//! the viewport samples. Neighbouring patches share their corner and side
//! controls, so their boundary curves match exactly and a CAD reader can
//! sew them into one shell (a solid when the surface is closed).
//!
//! Catmull-Clark (Shift+Tab) polygons have no finite exact form near
//! extraordinary points; they come out as flat facets of their render-level
//! surface, like Face polygons. Lines are left out.
use super::{
    PolygonRepresentation, SubdivisionSettings, catmull_clark_step, effective_kinds, patch_grids,
};
use crate::{Mesh, Result};
use glam::DVec3;

/// One Tab patch: its 3 × 3 controls `grid[a][b]`, `a` along side 0
/// (corner 0 → 1) and `b` from corner 0 toward corner 3, and its corners'
/// step point numbers (shared with the patches around it).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TabPatch {
    pub grid: [[DVec3; 3]; 3],
    pub corners: [u32; 4],
}
impl TabPatch {
    /// The three controls of side `i` (corner `i` to corner `i + 1`): the
    /// quadratic Bézier the patch's boundary follows there.
    pub fn side(&self, i: usize) -> [DVec3; 3] {
        let [a0, b0] = CORNER_CELLS[i];
        let [a1, b1] = CORNER_CELLS[(i + 1) % 4];
        let mid = self.grid[(a0 + a1) / 2][(b0 + b1) / 2];
        [self.grid[a0][b0], mid, self.grid[a1][b1]]
    }
}
/// Grid cell of each patch corner, in corner order.
const CORNER_CELLS: [[usize; 2]; 4] = [[0, 0], [2, 0], [2, 2], [0, 2]];

/// A mesh as CAD pieces.
#[derive(Clone, Debug, Default)]
pub(crate) struct CadPieces {
    /// The Tab polygons' patches.
    pub patches: Vec<TabPatch>,
    /// Flat polygons: Face polygons as they are, then Catmull-Clark
    /// polygons' render-level surface. Points are shared where the
    /// polygons share them.
    pub facets: Mesh,
    /// Cage polygons drawn as Catmull-Clark (written as facets).
    pub catmull_clark_polygons: usize,
}

/// `mesh` as exact Tab patches plus flat facets (see the module comment).
pub(crate) fn cad_pieces(mesh: &Mesh, settings: &SubdivisionSettings) -> Result<CadPieces> {
    let kinds = effective_kinds(mesh);
    let solid = |f: usize| !mesh.is_polyline(f);
    let tab: Vec<usize> = (0..mesh.faces.len())
        .filter(|&f| solid(f) && kinds[f] == PolygonRepresentation::Subdivs)
        .collect();
    let catmull_clark_polygons = (0..mesh.faces.len())
        .filter(|&f| solid(f) && kinds[f] == PolygonRepresentation::CatmullClark)
        .count();
    let patches = tab_patches(mesh, &tab);
    // Everything else as polygons: the display surface at render levels,
    // with Tab polygons kept as their cage (level 0) and then left out.
    let mut flat = settings.clone();
    flat.subdivision_level = 0;
    flat.catmull_clark_level = settings.catmull_clark_render_level;
    let (surface, sources) = super::display_surface(mesh, &flat)?;
    let mut facets = Mesh {
        vertices: surface.vertices.clone(),
        ..Default::default()
    };
    for (f, face) in surface.faces.iter().enumerate() {
        let source = sources[f];
        if solid(source) && kinds[source] != PolygonRepresentation::Subdivs {
            facets.faces.push(face.clone());
        }
    }
    Ok(CadPieces {
        patches,
        facets,
        catmull_clark_polygons,
    })
}

/// The patches of `mesh`'s polygons `source` (all of the Subdivision
/// kind), in polygon then corner order, exactly as `patch_surface` builds
/// them before sampling.
fn tab_patches(mesh: &Mesh, source: &[usize]) -> Vec<TabPatch> {
    if source.is_empty() {
        return Vec::new();
    }
    let net = super::local_net(mesh, source);
    let step = catmull_clark_step(&net, super::BoundaryRule::Smooth, true);
    let grids = patch_grids(&step);
    step.faces
        .iter()
        .zip(grids)
        .map(|(f, grid)| TabPatch {
            grid,
            corners: [f[0], f[1], f[2], f[3]],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subdivision::patches;

    fn tab_cube() -> Mesh {
        let mut mesh = crate::primitive::cube_geometry(DVec3::ZERO, DVec3::ONE).unwrap();
        mesh.attributes.representations = vec![PolygonRepresentation::Subdivs; mesh.faces.len()];
        mesh
    }

    #[test]
    fn tab_patches_match_the_viewport_patches() {
        let mesh = tab_cube();
        let pieces = cad_pieces(&mesh, &SubdivisionSettings::default()).unwrap();
        let net = patches::build(&mesh, 2).unwrap();
        assert_eq!(pieces.patches.len(), net.patches.len());
        assert!(pieces.facets.faces.is_empty());
        for (k, p) in pieces.patches.iter().enumerate() {
            let g = net.grid(k);
            for (mine, theirs) in p.grid.iter().flatten().zip(g.iter().flatten()) {
                {
                    // The viewport keeps f32 controls.
                    assert!(mine.distance(*theirs) < 1e-6, "patch {k}");
                }
            }
        }
    }

    #[test]
    fn patches_pass_through_the_display_surface_samples() {
        // Exact: the display surface's points are these patches sampled.
        let mut mesh = tab_cube();
        mesh.attributes.creases = vec![crate::MeshCrease {
            a: mesh.faces[0].vertices[0],
            b: mesh.faces[0].vertices[1],
            weight: 0.5,
        }];
        let settings = SubdivisionSettings {
            subdivision_level: 3,
            ..Default::default()
        };
        let pieces = cad_pieces(&mesh, &settings).unwrap();
        let (surface, _) = crate::subdivision::display_surface(&mesh, &settings).unwrap();
        let bern = |t: f64| [(1.0 - t) * (1.0 - t), 2.0 * t * (1.0 - t), t * t];
        for p in &pieces.patches {
            for (i, j) in [(0, 0), (1, 2), (3, 1), (2, 2)] {
                let (bu, bv) = (bern(i as f64 / 3.0), bern(j as f64 / 3.0));
                let mut at = DVec3::ZERO;
                for (row, wu) in p.grid.iter().zip(bu) {
                    for (c, wv) in row.iter().zip(bv) {
                        at += *c * wu * wv;
                    }
                }
                let nearest = surface
                    .vertices
                    .iter()
                    .map(|v| v.distance(at))
                    .fold(f64::INFINITY, f64::min);
                assert!(nearest < 1e-12, "sample ({i}, {j}) is {nearest} off");
            }
        }
    }

    #[test]
    fn neighbouring_patches_share_their_side_curves() {
        let pieces = cad_pieces(&tab_cube(), &SubdivisionSettings::default()).unwrap();
        let mut seen: std::collections::BTreeMap<(u32, u32), Vec<[DVec3; 3]>> = Default::default();
        for p in &pieces.patches {
            for i in 0..4 {
                seen.entry(crate::topology::edge_key(
                    p.corners[i],
                    p.corners[(i + 1) % 4],
                ))
                .or_default()
                .push(p.side(i));
            }
        }
        for (key, curves) in seen {
            assert_eq!(curves.len(), 2, "closed cube: side {key:?} has two patches");
            let [a, b] = [curves[0], curves[1]];
            // The same curve, run the other way.
            assert_eq!(a[1], b[1]);
            assert_eq!((a[0], a[2]), (b[2], b[0]));
        }
    }
}
