//! Text as a Mesh Stack op (`prim.text.item`, TYPE-IDS.md): the Text
//! tool's flat letter polygons (`text_geometry`, TXT-xx) added to the
//! op's input, with the tool's values.
//!
//! Provisional (MOQ-49): Vertical Location and Horizontal Justification
//! are stored by position in the tool's order (Top, Middle, Bottom; Left,
//! Center, Right); the default font is Operandi's own (the reference's is
//! a system font). Split and Output Type keep their only built choice
//! (by line, polygons) and aren't shown. Empty text adds nothing.
use crate::mesh_stack_builtin::{check, choice, distance};
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, Typed, TypedOp};
use crate::{Mesh, Result, TextJustification, TextLocation, TextParameters};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TextOp {
    pub text: String,
    pub font: String,
    pub size: f64,
    pub kerning: f64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    /// 0 Top, 1 Middle, 2 Bottom.
    pub location: u32,
    /// 0 Left, 1 Center, 2 Right.
    pub justification: u32,
    /// 0 by line (the only one built).
    pub split: u32,
    pub axis: u32,
    pub uvs: bool,
    pub flip: bool,
    /// 0 Polygon (the only one built).
    #[serde(rename = "type")]
    pub kind: u32,
}
impl Default for TextOp {
    fn default() -> Self {
        Self {
            text: String::new(),
            font: crate::text::DEFAULT_FONT.into(),
            size: 1.,
            kerning: 0.,
            pos_x: 0.,
            pos_y: 0.,
            pos_z: 0.,
            location: 2,
            justification: 0,
            split: 0,
            axis: 2,
            uvs: false,
            flip: false,
            kind: 0,
        }
    }
}
impl TypedOp for TextOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        let location = match self.location {
            0 => TextLocation::Top,
            1 => TextLocation::Middle,
            2 => TextLocation::Bottom,
            _ => return Err("Vertical Location must be 0 (Top) to 2 (Bottom)".into()),
        };
        let justification = match self.justification {
            0 => TextJustification::Left,
            1 => TextJustification::Center,
            2 => TextJustification::Right,
            _ => return Err("Horizontal Justification must be 0 (Left) to 2 (Right)".into()),
        };
        if self.axis > 2 {
            return Err("Axis must be 0 (X), 1 (Y) or 2 (Z)".into());
        }
        let p = TextParameters {
            text: self.text.clone(),
            font: self.font.clone(),
            size: self.size,
            tracking: self.kerning,
            position: DVec3::new(self.pos_x, self.pos_y, self.pos_z),
            location,
            justification,
            axis: self.axis as usize,
            flip: self.flip,
            make_uvs: self.uvs,
        };
        p.validate()?;
        let letters = crate::text::text_geometry(&p)?;
        let mut out = ctx.input.clone();
        let first = out.faces.len() as u32;
        crate::append_mesh(&mut out, letters)?;
        let n = out.faces.len() as u32;
        ctx.tag(&mut out, "text", first..n);
        Ok(out)
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    vec![Arc::new(
        Typed::<TextOp>::new(
            "prim.text.item",
            "Text",
            "Create/Primitives",
            false,
            vec![
                Field::new("text", "Text", FieldKind::Text),
                Field::new("font", "Font", FieldKind::Text),
                distance("size", "Size"),
                distance("kerning", "Tracking"),
                distance("posX", "Position X"),
                distance("posY", "Position Y"),
                distance("posZ", "Position Z"),
                choice(
                    "location",
                    "Vertical Location",
                    &["Top", "Middle", "Bottom"],
                ),
                choice(
                    "justification",
                    "Horizontal Justification",
                    &["Left", "Center", "Right"],
                ),
                hidden("split"),
                choice("axis", "Axis", &["X", "Y", "Z"]),
                check("uvs", "Make UVs"),
                check("flip", "Flip Polygons"),
                hidden("type"),
            ],
        )
        .with_groups(&[("text", "Text")]),
    )]
}
