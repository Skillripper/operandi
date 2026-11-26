//! The plane and centre a newly placed point lands on: the work plane, moved
//! by Locked Plane Offset, and the depth a view without perspective keeps.
use super::*;

impl Operandi {
    pub(crate) fn actual_plane(&self) -> u8 {
        if self.editor.scene.workplane.fixed {
            return self.editor.scene.workplane.plane;
        }
        self.units_preferences.view_plane(self.camera.basis().3)
    }
    /// The work plane's centre: the scene's, moved along the plane's normal
    /// by Locked Plane Offset while Lock Preferred Plane holds it.
    pub(crate) fn workplane_center(&self) -> DVec3 {
        let w = &self.editor.scene.workplane;
        let u = &self.units_preferences;
        if w.fixed || !u.lock_preferred {
            return w.center;
        }
        let normal = match u.preferred_plane {
            1 => DVec3::X,
            2 => DVec3::Z,
            _ => DVec3::Y,
        };
        w.center + normal * u.locked_offset
    }
    /// Where the plane for a newly placed point passes when it extends
    /// `from` (the current point of a sketch or curve). In a view without
    /// perspective (Top, Front, Left ... and free Orthographic) the plane
    /// goes through `from`, so the coordinate along the view's depth keeps
    /// the previous point's value and only the two in-view axes come from
    /// the pointer: a leg drawn in Front and continued in Left stays off
    /// the centre line. Perspective views, a fixed work plane, a locked
    /// preferred plane or no `from` keep the work plane's centre. Only the
    /// depth moves: the in-plane axes stay the work plane's, so Coordinate
    /// Rounding still steps from the grid's origin, not from `from`.
    pub(crate) fn placement_center(&self, from: Option<DVec3>) -> DVec3 {
        let mut center = self.workplane_center();
        let w = &self.editor.scene.workplane;
        if let Some(p) = from
            && self.camera.view != View::Perspective
            && !w.fixed
            && !self.units_preferences.lock_preferred
            && p.is_finite()
        {
            // The plane's normal axis: 0 XZ → Y, 1 YZ → X, 2 XY → Z.
            let depth = match self.actual_plane() {
                1 => 0,
                2 => 2,
                _ => 1,
            };
            center[depth] = p[depth];
        }
        center
    }
}
