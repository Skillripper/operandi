//! Ids of tool handles that both the viewport's handle drawing and the hover
//! tips (handle_tips.rs) name.

/// What a falloff handle moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FalloffHandle {
    /// Point `i` of `falloff_points` (Linear Start/End, Radial and Cylinder
    /// Center, then Size handles), moved freely in the view plane.
    Point(usize),
    /// Point `i` moved along one axis only (FO-40, FO-41, FO-42).
    Axis(usize, usize),
}

// Twist Bevel (twist_bevel_handles.rs).
/// Handle ids: Shift and Inset take 0 and 1 (the first and second handle a
/// drag off the handles stands for); the rings and the bend arrow follow
/// the plane circles' ids (13..16).
pub(crate) const TB_SHIFT: usize = 0;
pub(crate) const TB_INSET: usize = 1;
pub(crate) const TB_TWIST: usize = 20;
pub(crate) const TB_BEND: usize = 21;
pub(crate) const TB_BEND_DIRECTION: usize = 22;
