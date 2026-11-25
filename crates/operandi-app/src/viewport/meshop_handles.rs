//! Handles for the selected primitive MeshOp (Steve Hill via Jason,
//! 2026-11-25): with a primitive op's row selected in the Mesh Stack and
//! no tool on, the view shows its centre, one size square per axis and
//! its own falloff's points. Dragging one edits the op's values live,
//! one undo step per drag (SAFE-13, as the op's fields do).
//!
//! The handles follow the tool handles' look (UIC section 7): the centre
//! and size squares in the axis colours, falloff points as small points.
use super::*;
use operandi_core::mesh_stack_editor::OpRef;
use operandi_core::{Falloff, ShapeWeights};

/// One handle of the selected primitive op.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum OpHandle {
    /// Moves the shape (`cenX`..).
    Centre,
    /// Resizes along one axis (`sizeX`..).
    Size(usize),
    /// A point of the op's own falloff: Linear Start (0) and End (1), a
    /// Radial or Cylinder falloff's Centre (0).
    Falloff(usize),
}

/// A drag on one handle: the op, the handle, the op's values at the press
/// and the undo gesture the drag folds into.
#[derive(Clone, Debug)]
pub(crate) struct OpGrab {
    pub(crate) op: OpRef,
    pub(crate) handle: OpHandle,
    pub(crate) start: serde_json::Value,
    pub(crate) gesture: u64,
}

const AXES: [&str; 3] = ["X", "Y", "Z"];

fn vec3(values: &serde_json::Value, prefix: &str) -> Option<DVec3> {
    let c = |a: &str| values.get(format!("{prefix}{a}"))?.as_f64();
    Some(DVec3::new(c("X")?, c("Y")?, c("Z")?))
}

/// The op's falloff points in its own space.
fn falloff_points(values: &serde_json::Value) -> Vec<DVec3> {
    let Ok(Some(w)) = serde_json::from_value::<Option<ShapeWeights>>(values["falloff"].clone())
    else {
        return Vec::new();
    };
    match w.first {
        Falloff::Linear(f) => vec![f.start, f.end],
        Falloff::Radial(f) => vec![f.center],
        Falloff::Cylinder(f) => vec![f.center],
        _ => Vec::new(),
    }
}

impl Operandi {
    /// The selected primitive op and its values, when no tool is on.
    pub(crate) fn selected_primitive_op(&self) -> Option<(OpRef, serde_json::Value)> {
        if self.editor.live.is_some() || self.editor.has_active_task() {
            return None;
        }
        let row = self.editor.meshop_row?;
        let op_id = row.op?;
        let item = self.editor.scene.item(row.item)?;
        let op = item
            .mesh_stack
            .as_ref()?
            .ops
            .iter()
            .find(|o| o.id == op_id)?;
        let def = op.kind.def()?;
        if !def.id().starts_with("prim.") {
            return None;
        }
        let values = op.kind.values();
        vec3(&values, "cen")?;
        Some((
            OpRef {
                item: row.item,
                op: op_id,
            },
            values,
        ))
    }

    /// The selected primitive op's handles in world space.
    pub(crate) fn meshop_handles(&self) -> Vec<(DVec3, OpHandle)> {
        let Some((r, values)) = self.selected_primitive_op() else {
            return Vec::new();
        };
        let Ok(world) = self.editor.scene.world_matrix(r.item) else {
            return Vec::new();
        };
        let Some(centre) = vec3(&values, "cen") else {
            return Vec::new();
        };
        let mut out = vec![(world.transform_point3(centre), OpHandle::Centre)];
        if let Some(size) = vec3(&values, "size") {
            // The Cube's Size is its full width; the others' are radii.
            let full = values.get("segmentsX").is_some();
            for a in 0..3 {
                let mut p = centre;
                p[a] += if full { size[a] * 0.5 } else { size[a] };
                out.push((world.transform_point3(p), OpHandle::Size(a)));
            }
        }
        for (i, p) in falloff_points(&values).into_iter().enumerate() {
            out.push((world.transform_point3(p), OpHandle::Falloff(i)));
        }
        out
    }

