//! Prototype D, now MeshBlend's strip builder (`meshblend::strips`):
//! the B0 comparison calls it through here.
use super::Blended;
use super::sharp::Sharp;

/// Prototype D on a sharp result.
pub fn blend(sharp: &Sharp, width: f64, profile: f64, rows: u32) -> Blended {
    let joined = crate::meshblend::strips::Joined {
        pos: sharp.pos.clone(),
        tris: sharp.tris.clone(),
        source: sharp.source.clone(),
        face: sharp.face.clone(),
        names: sharp.names.clone(),
        items: (0..sharp.names.len() as u64).collect(),
    };
    let s = crate::meshblend::strips::build(&joined, width, profile, rows);
    Blended {
        mesh: s.mesh,
        kind: s.kind,
        source: s.source,
        problems: s.problems,
    }
}
