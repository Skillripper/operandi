//! MeshBlend (specifications meshblend-2026-11-24): live booleans of
//! ordinary mesh items, read as smooth surfaces. The live result is a
//! MeshBlend Item: a mesh item whose Mesh Stack starts with the MeshBlend
//! op (PLAN.md rule 4).
//!
//! B1: the recipe, the evaluator, the op and the commands. B2: rounded
//! strips along every join ([`strips`], the strip builder Jason picked in
//! B0, 2026-11-24), gated by the ST-10 topology check ([`check`]).
pub mod check;
pub mod commands;
pub mod evaluate;
pub mod op;
pub mod recipe;
pub mod strips;

pub use commands::{MeshBlendCommand, recipe_of};
pub use evaluate::{Blend, Input, Note, Origin, evaluate};
pub use op::MESHBLEND_OP;
pub use recipe::{BlendSource, Recipe, Role};
