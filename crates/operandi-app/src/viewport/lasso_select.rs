//! Lasso selection: what a finished lasso loop selects on each mesh.
use super::*;

impl Operandi {
    pub(crate) fn finish_lasso(&mut self, path: &[Pos2], middle: bool, mods: Modifiers, r: Rect) {
        // Half a point of tolerance: invisible on screen, but a hand-drawn
        // loop drops from thousands of points to a few dozen.
        let region = lasso::simplify(&lasso::outline(self.lasso_style, path), 0.5);
        let result = self.lasso_hits(&region, middle, r);
        match result {
            Ok(meshes) => {
                if !mods.shift && !mods.ctrl {
                    self.run(Command::ClearSelection);
                }
                let how = if mods.ctrl {
                    ClickHow::Remove
                } else {
                    ClickHow::Add
                };
                for (item, hits) in meshes {
                    for hit in hits {
                        self.click_select(Click {
                            hit: Some((item, hit)),
                            how,
                            over_skipped_mesh: false,
                        });
                    }
                }
                self.run(Command::EndSelection { cancel: false });
                self.status =
                    "Lasso selected enclosed components not hidden behind a surface".into();
            }
            Err(error) => {
                self.run(Command::EndSelection { cancel: true });
                self.status = error;
            }
        }
    }
    pub(crate) fn lasso_hits(
        &self,
        region: &[Pos2],
        middle: bool,
        r: Rect,
    ) -> operandi_core::Result<Vec<(u64, Vec<Component>)>> {
        if !lasso::valid(region) {
            return Err("Lasso requires a nonzero-area shape".into());
        }
        // Every shown active mesh (multi_active.rs), primary first.
        let shown = self.editor.scene.shown_active_meshes();
        let meshes: Vec<&operandi_core::Item> = shown
            .iter()
            .filter_map(|id| self.editor.scene.item(*id))
            .collect();
        if meshes.is_empty() {
            return Err("Lasso requires an active visible mesh".into());
        }
        // Curves and polylines lasso like polygons: their points are mesh
        // points, their spans are edges and each line is one element
        // (Jason's QA, 2026-11-10: a mesh holding a curve refused the
        // lasso outright, so nothing on it could be lassoed).
        let corners: usize = meshes
            .iter()
            .flat_map(|i| i.mesh.faces.iter())
            .map(|f| f.vertices.len())
            .sum();
        // Each corner is tested against each outline segment a few times;
        // corners outside the outline's bounds are rejected up front.
        if corners.saturating_mul(region.len()) > 50_000_000 {
            return Err("Lasso exceeds the 50-million intersection-work budget; shorten the path or reduce geometry".into());
        }
        meshes
            .into_iter()
            .map(|item| Ok((item.id, self.lasso_mesh(item, region, middle, r)?)))
            .collect()
    }
    /// One mesh's components inside the lasso `region` (see `lasso_hits`).
    pub(super) fn lasso_mesh(
        &self,
        item: &operandi_core::Item,
        region: &[Pos2],
        middle: bool,
        r: Rect,
    ) -> operandi_core::Result<Vec<Component>> {
        let partial = self.lasso_partial || self.lasso_partial_held;
        // Shaded/right culls; shaded/middle through; Wireframe reverses it.
        // Polygon Selection and Lasso: Ignore Shading Style (preferences).
        let prefs = self.selection_preferences;
        let cull = prefs
            .polygon_selection
            .lasso_culls(self.wireframe_view() && !prefs.lasso_ignore_shading, middle);
        // Where the points show (on a smooth surface, not its cage).
        let world: Vec<_> = self
            .shown_points(item)
            .iter()
            .map(|&v| self.editor.scene.world_point(item.id, v))
            .collect();
        // Edges and polygons test hiding at their cage centres, as before:
        // a chord between two surface points lies inside a convex surface.
        let cage: Vec<_> = item
            .mesh
            .vertices
            .iter()
            .map(|&v| self.editor.scene.world_point(item.id, v))
            .collect();
        let projected: Vec<_> = world
            .iter()
            .map(|&v| self.camera.project(v, r).map(|p| p.0))
            .collect();
        // Padded past the 0.001 on-the-line tolerance of `lasso::contains`.
        let bounds = Rect::from_points(region).expand(0.01);
        let accepts = |vertices: &[u32]| {
            let points: Option<Vec<_>> = vertices.iter().map(|&v| projected[v as usize]).collect();
            points.is_some_and(|points| {
                // Cheap bounds check first: a full enclosure needs every
                // corner inside the outline's bounds, a partial one needs
                // the element's bounds to touch them.
                let element = Rect::from_points(&points);
                let near = if partial {
                    element.intersects(bounds)
                } else {
                    bounds.contains_rect(element)
                };
                near && lasso::eligible(region, &points, partial)
            })
        };
        // Components use the same test as clicking (Jason, 2026-10-31 for
        // points, 2026-11-01 for edges and polygons): eligible unless a
        // surface lies in front of them. Edges are tested at their midpoint
        // and polygons at their center. This replaces a facing-only rule
        // that skipped unobstructed parts whose polygons face away.
        let surface = self.pick_surface();
        let unhidden = |at: DVec3| !cull || surface.visible(&self.camera, at, r);
        let center = |vertices: &[u32]| {
            vertices.iter().map(|&v| cage[v as usize]).sum::<DVec3>() / vertices.len() as f64
        };
        let mut hits = Vec::new();
        match self.editor.scene.selection.mode {
            Mode::Polygon => {
                for (index, face) in item.mesh.faces.iter().enumerate() {
                    if accepts(&face.vertices) && unhidden(center(&face.vertices)) {
                        hits.push(Component::Polygon(index as u32));
                    }
                }
            }
            Mode::Vertex => {
                for vertex in 0..item.mesh.vertices.len() as u32 {
                    if accepts(&[vertex]) && unhidden(world[vertex as usize]) {
                        hits.push(Component::Vertex(vertex));
                    }
                }
            }
            Mode::Edge => {
                for (a, b) in item.mesh.edges() {
                    if accepts(&[a, b]) && unhidden(center(&[a, b])) {
                        hits.push(Component::Edge(a, b));
                    }
                }
            }
            Mode::Item => {
                return Err(
                    "Item lasso is not supported by the current single-item selection state".into(),
                );
            }
        }
        Ok(hits)
    }
}
