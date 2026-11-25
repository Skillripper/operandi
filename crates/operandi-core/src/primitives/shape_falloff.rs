//! Falloffs on primitives (GitHub #15, Steve Hill's idea; Jason's go on the
//! revised proposal, 2026-11-25). The global falloff (Linear, Radial,
//! Cylinder, Noise) weights a primitive's points while it is made, and the
//! Falloff section says what the weight drives: each ring is scaled, moved
//! and turned in full at weight 1. Sampled once per ring on the spine, rings
//! stay round; sampled per point, a Noise or Radial falloff dents them.
//! Follow Falloff spacing moves the Segment rings to where the weight
//! changes fastest, so Segments shape the profile.
//!
//! It runs inside the shapes' one generator (primitive_shapes.rs), after the
//! plain shape is built, so the tools' previews, Drop, undo and the
//! primitive MeshOps give the same mesh. A MeshOp keeps its own copy of the
//! falloff layers ([`ShapeWeights`]): the tool falloff isn't scene data.
use crate::falloff::PointInputs;
use crate::{ActiveFalloff, Falloff, Mesh, MixMode, Result};
use glam::{DQuat, DVec3};
use serde::{Deserialize, Serialize};

/// Where the weight is read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FalloffSample {
    /// Once per ring, on the spine at the ring's height: rings stay round.
    #[default]
    Spine,
    /// At every point.
    Points,
}
impl FalloffSample {
    pub const ALL: [Self; 2] = [Self::Spine, Self::Points];
    pub fn label(self) -> &'static str {
        match self {
            Self::Spine => "Spine",
            Self::Points => "Points",
        }
    }
}

/// Where the Segment rings sit along the spine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FalloffSpacing {
    /// As the shape makes them.
    #[default]
    Even,
    /// Packed where the weight changes fastest. Straight-sided shapes only
    /// (Cube, Cylinder): moving a Sphere's rings would change the sphere.
    Follow,
}
impl FalloffSpacing {
    pub const ALL: [Self; 2] = [Self::Even, Self::Follow];
    pub fn label(self) -> &'static str {
        match self {
            Self::Even => "Even",
            Self::Follow => "Follow Falloff",
        }
    }
}

/// What the weight drives: the Falloff section's values. The defaults
/// change nothing, so a primitive only reacts to a falloff once one is set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeFalloff {
    /// The rings' size at weight 1 (1.0 = 100%, unchanged).
    pub scale: f64,
    /// How far the rings move at weight 1, metres.
    pub offset: DVec3,
    /// How far the rings turn about the spine at weight 1, degrees.
    pub twist: f64,
    pub sample: FalloffSample,
    pub spacing: FalloffSpacing,
}
impl Default for ShapeFalloff {
    fn default() -> Self {
        Self {
            scale: 1.,
            offset: DVec3::ZERO,
            twist: 0.,
            sample: FalloffSample::Spine,
            spacing: FalloffSpacing::Even,
        }
    }
}

/// The falloff layers a primitive reads: the ones that weigh positions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeWeights {
    pub first: Falloff,
    #[serde(default)]
    pub added: Vec<(Falloff, MixMode)>,
}
/// Why a falloff can't shape new geometry, if it can't.
pub fn unusable_reason(f: Falloff) -> Option<String> {
    match f {
        Falloff::Linear(_)
        | Falloff::Radial(_)
        | Falloff::Cylinder(_)
        | Falloff::Noise(_)
        | Falloff::Invert => None,
        other => Some(format!(
            "A {} falloff weighs points that already exist, and a new primitive has none; \
             use Linear, Radial, Cylinder or Noise",
            other.label()
        )),
    }
}
impl ShapeWeights {
    pub fn new(first: Falloff, added: Vec<(Falloff, MixMode)>) -> Result<Self> {
        let w = Self { first, added };
        w.validate()?;
        Ok(w)
    }
    /// The tool falloff's position layers, or why it can't shape a
    /// primitive.
    pub fn from_active(a: &ActiveFalloff) -> Result<Self> {
        Self::new(a.first, a.added.clone())
    }
    pub fn validate(&self) -> Result<()> {
        for f in std::iter::once(self.first).chain(self.added.iter().map(|a| a.0)) {
            if let Some(why) = unusable_reason(f) {
                return Err(why);
            }
            f.validate()?;
        }
        Ok(())
    }
    fn stack(&self) -> ActiveFalloff {
        ActiveFalloff {
            added: self.added.clone(),
            ..ActiveFalloff::new(self.first)
        }
    }
    /// A weight function over positions; `to_local` maps a point to the
    /// target mesh's space for falloffs with Use World Transforms off.
    pub fn weigher(&self, to_local: glam::DMat4) -> impl Fn(DVec3) -> f64 + '_ {
        let stack = self.stack();
        move |p| stack.weight_at_local(p, to_local.transform_point3(p), 0, PointInputs::default())
    }
}

/// Fine steps for Follow Falloff's ring placement.
const STEPS: usize = 512;

