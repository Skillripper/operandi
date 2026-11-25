//! What a polygon is (a face, a subdivision cage, a line or curve, a patch)
//! and how a line element is drawn. Every area that reads a mesh needs
//! these, so they live with the mesh in the base layer
//! (docs/AREA_DEPENDENCIES.md).
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolygonRepresentation {
    #[default]
    Face,
    Subdivs,
    CatmullClark,
    /// Ordered line element; closure adds a span, never another point.
    /// `curve` says how the points are drawn: straight (a polyline) or as
    /// a smooth Curve, B-Spline or Bezier through or near them.
    Polyline {
        closed: bool,
        #[serde(default, skip_serializing_if = "CurveKind::is_line")]
        curve: CurveKind,
    },
    /// A live patch (G7 PC-7): the polygon lists only its 3 or 4 corner
    /// points; its surface is a Coons patch over the open curves joining
    /// each pair of neighbouring corners (patch.rs), tessellated when the
    /// scene is evaluated. Edits treat it as a plain polygon.
    Patch,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveKind {
    /// Straight spans between the points: an ordinary polyline.
    #[default]
    Line,
    Curve {
        start_control: bool,
        end_control: bool,
    },
    BSpline,
    Bezier,
}
impl CurveKind {
    pub fn is_line(&self) -> bool {
        *self == CurveKind::Line
    }
    /// Fewest control points the curve needs (DRW-Q08: fewer is removed on
    /// Drop). Bezier counts stored points: two anchors with their handles.
    pub fn minimum_points(&self) -> usize {
        match self {
            CurveKind::Line => 2,
            CurveKind::Curve {
                start_control,
                end_control,
            } => 2 + usize::from(*start_control) + usize::from(*end_control),
            CurveKind::BSpline => 4,
            CurveKind::Bezier => 4,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            CurveKind::Line => "Polyline",
            CurveKind::Curve { .. } => "Curve",
            CurveKind::BSpline => "B-Spline",
            CurveKind::Bezier => "Bezier Curve",
        }
    }
}
