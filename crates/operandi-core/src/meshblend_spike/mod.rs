//! MeshBlend B0 spike (meshblend-2026-11-24, PLAN.md phase B0): the test
//! corpus, the ST-10 / ST-09 checker, and two strip prototypes compared on
//! the same scenes so Jason can pick the strip method (Q07).
//!
//! Nothing here is reachable from the UI. The pipeline both prototypes
//! share:
//!
//! 1. [`sharp`]: every source turned into its smooth surface (SB-20), then
//!    one exact Manifold boolean per branch (SB-13), keeping for every
//!    result triangle the source and source polygon it came from.
//! 2. A strip method on that sharp result:
//!    * [`strip_bevel`] (prototype B): rebuild polygons from provenance,
//!      pick the join edges, run our Edge Chamfer on them.
//!    * [`strip_builder`] (prototype D): our own builder: cut a band of the
//!      chosen width away on each side of every join, loft a quad band with
//!      the profile between the two cut lines, patch the corners.
//! 3. [`check`]: the ST-10 topology gate and the ST-09 repeatability test.
//!
//! [`picture`] draws results to PNG for the comparison page; [`corpus`]
//! holds the scenes (BLD-C01..C27 inputs and the FAILURES cases).
//!
//! B1 keeps the corpus, checker and sharp stage; only the losing strip
//! method is thrown away.
pub use crate::meshblend::check;
pub mod corpus;
pub mod picture;
pub mod sharp;
pub mod strip_bevel;
pub mod strip_builder;

use crate::Face;

/// A plain polygon with the given corners.
pub(crate) fn poly(vertices: Vec<u32>) -> Face {
    Face {
        vertices,
        uv: None,
        material: String::new(),
        smooth: false,
        part: None,
        material_label: None,
    }
}

/// What a strip method made: the mesh, which polygons are strip (or corner
/// patch) polygons, and why any strip couldn't be built.
#[derive(Clone, Debug, Default)]
pub struct Blended {
    pub mesh: crate::Mesh,
    /// Per polygon: 0 = surface, 1 = strip, 2 = corner patch.
    pub kind: Vec<u8>,
    /// Per polygon: the canonical source it lies on (strips: the first
    /// side's source).
    pub source: Vec<u32>,
    /// Strips or corners that couldn't be built, in plain words.
    pub problems: Vec<String>,
}