    /// The handle under `pos`, nearest first.
    pub(crate) fn meshop_handle_hit(&self, pos: Pos2, r: Rect) -> Option<OpGrab> {
        let reach = self.handle_preferences.small_point_hit;
        let (op, start) = self.selected_primitive_op()?;
        let handle = self
            .meshop_handles()
            .into_iter()
            .filter_map(|(p, h)| Some((self.camera.project(p, r)?.0.distance(pos), h)))
            .filter(|&(d, _)| d <= reach)
            .min_by(|a, b| a.0.total_cmp(&b.0))?
            .1;
        Some(OpGrab {
            op,
            handle,
            start,
            gesture: crate::colour_field::next_gesture(),
        })
    }

    /// A drag from `from` to `to`: the op's values from the press, moved.
    pub(crate) fn drag_meshop_handle(&mut self, grab: &OpGrab, from: Pos2, to: Pos2, r: Rect) {
        let (_, right, up, _) = self.camera.basis();
        let scale = self.camera.distance / (r.height() as f64 * 0.9);
        let d = to - from;
        let world_delta = (right * d.x as f64 - up * d.y as f64) * scale;
        let Ok(world) = self.editor.scene.world_matrix(grab.op.item) else {
            return;
        };
        let delta = world.inverse().transform_vector3(world_delta);
        let v = &grab.start;
        let mut changes = serde_json::Map::new();
        match grab.handle {
            OpHandle::Centre => {
                let Some(c) = vec3(v, "cen") else { return };
                for a in 0..3 {
                    changes.insert(format!("cen{}", AXES[a]), (c[a] + delta[a]).into());
                }
            }
            OpHandle::Size(a) => {
                let Some(s) = vec3(v, "size") else { return };
                let full = v.get("segmentsX").is_some();
                let grow = if full { 2. * delta[a] } else { delta[a] };
                changes.insert(format!("size{}", AXES[a]), (s[a] + grow).max(0.).into());
            }
            OpHandle::Falloff(i) => {
                let Ok(Some(mut w)) =
                    serde_json::from_value::<Option<ShapeWeights>>(v["falloff"].clone())
                else {
                    return;
                };
                w.first = match w.first {
                    Falloff::Linear(mut f) => {
                        if i == 0 {
                            f.start += delta;
                        } else {
                            f.end += delta;
                        }
                        Falloff::Linear(f)
                    }
                    Falloff::Radial(mut f) => {
                        f.center += delta;
                        Falloff::Radial(f)
                    }
                    Falloff::Cylinder(mut f) => {
                        f.center += delta;
                        Falloff::Cylinder(f)
                    }
                    other => other,
                };
                let Ok(json) = serde_json::to_value(Some(w)) else {
                    return;
                };
                changes.insert("falloff".into(), json);
            }
        }
        // One undo step for the whole drag (SAFE-13).
        let previous = self.editor.undo_fold.replace(grab.gesture);
        self.run(Command::MeshOpSet {
            op: grab.op,
            values: changes,
        });
        self.editor.undo_fold = previous;
    }

    /// Draws the selected primitive op's handles.
    pub(super) fn draw_meshop_handles(&self, painter: &egui::Painter, r: Rect) {
        if !self.handle_preferences.unselected_shown() {
            return;
        }
        let handles = self.meshop_handles();
        let Some(&(centre, _)) = handles.first() else {
            return;
        };
        let Some((c, _)) = self.camera.project(centre, r) else {
            return;
        };
        let side = self.handle_preferences.small_point_size();
        for &(p, h) in &handles {
            let Some((q, _)) = self.camera.project(p, r) else {
                continue;
            };
            let size = side * self.handle_zoom_at(p);
            match h {
                OpHandle::Centre => {
                    painter.circle_filled(q, size * 0.6, palette::selection());
                }
                OpHandle::Size(a) => {
                    let color = palette::handles()[a];
                    painter.line_segment(
                        [c, q],
                        Stroke::new(self.handle_preferences.line_width, color),
                    );
                    painter.rect_filled(Rect::from_center_size(q, vec2(size, size)), 0., color);
                }
                OpHandle::Falloff(_) => {
                    painter.circle_stroke(
                        q,
                        size * 0.6,
                        Stroke::new(self.handle_preferences.line_width, palette::selection()),
                    );
                }
            }
        }
    }
}