impl ShapeFalloff {
    pub fn validate(&self) -> Result<()> {
        if ![self.scale, self.twist].iter().all(|v| v.is_finite()) || !self.offset.is_finite() {
            return Err("Falloff Scale, Offset and Twist must be finite numbers".into());
        }
        if self.scale < 0. {
            return Err("Falloff Scale can't be negative".into());
        }
        Ok(())
    }
    /// Whether these values change a shape at all.
    pub fn neutral(&self) -> bool {
        self.scale == 1.
            && self.offset == DVec3::ZERO
            && self.twist == 0.
            && self.spacing == FalloffSpacing::Even
    }
    /// Shapes `mesh`, built around `centre` with its spine along world
    /// `axis`, by `weight` (a position's falloff weight). `respace` says
    /// whether Follow Falloff may move rings.
    pub fn apply(
        &self,
        mesh: &mut Mesh,
        centre: DVec3,
        axis: usize,
        respace: bool,
        weight: &dyn Fn(DVec3) -> f64,
    ) -> Result<()> {
        self.validate()?;
        if self.neutral() || mesh.vertices.is_empty() {
            return Ok(());
        }
        if axis > 2 {
            return Err("Axis must be X, Y or Z".into());
        }
        let (lo, hi) = mesh
            .vertices
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v[axis]), hi.max(v[axis]))
            });
        let height = hi - lo;
        let spine_at = |h: f64| {
            let mut s = centre;
            s[axis] = h;
            s
        };
        let table = (respace && self.spacing == FalloffSpacing::Follow && height > 0.)
            .then(|| density_table(&|t| weight(spine_at(lo + t * height))));
        let mut spine = DVec3::ZERO;
        spine[axis] = 1.;
        for v in &mut mesh.vertices {
            if let Some(table) = &table {
                let t = respace_t(table, (v[axis] - lo) / height);
                v[axis] = lo + t * height;
            }
            let on_spine = spine_at(v[axis]);
            let w = match self.sample {
                FalloffSample::Spine => weight(on_spine),
                FalloffSample::Points => weight(*v),
            };
            let w = if w.is_finite() { w } else { 0. };
            let across = *v - on_spine;
            let turn = DQuat::from_axis_angle(spine, (self.twist * w).to_radians());
            let size = (1. + (self.scale - 1.) * w).max(0.);
            *v = on_spine + turn * (across * size) + self.offset * w;
        }
        // A ring scaled to 0 leaves zero-area polygons: keep them as
        // explicit degenerate records, as the shapes do (shapes.rs).
        for f in 0..mesh.faces.len() {
            if mesh.normal(f) == DVec3::ZERO {
                mesh.attributes.degenerate_faces.insert(f as u32);
            }
        }
        mesh.validate()
    }
}

