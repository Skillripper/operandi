//! Chamfer Edit as a Mesh Stack op (`chamfer.edit.item`, TYPE-IDS.md):
//! reworks an existing chamfer or bevel (its strip polygons are the op's
//! Selection): width, segments and roundness, through the direct tool's
//! engine (`chamfer_geometry`). With no Selection the op looks for strips
//! among every polygon. Mode is stored by position (0 Inset, 1 Width;
//! MOQ-25).
use crate::mesh_stack_builtin::{check, choice, count, distance, finite};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{ChamferParameters, Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ChamferEditOp {
    pub offset: f64,
    /// 0 Inset, 1 Width.
    pub mode: u32,
    pub segments: u32,
    /// 1.0 = 100%.
    pub depth: f64,
    pub reversex: bool,
    pub reversey: bool,
    pub source_segs: u32,
    pub abs_size: bool,
    pub swap_side: bool,
}
impl Default for ChamferEditOp {
    fn default() -> Self {
        let p = ChamferParameters::default();
        Self {
            offset: p.offset,
            mode: 0,
            segments: p.segments,
            depth: p.depth / 100.,
            reversex: p.reverse_x,
            reversey: p.reverse_y,
            source_segs: p.source_segments,
            abs_size: p.absolute,
            swap_side: p.swap_side,
        }
    }
}
impl TypedOp for ChamferEditOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        finite(&[self.offset, self.depth], "Offset and Depth")?;
        if !(1..=64).contains(&self.segments) || !(1..=64).contains(&self.source_segs) {
            return Err("Segments and Source Segments must be 1 to 64".into());
        }
        let width_mode = match self.mode {
            0 => false,
            1 => true,
            _ => return Err("Mode must be 0 (Inset) or 1 (Width)".into()),
        };
        let p = ChamferParameters {
            offset: self.offset,
            segments: self.segments,
            source_segments: self.source_segs,
            depth: self.depth * 100.,
            width_mode,
            absolute: self.abs_size,
            swap_side: self.swap_side,
            reverse_x: self.reversex,
            reverse_y: self.reversey,
        };
        crate::chamfer::chamfer_geometry(ctx.input, &ctx.chosen_or_all()?, p)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(Typed::<ChamferEditOp>::new(
        "chamfer.edit.item",
        "Chamfer Edit",
        "Edge",
        true,
        vec![
            distance("offset", "Offset"),
            choice("mode", "Mode", &["Inset", "Width"]),
            count("segments", "Segments", 1, 64),
            Field::new("depth", "Depth", FieldKind::Percent),
            check("reversex", "Reverse Direction"),
            check("reversey", "Reverse Depth"),
            count("sourceSegs", "Source Segments", 1, 64),
            check("absSize", "Absolute Size"),
            check("swapSide", "Swap Editing Side"),
        ],
    ))]
}
