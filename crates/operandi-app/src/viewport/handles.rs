//! The live tool's handles: which ones it shows, where they sit and point
//! (their frames), and which one the pointer is over.
use super::*;

impl Operandi {
    /// Axis Rotate (TC-30): the one ring it shows, the work plane's normal
    /// (plane 0 is XZ, 1 YZ, 2 XY).
    pub(crate) fn axis_rotate_axis(&self) -> Option<usize> {
        let p = self.editor.preset.as_ref()?;
        if !p.row().actor.single_ring {
            return None;
        }
        Some([1, 0, 2][self.actual_plane().min(2) as usize])
    }
    pub(crate) fn handles(&self, r: Rect) -> Vec<(usize, Pos2, Pos2, String)> {
        let all = self.all_handles(r);
        match self.axis_rotate_axis() {
            Some(axis) => all.into_iter().filter(|h| h.0 == axis).collect(),
            None => all,
        }
    }
    pub(super) fn all_handles(&self, r: Rect) -> Vec<(usize, Pos2, Pos2, String)> {
        let Some(l) = &self.editor.live else {
            return Vec::new();
        };
        // CS-01: an armed tool draws nothing until its first click.
        if self.editor.is_armed() {
            return Vec::new();
        }
        // CP-A02: transform tools show nothing on a locked item.
        if self.editor.scene.selection.mode == Mode::Item
            && l.tool.transforms_items()
            && self
                .editor
                .scene
                .active()
                .is_some_and(operandi_core::Item::is_locked)
        {
            return Vec::new();
        }
        if self.handle_preferences.selected == crate::handle_preferences::HandleDetail::Invisible {
            return Vec::new();
        }
        // On-screen handle lengths follow Handle Scale: Standard, and with
        // original-compatible A1 the zoom at the handle's centre.
        let s_at = |center: DVec3| {
            let zoom = self.handle_zoom_at(center);
            self.handle_zoom.set(zoom);
            self.handle_preferences.scale * zoom
        };
        // The handle layout is on the tool's card. Tools without handles
        // grab, paint, draw or click (Tweak's click-to-show handle is TWK-3).
        let card = l.tool.card().handles;
        if card.kind == HandleKind::None {
            return Vec::new();
        }
        if card.kind == HandleKind::RadialBounds {
            let Some(radial) = l.params.radial else {
                return Vec::new();
            };
            if !l.applied && radial.radius.min_element() > 0. {
                return Vec::new();
            }
            let mut out = Vec::new();
            // N21 (Jason, 2026-11-17; provisional, Q-DC-04): once the shape
            // is drawn, three move arrows at its centre position it, as the
            // Cube's do; the squares still shape it.
            if radial.radius.max_element() > 0.
                && let Some((center, _)) = self.camera.project(l.params.position, r)
            {
                let s = s_at(l.params.position);
                for axis in 0..3 {
                    let direction = [DVec3::X, DVec3::Y, DVec3::Z][axis];
                    if let Some((b, _)) = self.camera.project(
                        l.params.position + direction * self.camera.distance * 0.1,
                        r,
                    ) && center.distance(b) > 5.
                    {
                        out.push((
                            axis,
                            center,
                            center + (b - center).normalized() * 72. * s,
                            ["X", "Y", "Z"][axis].into(),
                        ));
                    }
                }
            }
            for axis in 0..3 {
                let direction = [DVec3::X, DVec3::Y, DVec3::Z][axis];
                let (Some((center, _)), Some((unit, _))) = (
                    self.camera.project(l.params.position, r),
                    self.camera.project(l.params.position + direction, r),
                ) else {
                    continue;
                };
                if center.distance(unit) < 1. {
                    continue;
                }
                for maximum in [false, true] {
                    let sign = if maximum { 1. } else { -1. };
                    let point = l.params.position + direction * radial.radius[axis] * sign;
                    if let Some((mut point, _)) = self.camera.project(point, r) {
                        if radial.radius[axis] == 0. {
                            point = center
                                + (unit - center).normalized()
                                    * 22.
                                    * s_at(l.params.position)
                                    * sign as f32;
                        }
                        out.push((
                            3 + axis * 2 + usize::from(maximum),
                            point,
                            point,
                            format!(
                                "{}{} {}",
                                if maximum { "+" } else { "−" },
                                ["X", "Y", "Z"][axis],
                                if l.tool == Tool::Sphere {
                                    "radius"
                                } else {
                                    "bound"
                                }
                            ),
                        ));
                    }
                }
            }
            return out;
        }
        if l.tool == Tool::EdgeExtend {
            return self.edge_extend_handles(l, r);
        }
        if card.kind == HandleKind::Spine {
            return self.twist_bevel_handles(l, r);
        }
        if l.tool == Tool::Transform {
            let center = self.transform_handle_center();
            let Some((origin, _)) = self.camera.project(center, r) else {
                return Vec::new();
            };
            let s = s_at(center);
            let mut out = Vec::new();
            for axis in 0..3 {
                let direction = self.handle_axis(axis);
                if let Some((unit, _)) = self
                    .camera
                    .project(center + direction * self.camera.distance * 0.1, r)
                    && unit.distance(origin) > 1.
                {
                    let screen = (unit - origin).normalized();
                    out.push((
                        axis,
                        origin,
                        origin + screen * 72. * s,
                        format!("Move {}", ["X", "Y", "Z"][axis]),
                    ));
                    out.push((
                        6 + axis,
                        origin + screen * 30. * s,
                        origin + screen * 44. * s,
                        format!("Scale {}", ["X", "Y", "Z"][axis]),
                    ));
                }
                if let Some(end) = self.rotation_ring(axis, r).last() {
                    out.push((
                        3 + axis,
                        origin,
                        *end,
                        format!("Rotate {}", ["X", "Y", "Z"][axis]),
                    ));
                }
            }
            // PH-32: the move part's plane circles.
            out.extend(self.plane_handles(l.tool, r));
            out.push((FREE_HANDLE, origin, origin, "Free move".into()));
            return out;
        }
        if l.tool == Tool::Box && !l.applied {
            return Vec::new();
        }
        if l.tool.is_smooth_shift() && !l.applied {
            return Vec::new();
        }
        // User correction 2026-10-30: Extrude shows no gizmo until the first
        // viewport click starts the extrusion.
        if l.tool == Tool::Extrude && !l.applied && !self.editor.i2_active() {
            return Vec::new();
        }
        let item = self.editor.scene.active();
        // The pivot service's handle under Symmetry (ACS-01): the polygon
        // stage follows the primary side's live caps.
        let polygon_symmetry_centre = self
            .editor
            .symmetry_handle_centre()
            .filter(|_| card.anchor == HandleAnchor::PolygonStage);
        let origin = match card.anchor {
            HandleAnchor::Edge => self.edge_handle_frame().map(|f| f.0).unwrap_or(DVec3::ZERO),
            HandleAnchor::EdgeRail => self.edge_slide_frame().map(|f| f.0).unwrap_or(DVec3::ZERO),
            HandleAnchor::Vertex => self.vertex_frame().map(|f| f.0).unwrap_or(DVec3::ZERO),
            HandleAnchor::None => unreachable!("{} has no handles", l.tool.label()),
            HandleAnchor::ChamferStrip => l
                .base
                .active()
                .and_then(|item| {
                    operandi_core::chamfer_strips(&item.mesh, &l.base.selection.faces)
                        .first()
                        .filter(|&&face| (face as usize) < item.mesh.faces.len())
                        .map(|&face| item.mesh.center(face as usize))
                })
                .unwrap_or(DVec3::ZERO),
            HandleAnchor::Position => l.params.position,
            HandleAnchor::PolygonStage => polygon_symmetry_centre.unwrap_or_else(|| {
                let frame = l.action_frame;
                if self.editor.action_context.center == CenterMode::Origin {
                    frame.map(|f| f.center).unwrap_or(DVec3::ZERO)
                } else if self.editor.action_context.center == CenterMode::Selection
                    || (l.tool == Tool::Bevel && self.editor.scene.selection.faces.len() > 1)
                {
                    // Follow the current cap translation/normal field while the
                    // stage orientation stays captured from its immutable base.
                    // Default multi-face Bevel uses the whole selection, not
                    // the first cap. An explicit Origin still takes precedence.
                    // World-space resolve, so moved items place the handle right.
                    operandi_core::Editor::extrude_frame(
                        ActionContext {
                            center: CenterMode::Selection,
                            axis: AxisOrientation::World,
                        },
                        l.tool,
                        &self.editor.scene,
                    )
                    .map(|f| f.center)
                    .unwrap_or(DVec3::ZERO)
                } else {
                    item.and_then(|i| {
                        self.editor
                            .scene
                            .selection
                            .faces
                            .first()
                            .map(|&f| i.mesh.center(f as usize))
                    })
                    .unwrap_or(DVec3::ZERO)
                }
            }),
            // The first selected polygon of the drag's base (polygon 0 when
            // nothing is selected and every polygon is affected).
            HandleAnchor::FirstPolygon => l
                .base
                .active()
                .and_then(|i| {
                    let f = l.base.selection.faces.first().copied().unwrap_or(0) as usize;
                    (f < i.mesh.faces.len()).then(|| i.mesh.center(f))
                })
                .unwrap_or(DVec3::ZERO),
            HandleAnchor::ActionCentre => self.transform_handle_center(),
        };
        // Face-center origins above are item-local; map them to world.
        let local_origin = card.anchor.item_local()
            || (l.tool == Tool::Bevel
                && self.editor.action_context.center != CenterMode::Origin
                && self.editor.action_context.center != CenterMode::Selection
                && self.editor.scene.selection.faces.len() <= 1);
        let origin = if polygon_symmetry_centre.is_none()
            && (local_origin
                || (l.tool.is_smooth_shift()
                    && self.editor.action_context.center != CenterMode::Origin))
        {
            item.and_then(|i| self.editor.scene.world_matrix(i.id).ok())
                .map(|m| m.transform_point3(origin))
                .unwrap_or(origin)
        } else {
            origin
        };
        // With Symmetry on, the handle sits on the primary pivot (the pivot
        // service, ACS-01/03), not halfway between the two halves.
        let origin = if matches!(
            card.anchor,
            HandleAnchor::Edge | HandleAnchor::Vertex | HandleAnchor::EdgeRail
        ) && !l.tool.transforms_items()
        {
            self.editor.symmetry_handle_centre().unwrap_or(origin)
        } else {
            origin
        };
        let s = s_at(origin);
        let mut out = Vec::new();
        for axis in 0..card.kind.axis_count() {
            if l.tool == Tool::Rotate {
                let ring = self.rotation_ring(axis, r);
                if let Some((o, _)) = self.camera.project(self.transform_handle_center(), r)
                    && let Some(end) = ring.last()
                {
                    out.push((axis, o, *end, ["X", "Y", "Z"][axis].into()));
                }
                continue;
            }
            let direction = self.handle_axis(axis);
            let length = self.camera.distance * 0.1;
            if let (Some((a, _)), Some((b, _))) = (
                self.camera.project(origin, r),
                self.camera.project(origin + direction * length, r),
            ) && a.distance(b) > 5.
            {
                // ACS-Q-HANDLE1: the box-centre tools' arms start about
                // 25 px out from the centre.
                let gap = if operandi_core::handle_pick(l.tool)
                    == operandi_core::HandlePick::BoxCentre
                    && card.anchor != HandleAnchor::PolygonStage
                {
                    25. * s
                } else {
                    0.
                };
                let dir = (b - a).normalized();
                let a = a + dir * gap;
                out.push((
                    axis,
                    a,
                    a + dir * 72. * s,
                    if l.tool == Tool::Chamfer {
                        "Offset".into()
                    } else if l.tool.is_smooth_shift() {
                        if axis == 0 { "Offset" } else { "Scale" }.into()
                    } else if l.tool == Tool::Bevel {
                        if axis == 0 { "Shift" } else { "Inset" }.into()
                    } else if l.tool == Tool::EdgeExtrude {
                        if axis == 0 { "Extrude" } else { "Width" }.into()
                    } else if l.tool == Tool::EdgeSlide {
                        "Slide".into()
                    } else if l.tool == Tool::EdgeCrease {
                        "Weight".into()
                    } else if l.tool == Tool::EdgeBevel {
                        "Inset".into()
                    } else if matches!(l.tool, Tool::EdgeChamfer | Tool::EdgeRelax) {
                        "Offset".into()
                    } else if l.tool == Tool::VertexBevel {
                        "Inset".into()
                    } else if l.tool == Tool::VertexExtrude {
                        if axis == 0 { "Extrude" } else { "Width" }.into()
                    } else if l.tool == Tool::VertexMerge {
                        "Distance".into()
                    } else if l.tool == Tool::VertexSlide {
                        "Slide".into()
                    } else if l.tool == Tool::Inset {
                        "Inset".into()
                    } else if l.tool == Tool::Shift {
                        "Shift".into()
                    } else {
                        ["X", "Y", "Z"][axis].into()
                    },
                ));
            }
        }
        if matches!(l.tool, Tool::Move | Tool::Rotate)
            && let Some((center, _)) = self.camera.project(self.transform_handle_center(), r)
        {
            out.push((
                FREE_HANDLE,
                center,
                center,
                if l.tool == Tool::Move {
                    "Free move"
                } else {
                    "Free rotate"
                }
                .into(),
            ));
        }
        // The Scale tool's center handle (where the three axis
        // handles meet) scales the selection uniformly.
        if l.tool == Tool::Scale
            && let Some((center, _)) = self.camera.project(origin, r)
        {
            out.push((SCALE_UNIFORM_HANDLE, center, center, "Uniform".into()));
        }
        // PH-30/31: Move's and Scale's plane circles.
        out.extend(self.plane_handles(l.tool, r));
        if l.tool == Tool::Box {
            for axis in 0..3 {
                for maximum in [false, true] {
                    // At zero depth both bounds coincide. One positive-bound
                    // square owns the center hit so upward/outward hauling can
                    // grow a solid instead of selecting the crossed min bound.
                    if l.params.size[axis] == 0. && !maximum {
                        continue;
                    }
                    let mut point = l.params.position;
                    point[axis] += l.params.size[axis] * if maximum { 0.5 } else { -0.5 };
                    if let Some((point, _)) = self.camera.project(point, r) {
                        out.push((
                            3 + axis * 2 + usize::from(maximum),
                            point,
                            point,
                            format!(
                                "{}{}",
                                if maximum { "+" } else { "−" },
                                ["X", "Y", "Z"][axis]
                            ),
                        ));
                    }
                }
            }
        }
        out
    }
    pub(crate) fn handle_axis(&self, axis: usize) -> DVec3 {
        // Frame-based tools point their handles along their card's frame.
        if let Some(l) = self.editor.live.as_ref() {
            match l.tool.card().handles.axes {
                HandleAxes::Tool => {}
                HandleAxes::VertexRail | HandleAxes::VertexNormal => {
                    let Some((_, normal, rail, _)) = self.vertex_frame() else {
                        return DVec3::X;
                    };
                    return if l.tool.card().handles.axes == HandleAxes::VertexNormal && axis == 0 {
                        normal
                    } else if l.tool == Tool::VertexSlide {
                        l.params.delta.normalize_or(rail)
                    } else {
                        rail
                    };
                }
                HandleAxes::EdgeRail => {
                    return self.edge_slide_frame().map(|f| f.1).unwrap_or(DVec3::X);
                }
                HandleAxes::EdgeSide => {
                    return self.edge_handle_frame().map(|f| f.2).unwrap_or(DVec3::X);
                }
                HandleAxes::EdgeNormal => {
                    return self
                        .edge_handle_frame()
                        .map(|(_, up, side)| if axis == 0 { up } else { side })
                        .unwrap_or(DVec3::Z);
                }
            }
        }
        if axis > 2
            && self
                .editor
                .live
                .as_ref()
                .is_some_and(|l| l.tool == Tool::Scale)
        {
            return DVec3::ZERO;
        }
        let Some(l) = &self.editor.live else {
            return DVec3::ZERO;
        };
        let a = [DVec3::X, DVec3::Y, DVec3::Z][axis];
        // Chamfer Edit's single (blue) widget points along the strip normal.
        if l.tool == Tool::Chamfer
            && let Some(i) = l.base.active()
            && let Some(&f) = operandi_core::chamfer_strips(&i.mesh, &l.base.selection.faces)
                .first()
                .filter(|&&f| (f as usize) < i.mesh.faces.len())
        {
            let direction = i.mesh.normal(f as usize);
            return l
                .base
                .world_matrix(i.id)
                .map(|m| m.transform_vector3(direction).normalize_or_zero())
                .unwrap_or(direction);
        }
        // Inset: Bevel's Inset direction (centre away from the first corner);
        // Shift: the polygon's normal. Frozen from the drag's base.
        if matches!(l.tool, Tool::Inset | Tool::Shift)
            && let Some(i) = l.base.active()
        {
            let fi = l.base.selection.faces.first().copied().unwrap_or(0) as usize;
            let Some(f) = i.mesh.faces.get(fi) else {
                return DVec3::Z;
            };
            let direction = if l.tool == Tool::Shift {
                i.mesh.normal(fi)
            } else {
                (i.mesh.center(fi) - i.mesh.vertices[f.vertices[0] as usize]).normalize_or_zero()
            };
            return l
                .base
                .world_matrix(i.id)
                .map(|m| m.transform_vector3(direction).normalize_or_zero())
                .unwrap_or(direction);
        }
        if l.tool.is_smooth_shift()
            && let Some(i) = l.base.active()
        {
            let fi = l.base.selection.faces.first().copied().unwrap_or(0) as usize;
            let f = &i.mesh.faces[fi];
            let direction = if axis == 0 {
                i.mesh.normal(fi)
            } else {
                (i.mesh.center(fi) - i.mesh.vertices[f.vertices[0] as usize]).normalize_or_zero()
            };
            return l
                .base
                .world_matrix(i.id)
                .map(|m| m.transform_vector3(direction).normalize_or_zero())
                .unwrap_or(direction);
        }
        if l.tool == Tool::Bevel {
            if self.editor.action_context.axis == AxisOrientation::Selection
                && let Some(frame) = l.action_frame
            {
                return if axis == 0 {
                    frame.basis.y_axis
                } else {
                    frame.basis.z_axis
                };
            }
            // Existing face-local Bevel roles; freeze directions from stage base.
            if let Some(i) = l.base.active()
                && let Some(&f) = l.base.selection.faces.first()
            {
                let local = if axis == 0 {
                    i.mesh.normal(f as usize)
                } else {
                    let face = &i.mesh.faces[f as usize];
                    (i.mesh.center(f as usize) - i.mesh.vertices[face.vertices[0] as usize])
                        .normalize_or_zero()
                };
                // Face-local directions shown in world space for moved items.
                return l
                    .base
                    .world_matrix(i.id)
                    .map(|m| m.transform_vector3(local).normalize_or_zero())
                    .unwrap_or(local);
            }
            a
        } else if let Some(basis) = self.local_handle_basis() {
            // AC-75: under Local the arrows follow the last selected
            // element's cluster.
            basis * a
        } else if let Some(frame) = l.action_frame {
            frame.basis * a
        } else {
            a
        }
    }
    /// Original-compatible A1: handles have a size in the scene, so they
    /// shrink on screen as the view zooms in on them and grow as it zooms
    /// out; 1 (constant screen size) with A1 off. See `Camera::handle_zoom`.
    pub(crate) fn handle_zoom_at(&self, center: DVec3) -> f32 {
        if crate::original_compatible::is_original(WORLD_SIZE_HANDLES) {
            self.camera.handle_zoom(center) as f32
        } else {
            1.
        }
    }
    /// Handle Scale: Standard times `handle_zoom_at(center)`.
    pub(crate) fn handle_scale_at(&self, center: DVec3) -> f32 {
        self.handle_preferences.scale * self.handle_zoom_at(center)
    }
    /// The live transform handle's frame before the drag's delta: where
    /// the handle sits at `delta = 0` and the axes a unit of `delta` moves
    /// it along. Under Local (AC-75) that's the last selected element's
    /// cluster frame, which is also what the core moves that cluster by;
    /// otherwise the action frame, with the polygon stage's Symmetry
    /// centre for component tools. Snapping measures in this same frame.
    pub(crate) fn transform_handle_frame(&self) -> Option<operandi_core::ActionFrame> {
        let l = self.editor.live.as_ref()?;
        let frame = l.action_frame?;
        // The pivot service places the frame (ACS-01/07); the polygon
        // stage's handle follows its live caps.
        if let Some(local) = self.local_handle_frame() {
            return Some(local);
        }
        let center = if l.tool.transforms_items() {
            frame.center
        } else {
            self.editor.symmetry_handle_centre().unwrap_or(frame.center)
        };
        Some(operandi_core::ActionFrame {
            center,
            basis: frame.basis,
        })
    }
    pub(crate) fn transform_handle_center(&self) -> DVec3 {
        let Some(l) = self.editor.live.as_ref() else {
            return DVec3::ZERO;
        };
        self.transform_handle_frame()
            .map(|frame| {
                frame.center
                    + if matches!(l.tool, Tool::Move | Tool::Transform | Tool::EdgeExtend) {
                        frame.basis * l.params.delta
                    } else {
                        DVec3::ZERO
                    }
            })
            .unwrap_or(DVec3::ZERO)
    }
    pub(super) fn rotation_ring(&self, axis: usize, r: Rect) -> Vec<Pos2> {
        let center = self.transform_handle_center();
        let Some((_, depth)) = self.camera.project(center, r) else {
            return vec![];
        };
        let radius = if self
            .editor
            .live
            .as_ref()
            .is_some_and(|l| combined_handles(l.tool))
        {
            96.
        } else {
            72.
        } * self.handle_scale_at(center) as f64;
        let world_radius = radius
            * if self.camera.view == View::Perspective {
                depth
            } else {
                self.camera.distance
            }
            / (r.height() as f64 * 0.9);
        let u = self.handle_axis((axis + 1) % 3);
        let v = self.handle_axis((axis + 2) % 3);
        (0..=40)
            .filter_map(|i| {
                let angle = i as f64 * std::f64::consts::TAU / 48.;
                self.camera
                    .project(
                        center + (u * angle.cos() + v * angle.sin()) * world_radius,
                        r,
                    )
                    .map(|p| p.0)
            })
            .collect()
    }
    /// The handle under `p`. PH-11 (provisional order): the center handle,
    /// then the arrows and rings, then the plane circles.
    pub(crate) fn handle_hit(&self, p: Pos2, r: Rect) -> Option<usize> {
        // The Snaps and Precision presets' handles sit on top.
        if let Some(id) = self.precision_hit(p, r) {
            return Some(id);
        }
        match self.handle_hit_axes(p, r) {
            // Transform's rotation rings pass through the corner where its
            // plane circles sit (PH-07): a press on a circle takes the
            // circle, while a press on the ring elsewhere still turns.
            Some(ring)
                if (3..6).contains(&ring)
                    && self
                        .editor
                        .live
                        .as_ref()
                        .is_some_and(|l| combined_handles(l.tool)) =>
            {
                self.plane_handle_hit(p, r).or(Some(ring))
            }
            Some(hit) => Some(hit),
            None => self.plane_handle_hit(p, r),
        }
    }
    pub(super) fn handle_hit_axes(&self, p: Pos2, r: Rect) -> Option<usize> {
        if self
            .editor
            .live
            .as_ref()
            .is_some_and(|l| l.tool.card().handles.kind == HandleKind::Spine)
        {
            return self.twist_bevel_hit(p, r);
        }
        if let Some(live) = &self.editor.live
            && matches!(live.tool, Tool::Move | Tool::Rotate | Tool::Transform)
            && let Some(free) = self.handles(r).into_iter().find(|h| {
                h.0 == FREE_HANDLE && Rect::from_center_size(h.2, vec2(12., 12.)).contains(p)
            })
        {
            return Some(free.0);
        }
        if let Some(live) = &self.editor.live
            && combined_handles(live.tool)
        {
            let handles = self.handles(r);
            let mut hits: Vec<_> = handles
                .iter()
                .filter(|h| {
                    !(3..6).contains(&h.0) && !is_plane_handle(h.0) && p.distance(h.2) <= 7.
                })
                .map(|h| h.0)
                .collect();
            if hits.is_empty() {
                for axis in 0..3 {
                    // Edge Extend draws its rings only with Rotate on.
                    if !handles.iter().any(|h| h.0 == axis + 3) {
                        continue;
                    }
                    if self
                        .rotation_ring(axis, r)
                        .windows(2)
                        .any(|s| input::segment_distance(p, s[0], s[1]) <= 5.)
                    {
                        hits.push(axis + 3);
                    }
                }
            }
            return if hits.len() == 1 { Some(hits[0]) } else { None };
        }
        if self
            .editor
            .live
            .as_ref()
            .is_some_and(|l| l.tool == Tool::Rotate)
        {
            return (0..3).find(|&axis| {
                self.rotation_ring(axis, r)
                    .windows(2)
                    .any(|s| input::segment_distance(p, s[0], s[1]) <= 5.)
            });
        }
        let handles = self.handles(r);
        if let Some(center) = handles.iter().find(|h| {
            h.0 == SCALE_UNIFORM_HANDLE
                && self
                    .editor
                    .live
                    .as_ref()
                    .is_some_and(|l| l.tool == Tool::Scale)
                && self.scale_center_contains(h.2, p)
        }) {
            return Some(center.0);
        }
        if self
            .editor
            .live
            .as_ref()
            // Every primitive the same way (Jason, 2026-11-21): its side
            // squares first, then the move arrows along their whole shaft,
            // not only at the tip (Sphere and Cylinder took the tip only,
            // so a press on the arrow dragged the nearest side instead).
            .is_some_and(|live| live.tool.is_primitive())
            && let Some(side) = handles.iter().filter(|h| h.0 >= 3).find(|h| {
                Rect::from_center_size(h.2, Vec2::splat(self.handle_preferences.point_hit_size))
                    .contains(p)
            })
        {
            return Some(side.0);
        }
        handles
            .iter()
            .filter(|h| !is_plane_handle(h.0))
            .find(|(_, a, b, _)| {
                Rect::from_center_size(*b, Vec2::splat(self.handle_preferences.point_hit_size))
                    .contains(p)
                    || input::segment_distance(p, *a, *b)
                        <= self.handle_preferences.line_hit_width / 2.
            })
            .map(|h| h.0)
    }
    /// Edge Slide's handle frame (`HandleAnchor::EdgeRail`), once per drag;
    /// see `operandi_core::edge_slide_handle_frame`.
    pub(super) fn edge_slide_frame(&self) -> Option<(DVec3, DVec3, f64)> {
        let l = self.editor.live.as_ref()?;
        let symmetry = self.editor.active_symmetry();
        self.edge_slide_frame_memo.get_with(
            &l.base,
            (symmetry, operandi_core::HandlePick::FirstPicked),
            || operandi_core::edge_slide_handle_frame(&l.base, symmetry),
        )
    }
    /// The vertex tools' handle frame (`HandleAnchor::Vertex`), once per
    /// drag; see `operandi_core::vertex_handle_frame`.
    pub(super) fn vertex_frame(&self) -> Option<(DVec3, DVec3, DVec3, f64)> {
        let l = self.editor.live.as_ref()?;
        let symmetry = self.editor.active_symmetry();
        let pick = operandi_core::handle_pick(l.tool);
        self.vertex_frame_memo
            .get_with(&l.base, (symmetry, pick), || {
                operandi_core::vertex_handle_frame(&l.base, symmetry, pick)
            })
    }
    /// The edge tools' handle frame (`HandleAnchor::Edge`), once per drag;
    /// see `operandi_core::edge_handle_frame`.
    pub(super) fn edge_handle_frame(&self) -> Option<(DVec3, DVec3, DVec3)> {
        let l = self.editor.live.as_ref()?;
        let symmetry = self.editor.active_symmetry();
        let pick = operandi_core::handle_pick(l.tool);
        self.edge_frame_memo
            .get_with(&l.base, (symmetry, pick), || {
                operandi_core::edge_handle_frame(&l.base, symmetry, pick)
            })
    }
}