/// `table[i]` = the share of all ring density below `i / STEPS` of the
/// spine: steeper weight, more rings.
fn density_table(weight: &dyn Fn(f64) -> f64) -> Vec<f64> {
    let h = 1. / STEPS as f64;
    let mut table = Vec::with_capacity(STEPS + 1);
    let mut sum = 0.;
    table.push(0.);
    let mut before = weight(0.);
    for i in 1..=STEPS {
        let now = weight(i as f64 * h);
        // 4 makes a full-height ramp's steepest part about five times as
        // dense as its flat part.
        let slope = ((now - before) / h).abs();
        let slope = if slope.is_finite() {
            slope.min(50.)
        } else {
            0.
        };
        sum += h * (1. + 4. * slope);
        table.push(sum);
        before = now;
    }
    for c in &mut table {
        *c /= sum;
    }
    table
}
/// Where a ring evenly placed at `t` goes: the inverse of the table.
fn respace_t(table: &[f64], t: f64) -> f64 {
    let t = t.clamp(0., 1.);
    let i = table.partition_point(|&c| c < t).clamp(1, STEPS);
    let (c0, c1) = (table[i - 1], table[i]);
    let f = if c1 > c0 { (t - c0) / (c1 - c0) } else { 0. };
    (i as f64 - 1. + f) / STEPS as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FalloffShape, LinearFalloff};

    /// Steve's spine ramp: a Linear falloff, full at the top, none at the
    /// base of the default 1.5 m cylinder.
    fn ramp() -> ShapeWeights {
        ShapeWeights::new(
            Falloff::Linear(LinearFalloff {
                start: DVec3::new(0., 0.75, 0.),
                end: DVec3::new(0., -0.75, 0.),
                shape: FalloffShape::Linear,
                world: true,
                ..Default::default()
            }),
            Vec::new(),
        )
        .unwrap()
    }
    fn tall_cylinder(segments: u32) -> Mesh {
        crate::CylinderShape {
            segments,
            make_uvs: false,
            ..Default::default()
        }
        .mesh()
        .unwrap()
    }
    fn radius_at(mesh: &Mesh, y: f64) -> f64 {
        mesh.vertices
            .iter()
            .filter(|v| (v.y - y).abs() < 1e-9)
            .map(|v| v.x.hypot(v.z))
            .fold(0., f64::max)
    }
    fn shaped(f: ShapeFalloff, mut mesh: Mesh, weights: &ShapeWeights) -> Mesh {
        let w = weights.weigher(glam::DMat4::IDENTITY);
        f.apply(&mut mesh, DVec3::ZERO, 1, true, &w).unwrap();
        mesh
    }

    #[test]
    fn neutral_values_change_nothing() {
        let mesh = tall_cylinder(4);
        assert_eq!(shaped(ShapeFalloff::default(), mesh.clone(), &ramp()), mesh);
    }

    #[test]
    fn scale_tapers_each_ring_by_its_weight() {
        let f = ShapeFalloff {
            scale: 0.5,
            ..Default::default()
        };
        let mesh = shaped(f, tall_cylinder(4), &ramp());
        assert!(
            (radius_at(&mesh, -0.75) - 0.5).abs() < 1e-9,
            "base unchanged"
        );
        assert!((radius_at(&mesh, 0.) - 0.375).abs() < 1e-9, "half way");
        assert!((radius_at(&mesh, 0.75) - 0.25).abs() < 1e-9, "top at 50%");
    }

    #[test]
    fn twist_and_offset_move_only_the_upper_rings() {
        let f = ShapeFalloff {
            twist: 90.,
            offset: DVec3::new(0.2, 0., 0.),
            ..Default::default()
        };
        let mesh = shaped(f, tall_cylinder(2), &ramp());
        // The top ring's point at +X turns 90° about +Y to -Z, then moves
        // 0.2 m along X.
        assert!(
            mesh.vertices
                .iter()
                .any(|v| (*v - DVec3::new(0.2, 0.75, -0.5)).length() < 1e-9)
        );
        assert!((radius_at(&mesh, -0.75) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn follow_spacing_packs_rings_where_the_weight_changes() {
        // A ramp only over the upper half: rings bunch there.
        let weights = ShapeWeights::new(
            Falloff::Linear(LinearFalloff {
                start: DVec3::new(0., 0.75, 0.),
                end: DVec3::new(0., 0., 0.),
                world: true,
                ..Default::default()
            }),
            Vec::new(),
        )
        .unwrap();
        let even = ShapeFalloff {
            scale: 0.2,
            ..Default::default()
        };
        let follow = ShapeFalloff {
            spacing: FalloffSpacing::Follow,
            ..even
        };
        let rings_above_zero = |m: &Mesh| {
            let mut ys: Vec<i64> = m
                .vertices
                .iter()
                .filter(|v| v.y > 1e-9)
                .map(|v| (v.y * 1e6).round() as i64)
                .collect();
            ys.sort();
            ys.dedup();
            ys.len()
        };
        let a = shaped(even, tall_cylinder(8), &weights);
        let b = shaped(follow, tall_cylinder(8), &weights);
        assert!(rings_above_zero(&b) > rings_above_zero(&a));
        assert!((radius_at(&b, -0.75) - 0.5).abs() < 1e-9, "ends stay");
        assert!((radius_at(&b, 0.75) - 0.1).abs() < 1e-9);
    }

    #[test]
    fn points_sampling_dents_rings_spine_sampling_keeps_them_round() {
        // A Radial falloff off to one side: per point, the near side of a
        // ring weighs more than the far side.
        let weights = ShapeWeights::new(
            Falloff::Radial(crate::RadialFalloff {
                center: DVec3::new(0.5, 0., 0.),
                size: DVec3::splat(0.8),
                world: true,
                ..Default::default()
            }),
            Vec::new(),
        )
        .unwrap();
        let f = ShapeFalloff {
            scale: 0.5,
            ..Default::default()
        };
        let round = |m: &Mesh| {
            let r: Vec<f64> = m
                .vertices
                .iter()
                .filter(|v| v.y.abs() < 1e-9)
                .map(|v| v.x.hypot(v.z))
                .collect();
            let (lo, hi) = r
                .iter()
                .fold((f64::MAX, 0f64), |(a, b), &x| (a.min(x), b.max(x)));
            hi - lo < 1e-9
        };
        let spine = shaped(f, tall_cylinder(2), &weights);
        assert!(round(&spine));
        let points = shaped(
            ShapeFalloff {
                sample: FalloffSample::Points,
                ..f
            },
            tall_cylinder(2),
            &weights,
        );
        assert!(!round(&points));
    }

    #[test]
    fn scale_zero_makes_a_point_and_stays_valid() {
        let f = ShapeFalloff {
            scale: 0.,
            ..Default::default()
        };
        let mesh = shaped(f, tall_cylinder(3), &ramp());
        assert!(radius_at(&mesh, 0.75) < 1e-12);
        assert!(!mesh.attributes.degenerate_faces.is_empty());
    }

    #[test]
    fn point_falloffs_are_refused_with_a_reason() {
        let e = ShapeWeights::new(
            Falloff::Selection(crate::SelectionFalloff::default()),
            Vec::new(),
        )
        .unwrap_err();
        assert!(e.contains("Selection"), "{e}");
        let bad = ShapeFalloff {
            scale: -1.,
            ..Default::default()
        };
        let mut mesh = tall_cylinder(1);
        assert!(bad.apply(&mut mesh, DVec3::ZERO, 1, true, &|_| 1.).is_err());
    }
}
