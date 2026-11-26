//! Radial Sweep as a Mesh Stack op: the Radial Sweep (Lathe) tool's engine
//! (`duplicate_sweep.rs`, LT-xx) on the op's input. The profile is the
//! chosen edges, or with no Selection the whole mesh (lines and curves
//! sweep as lines and are used up, polygons sweep their outline, LT-10..13).
//! The reference saves it as a Sweep Effector (`effector.sweep.item`) with
//! a Helix generator (`pmodel.helix`) in its Tool Pipe; Operandi keeps the
//! two as one op with flat values, as Curve Extrude does (TYPE-IDS.md).
//!
//! Provisional (MOQ-47): the op's own id `radial.sweep.item`, and the
//! Sweep Effector channels Operandi's sweep doesn't use (Size, Curve
//! Subdivision Tolerance, Offset X/Y, Reverse X/Y, Pivot, Angle) aren't
//! kept. Square is kept, not shown, as on Radial Array. The sweep runs in
//! the mesh's own space.
use crate::duplicate::{Circle, Lathe, Sweep, UvsAround};
use crate::mesh_stack_builtin::{check, choice, distance};
use crate::mesh_stack_tools::hidden;
use crate::meshop_types::{Field, FieldKind, OpContext, OpDef, SelElement, Typed, TypedOp};
use crate::{Mesh, Mode, Result, Scene};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RadialSweepOp {
    /// 0 X, 1 Y, 2 Z.
    pub axis: u32,
    /// Count: steps round the axis.
    pub sides: u32,
    pub vec_x: f64,
    pub vec_y: f64,
    pub vec_z: f64,
    /// Radians.
    pub start: f64,
    pub end: f64,
    pub offset: f64,
    pub cen_x: f64,
    pub cen_y: f64,
    pub cen_z: f64,
    pub square: bool,
    /// Sweep UVs: 0 U, 1 V (which way round the axis U runs, LT-27).
    pub uvs: u32,
    pub flip: bool,
    pub cap0: bool,
    pub cap1: bool,
}
impl Default for RadialSweepOp {
    fn default() -> Self {
        Self {
            axis: 1,
            sides: 8,
            vec_x: 0.,
            vec_y: 1.,
            vec_z: 0.,
            start: 0.,
            end: std::f64::consts::TAU,
            offset: 0.,
            cen_x: 0.,
            cen_y: 0.,
            cen_z: 0.,
            square: false,
            uvs: 0,
            flip: false,
            cap0: true,
            cap1: true,
        }
    }
}
impl TypedOp for RadialSweepOp {
    fn run(&self, ctx: &OpContext) -> Result<Mesh> {
        if self.axis > 2 {
            return Err("Axis must be 0 (X), 1 (Y) or 2 (Z)".into());
        }
        if self.uvs > 1 {
            return Err("Sweep UVs must be 0 (U) or 1 (V)".into());
        }
        let lathe = Lathe {
            circle: Circle {
                count: self.sides,
                axis: self.axis as usize,
                center: DVec3::new(self.cen_x, self.cen_y, self.cen_z),
                direction: DVec3::new(self.vec_x, self.vec_y, self.vec_z),
                start: self.start.to_degrees(),
                end: self.end.to_degrees(),
                offset: self.offset,
            },
            sweep: Sweep {
                flip: self.flip,
                cap_start: self.cap0,
                cap_end: self.cap1,
                uvs_around: if self.uvs == 0 {
                    UvsAround::U
                } else {
                    UvsAround::V
                },
            },
        };
        // The tool works on a scene: one item at the origin holding the
        // input, with the op's elements selected.
        let mut scene = Scene::default();
        scene.new_mesh()?;
        let id = scene.items.last().map(|i| i.id).ok_or("No mesh")?;
        scene.item_mut(id).ok_or("No mesh")?.mesh = ctx.input.clone().into();
        scene.select_item(Some(id));
        let s = &mut scene.selection;
        s.mode = Mode::Polygon;
        if let Some(picked) = ctx.selection {
            if ctx.element == SelElement::Edge {
                let all = ctx.input.edges();
                s.mode = Mode::Edge;
                s.edges = picked
                    .iter()
                    .filter_map(|&e| all.get(e as usize).copied())
                    .collect();
            } else {
                s.faces = picked.to_vec();
            }
        }
        crate::duplicate_sweep::mesh_lathe(&mut scene, &lathe)?;
        Ok((*scene.item(id).ok_or("No mesh")?.mesh).clone())
    }
}

pub(crate) fn entries() -> Vec<Arc<dyn OpDef>> {
    let fields = vec![
        choice("axis", "Axis", &["X", "Y", "Z"]),
        Field::new(
            "sides",
            "Count",
            FieldKind::Count {
                min: 1,
                max: 100_000,
            },
        ),
        distance("vecX", "Axis X"),
        distance("vecY", "Axis Y"),
        distance("vecZ", "Axis Z"),
        Field::new("start", "Start Angle", FieldKind::Angle),
        Field::new("end", "End Angle", FieldKind::Angle),
        distance("offset", "Offset"),
        distance("cenX", "Center X"),
        distance("cenY", "Center Y"),
        distance("cenZ", "Center Z"),
        hidden("square"),
        choice("uvs", "Sweep UVs", &["U", "V"]),
        check("flip", "Invert Polygons"),
        check("cap0", "Cap Start"),
        check("cap1", "Cap End"),
    ];
    crate::meshop_types::with_element(
        vec![Arc::new(Typed::<RadialSweepOp>::new(
            "radial.sweep.item",
            "Radial Sweep",
            "Duplicate",
            true,
            fields,
        ))],
        SelElement::Edge,
    )
}
