//! MeshBlend (specifications meshblend-2026-11-24): live booleans of
//! ordinary mesh items, read as smooth surfaces. The live result is a
//! MeshBlend Item: a mesh item whose Mesh Stack starts with the MeshBlend
//! op (PLAN.md rule 4).
//!
//! Phase B1 (this module): the recipe, the evaluator, the op and the
//! commands, with sharp joins. Strips along the joins come in B2, built
//! with the strip builder Jason picked in B0 (`meshblend_spike`,
//! prototype D, 2026-11-24).
pub mod commands;
pub mod evaluate;
pub mod op;
pub mod recipe;

pub use commands::{MeshBlendCommand, recipe_of};
pub use evaluate::{Blend, Input, Note, Origin, evaluate};
pub use op::MESHBLEND_OP;
pub use recipe::{BlendSource, Recipe, Role};
