//! Selecting with clicks: a click, Shift/Ctrl click, fill press, double
//! clicks for connected polygons and edge loops, Mouse Regions, and the
//! selection status line.
use super::*;

impl Operandi {
    /// Mouse Regions Trigger Selections: a right-click on a point, edge or
    /// polygon switches to that element's mode and selects it. False (and
    /// nothing happens) when the preference is off or nothing is there.
    pub(crate) fn mouse_region_click(&mut self, p: Pos2, r: Rect) -> bool {
        if !self.selection_preferences.mouse_regions {
            return false;
        }
        let Some((item, hit)) = self.pick_closest(p, r) else {
            return false;
        };
        let mode = match hit {
            Component::Vertex(_) => Mode::Vertex,
            Component::Edge(..) => Mode::Edge,
            Component::Polygon(_) => Mode::Polygon,
            _ => return false,
        };
        if self.editor.scene.selection.mode != mode {
            self.run(Command::Mode(mode));
        }
        self.click_select(Click {
            hit: Some((item, hit)),
            how: ClickHow::Replace,
            over_skipped_mesh: false,
        });
        true
    }
    /// Polygon Selection Fill: Shift+Cmd click (Shift+Ctrl off macOS) on a
    /// polygon inside a selected perimeter (Jason decision 2026-11-01,
    /// SEL-Q-M3); on any other polygon, the row between it and the last pick
    /// (SEL-PHY-03). Returns the gesture when the press belongs to Fill; other
    /// presses, including plain Shift+click, fall through to ordinary picking.
    pub(crate) fn fill_press(
        &mut self,
        pos: Pos2,
        r: Rect,
        mods: Modifiers,
    ) -> Option<input::GestureKind> {
        // `command` is Cmd on macOS and Ctrl elsewhere (synthetic events may
        // carry only `mac_cmd`).
        if self.editor.scene.selection.mode != Mode::Polygon
            || !mods.shift
            || !(mods.command || mods.mac_cmd)
            || mods.alt
        {
            return None;
        }
        match self.pick_component(pos, r) {
            Some((item, Component::Polygon(face))) => {
                // The clicked mesh's polygons (any active mesh).
                let before = self.editor.scene.selection.faces_of(item).to_vec();
                if !before.contains(&face) {
                    self.run(Command::SelectionFill { item, face });
                }
                let after = self.editor.scene.selection.faces_of(item).len();
                if after > before.len() + 1 {
                    self.status = format!("Selection Fill: {after} polygons selected");
                } else if !before.is_empty() && !before.contains(&face) {
                    // SEL-PHY-03: outside a selected perimeter (Fill found
                    // nothing but the polygon itself), the click picks the
                    // polygon and selects the row between it and the last
                    // pick.
                    if after == before.len() {
                        self.click_select(Click {
                            hit: Some((item, Component::Polygon(face))),
                            how: ClickHow::Add,
                            over_skipped_mesh: false,
                        });
                    }
                    self.execute("Select Between");
                    self.status = format!(
                        "Select Between: {} polygons selected",
                        self.editor.scene.selection.faces.len()
                    );
                }
            }
            _ => {
                self.status =
                    "Selection Fill: Shift+Cmd click a polygon inside the selected perimeter".into()
            }
        }
        Some(input::GestureKind::Blocked)
    }
    /// Polygon mode: the second press of a double-click on a polygon selects
    /// every polygon connected to it (Shift adds that piece to the
    /// selection). Like the edge-loop double-click, the first press was an
    /// ordinary click and its own undo step; the piece is one more "Select
    /// Connected" step. Any other press falls through to ordinary picking.
    pub(crate) fn polygon_connected_double_click(
        &mut self,
        pos: Pos2,
        r: Rect,
        mods: Modifiers,
        now: f64,
        delay: f64,
    ) -> Option<input::GestureKind> {
        if self.editor.scene.selection.mode != Mode::Polygon {
            return None;
        }
        // Taken, not peeked: after a double-click, a third press starts over.
        let (first_time, first_pos) = self.last_component_press.take()?;
        if mods.ctrl
            || mods.alt
            || mods.command
            || mods.mac_cmd
            || now - first_time > delay
            || first_pos.distance(pos) > EDGE_DOUBLE_CLICK_DISTANCE
        {
            return None;
        }
        // Picked only on the second press, so an ordinary click on a big
        // mesh still costs one pick.
        let Some((item, Component::Polygon(face))) = self.pick_component(pos, r) else {
            return None;
        };
        // UIC-160: report the result of the same command the gesture dispatches.
        if self.run_ok(Command::SelectConnectedPolygons {
            item,
            face,
            add: mods.shift,
        }) {
            let total = self.editor.scene.selection.faces_of(item).len();
            self.status = format!("Select Connected: {total} selected Polygons");
        }
        Some(input::GestureKind::Blocked)
    }
    /// Edge mode: the second press of a double-click on an edge selects
    /// that edge's loop (Shift adds the loop to the selection). The first
    /// press was an ordinary click, so it already picked the edge as its own
    /// undo step; the loop is one more "Select Loop" step. Returns the
    /// gesture when the press is that second press; any other press falls
    /// through to ordinary picking (and the caller remembers it as a
    /// possible first press).
    pub(crate) fn edge_loop_double_click(
        &mut self,
        pos: Pos2,
        r: Rect,
        mods: Modifiers,
        now: f64,
        delay: f64,
    ) -> Option<input::GestureKind> {
        // Taken, not peeked: after a double-click, a third press starts over.
        let (first_time, first_pos) = self.last_component_press.take()?;
        if self.editor.scene.selection.mode != Mode::Edge
            || mods.ctrl
            || mods.alt
            || mods.command
            || mods.mac_cmd
            || now - first_time > delay
            || first_pos.distance(pos) > EDGE_DOUBLE_CLICK_DISTANCE
        {
            return None;
        }
        // Picked only on the second press, so an ordinary click on a big
        // mesh still costs one pick.
        let Some((item, Component::Edge(a, b))) = self.pick_component(pos, r) else {
            return None;
        };
        let before = self.editor.scene.selection.edges_of(item).len();
        self.run(Command::SelectEdgeLoop {
            item,
            a,
            b,
            add: mods.shift,
        });
        let after = self.editor.scene.selection.edges_of(item).len();
        // With Shift the first press already added the clicked edge.
        let grew = if mods.shift {
            after > before
        } else {
            after > 1
        };
        self.status = if grew {
            format!("Select Loop: {after} selected Edge")
        } else {
            format!("Edge loop stops here; {after} selected Edge")
        };
        Some(input::GestureKind::Blocked)
    }
    /// Whether painting `hit` (adding it, or removing it with `remove`)
    /// would change the selection, counting its Symmetry counterpart.
    pub(super) fn paint_changes(&self, item: u64, hit: Component, remove: bool) -> bool {
        let scene = &self.editor.scene;
        let s = &scene.selection;
        // A component of any active mesh paints into that mesh's selection;
        // Shift/Ctrl item paints add and remove items.
        if hit != Component::Item
            && s.item != Some(item)
            && !scene.shown_active_meshes().contains(&item)
        {
            return true;
        }
        let selected = |c: Component| match c {
            Component::Vertex(v) => s.vertices_of(item).contains(&v),
            Component::Edge(a, b) => s.edges_of(item).contains(&(a.min(b), a.max(b))),
            Component::Polygon(f) => s.faces_of(item).contains(&f),
            Component::Item => s.item == Some(item) || s.also.contains(&item),
        };
        let mirror = scene.symmetric_counterpart(item, hit, self.editor.active_symmetry());
        [Some(hit), mirror]
            .into_iter()
            .flatten()
            .any(|c| selected(c) == remove)
    }
    /// Picks at `p` and applies the click; true when it hit a component.
    pub(crate) fn select_at(&mut self, p: Pos2, r: Rect, mods: Modifiers) -> bool {
        let hit = self.pick_component(p, r);
        let how = ClickHow::from_keys(mods.shift, mods.ctrl);
        // Only a plain click on nothing needs to know what it missed.
        let over_skipped_mesh =
            hit.is_none() && how == ClickHow::Replace && self.on_other_mesh(p, r);
        if !self.click_select(Click {
            hit,
            how,
            over_skipped_mesh,
        }) {
            return hit.is_some();
        }
        if hit.is_some() {
            // SL-03: Shortest Path Mode adds the path to a Shift+clicked element.
            self.shortest_path_after_click(mods.shift, mods.ctrl);
        }
        self.selection_status();
        hit.is_some()
    }
    /// Every viewport selection click ends here (AR-23): the hit becomes
    /// a selection change in `click_select::resolve`, so a later selection
    /// style changes one place. False when only the active mesh picks
    /// here (the status line says so).
    pub(crate) fn click_select(&mut self, click: Click) -> bool {
        match click_select::resolve(&self.editor.scene, click) {
            click_select::Outcome::Run(command) => self.run(*command),
            click_select::Outcome::Nothing => {}
            click_select::Outcome::OnlyActiveMeshPicks => {
                self.status =
                    "Only the active mesh picks here (Preferences > Selection: Pick Other \
                     Meshes in Component Modes)"
                        .into();
                return false;
            }
        }
        true
    }
    /// Whether `p` is over a mesh that limited picking (`pick_only`) skips.
    pub(super) fn on_other_mesh(&self, p: Pos2, r: Rect) -> bool {
        let only = self.pick_only();
        !only.is_empty()
            && self
                .pick_surface()
                .pick(&self.camera, p, r)
                .is_some_and(|(id, _, _)| !only.contains(&id))
    }
    /// "N selected <mode>" in the status line.
    pub(super) fn selection_status(&mut self) {
        let s = &self.editor.scene.selection;
        // Components count over every active mesh.
        self.status = format!(
            "{} selected {:?}",
            match s.mode {
                Mode::Vertex | Mode::Edge | Mode::Polygon => s.component_count(),
                Mode::Item => usize::from(s.item.is_some()),
            },
            s.mode
        );
    }
}
