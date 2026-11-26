//! Loop Slice as a Mesh Stack op (`poly.loopSlice.item`, TYPE-IDS.md):
//! slices loops across the rings of the op's edges, through the direct
//! tool's engine (`loop_slice_cut`).
//!
//! Each chosen edge names one ring; a ring is cut once however many of
//! its edges are chosen. With no Selection every ring is cut (the edge
//! ops' "nothing chosen = whole mesh" rule). With Count above 1 the
//! slices are spaced evenly and Position is set aside, as when the tool's
//! Count changes (provisional, MOQ-35). Mode is stored by position
//! (0 Free, 1 Uniform, 2 Symmetry; MOQ-25). The slider-bar and profile
//! channels are saved and kept, not used.
use crate::mesh_stack_builtin::{check, choice, count, distance, finite};
use crate::mesh_stack_tools::{chosen_edges, hidden};
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{LoopSlice, LoopSliceMode, Mesh, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LoopSliceOp {
    pub count: u32,
    /// Where a single slice sits across its ring (1.0 = 100%).
    pub pos: f64,
    /// 0 Free, 1 Uniform, 2 Symmetry.
    pub mode: u32,
    /// Slice Selected.
    pub select: bool,
    /// Keep Quads.
    pub quad: bool,
    /// Slice N-gon.
    pub ngon: bool,
    /// Inset.
    pub depth: f64,
    /// Preserve Curvature.
    pub curvature: bool,
    pub tension: f64,
    pub split: bool,
    pub caps: bool,
    pub gap: f64,
    // Saved and kept, not used.
    pub edit: u32,
    pub curr: u32,
    pub sx: u32,
    pub sy: u32,
    pub sw: u32,
    pub reversex: bool,
    pub reversey: bool,
    pub aspect: bool,
    pub new_sel: bool,
    pub draw_sliced: bool,
}
impl Default for LoopSliceOp {
    fn default() -> Self {
        let t = LoopSlice::default();
        Self {
            count: t.count() as u32,
            pos: t.positions[0],
            mode: 0,
            select: t.slice_selected,
            quad: t.keep_quads,
            ngon: t.slice_ngon,
            depth: t.inset,
            curvature: t.preserve_curvature,
            tension: t.tension,
            split: t.split,
            caps: t.cap_sections,
            gap: t.gap,
            edit: 0,
            curr: 0,
            sx: t.slider_x as u32,
            sy: t.slider_y as u32,
            sw: t.length as u32,
            reversex: false,
            reversey: false,
            aspect: true,
            new_sel: t.select_new,
            draw_sliced: true,
        }
    }
}
impl TypedOp for LoopSliceOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        finite(
            &[self.pos, self.depth, self.tension, self.gap],
            "Loop Slice values",
        )?;
        if !(1..=100).contains(&self.count) {
            return Err("Loop Slice Count must be 1 to 100".into());
        }
        let mode = match self.mode {
            0 => LoopSliceMode::Free,
            1 => LoopSliceMode::Uniform,
            2 => LoopSliceMode::Symmetry,
            _ => {
                return Err("Loop Slice Mode must be 0 (Free), 1 (Uniform) or 2 (Symmetry)".into());
            }
        };
        let mut t = LoopSlice {
            mode,
            positions: vec![self.pos],
            slice_selected: self.select,
            keep_quads: self.quad,
            slice_ngon: self.ngon,
            preserve_curvature: self.curvature,
            tension: self.tension,
            inset: self.depth,
            split: self.split,
            cap_sections: self.caps,
            gap: self.gap,
            rings: chosen_edges(ctx),
            ..LoopSlice::default()
        };
        t.set_count(self.count as usize);
        t.normalize(0);
        if t.rings.is_empty() {
            return Ok(ctx.input.clone());
        }
        let edges_chosen = ctx.selection.is_some() && ctx.element == SelElement::Edge;
        let (mesh, _, _) = crate::slice::loop_slice_cut(ctx.input, &t, &[], edges_chosen)?;
        Ok(mesh)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let mut fields = vec![
        count("count", "Count", 1, 100),
        Field::new("pos", "Position", FieldKind::Percent),
        choice("mode", "Mode", &["Free", "Uniform", "Symmetry"]),
        check("select", "Slice Selected"),
        check("quad", "Keep Quads"),
        check("ngon", "Slice N-gon"),
        distance("depth", "Inset"),
        check("curvature", "Preserve Curvature"),
        Field::new("tension", "Tension", FieldKind::Percent),
        check("split", "Split"),
        check("caps", "Cap Sections"),
        distance("gap", "Gap"),
    ];
    fields.extend(
        [
            "edit",
            "curr",
            "sx",
            "sy",
            "sw",
            "reversex",
            "reversey",
            "aspect",
            "newSel",
            "drawSliced",
        ]
        .map(hidden),
    );
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<LoopSliceOp>::new(
            "poly.loopSlice.item",
            "Loop Slice",
            "Edit",
            true,
            fields,
        ))],
        SelElement::Edge,
    )
}
