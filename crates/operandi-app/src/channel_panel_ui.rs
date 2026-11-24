//! The channel panel (BEHAVIOR-CHANNEL-PANEL CP-30..54, UIC-188): the
//! current channel set's rows in an in-view panel over the active 3D view.
//! Each row is `Item : Channel`, its status light (the Properties light,
//! UIC-112: click to key) and its value field (channel_row.rs: drag to
//! haul live, click to type). The header carries the set's name, a close
//! (×) at its left end, and drags the panel inside the view; its place is
//! kept per view as a fraction of the view (layout state, not scene data).
//! With no set current, Show Animated lists the selected items' animated
//! channels instead (CP-34).
//!
//! The timeline's Channel Set menu picks the current set and makes,
//! fills and empties sets (CP-10/20/22); a row's ⋯ menu moves or removes
//! it (CP-23, provisional: no drag-to-reorder yet).
use super::*;
use crate::unavailable::Gate;
use operandi_core::animation::ChannelKey;

/// Why Add/Remove Selected Channels are greyed.
const NO_PICK: &str = "Click rows' labels to pick channels first";

/// Where the panel sits in a view: its top-left as a fraction of the view.
const DEFAULT_AT: Vec2 = vec2(0.02, 0.08);

fn show_animated_id() -> Id {
    Id::new("channel-panel-show-animated")
}

/// `Item : Channel` with display names (CP-40).
pub(crate) fn row_label(scene: &operandi_core::Scene, k: &ChannelKey) -> String {
    let item = match scene.item(k.item) {
        Some(i) => i.name.clone(),
        None => scene
            .animation
            .rig
            .piece(k.item)
            .map(|p| operandi_core::rig::label(&p.kind))
            .unwrap_or_else(|| k.item.to_string()),
    };
    let channel = scene
        .animation
        .rig
        .user_channel(k.item, &k.channel)
        .filter(|u| !u.display.is_empty())
        .map(|u| {
            let part = k.channel.strip_prefix(&u.name).unwrap_or("");
            format!("{}{part}", u.display)
        })
        .unwrap_or_else(|| k.channel.clone());
    format!("{item} : {channel}")
}

impl Operandi {
    /// The rows the panel shows now and its title: the current set's, or
    /// with Show Animated the selected items' animated channels (CP-34).
    fn channel_panel_rows(&self, ctx: &Context) -> Option<(Option<u64>, String, Vec<ChannelKey>)> {
        let s = &self.editor.scene;
        if let Some(set) = s.current_channel_set() {
            let rows = set
                .members
                .iter()
                .filter(|k| operandi_core::animation::read(s, k).is_some())
                .cloned()
                .collect();
            return Some((Some(set.id), set.name.clone(), rows));
        }
        if !ctx.data(|d| d.get_temp::<bool>(show_animated_id()).unwrap_or(false)) {
            return None;
        }
        let selected = s.selected_items();
        let rows: Vec<ChannelKey> = s
            .animation
            .channels
            .iter()
            .filter(|(k, c)| selected.contains(&k.item) && !c.curve.is_empty())
            .map(|(k, _)| k.clone())
            .collect();
        Some((None, "Animated".into(), rows))
    }

    /// Draws the panel over the active 3D view; returns where (so the view
    /// doesn't take clicks there).
    pub(super) fn channel_panel(&mut self, ctx: &Context) -> Option<Rect> {
        if !crate::anim_switch::enabled() {
            return None;
        }
        let (set, title, rows) = self.channel_panel_rows(ctx)?;
        let view = self
            .view_rects
            .get(&self.active_view)
            .copied()
            .unwrap_or(self.viewport_rect);
        if view.width() < 80. || view.height() < 60. {
            return None;
        }
        let at_id = Id::new(("channel-panel-at", self.active_view));
        let at: Vec2 = ctx.data_mut(|d| d.get_persisted(at_id).unwrap_or(DEFAULT_AT));
        let pos = view.min + vec2(at.x * view.width(), at.y * view.height());
        let mut lines: Vec<String> = Vec::new();
        let mut close = false;
        let mut moved = Vec2::ZERO;
        let shown = egui::Area::new(Id::new("channel-panel"))
            .order(egui::Order::Foreground)
            .constrain_to(view)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::popup(&ctx.style())
                    .fill(theme::popup_fill())
                    .show(ui, |ui| {
                        ui.set_max_width(360.);
                        // UIC-188: the header: × at its left end, the name,
                        // and a strip that drags the panel.
                        let header = ui.horizontal(|ui| {
                            if ui
                                .small_button("×")
                                .on_hover_text("Close the panel (the set is kept)")
                                .clicked()
                            {
                                close = true;
                            }
                            ui.strong(&title);
                            ui.allocate_space(vec2(ui.available_width().max(12.), 14.));
                        });
                        let grip = ui.interact(
                            header.response.rect,
                            Id::new("channel-panel-header"),
                            Sense::drag(),
                        );
                        if grip.dragged() {
                            moved = grip.drag_delta();
                        }
                        if grip.hovered() || grip.dragged() {
                            ctx.set_cursor_icon(egui::CursorIcon::Grab);
                        }
                        ui.separator();
                        if rows.is_empty() {
                            ui.weak(if set.is_some() {
                                "No channels yet: select rows' labels, then Channel Set > Add Selected Channels"
                            } else {
                                "The selected items have no keys"
                            });
                        }
                        ui.spacing_mut().item_spacing.y = crate::form::layout::panel::ROW_GAP;
                        crate::form::rows::scope(ui, "channel-panel", |ui| {
                            for (i, k) in rows.iter().enumerate() {
                                let label = row_label(&self.editor.scene, k);
                                self.anim_row_keys(std::slice::from_ref(k), false);
                                ui.horizontal(|ui| {
                                    let r = crate::form::rows::row(ui, &label, |ui| {
                                        self.channel_field(ui, k, &mut lines)
                                    });
                                    let _ = r;
                                    if let Some(set) = set {
                                        let menu = ui.small_button("⋯").on_hover_text("Move or remove this row");
                                        egui::Popup::menu(&menu).show(|ui| {
                                            let text = k.text();
                                            if ui.gated(crate::unavailable::unless(i > 0, "Already the top row"), egui::Button::new("Move Up")).clicked() {
                                                lines.push(format!("operandi.channelSet.move set:{set} channel:{text} by:-1"));
                                            }
                                            if ui.gated(crate::unavailable::unless(i + 1 < rows.len(), "Already the bottom row"), egui::Button::new("Move Down")).clicked() {
                                                lines.push(format!("operandi.channelSet.move set:{set} channel:{text} by:1"));
                                            }
                                            if ui.button("Remove From Set").clicked() {
                                                lines.push(format!("operandi.channelSet.remove set:{set} channels:{text}"));
                                            }
                                        });
                                    }
                                });
                            }
                        });
                    })
                    .response
                    .rect
            });
        if moved != Vec2::ZERO {
            let size = shown.inner.size();
            let mut p = pos + moved;
            p.x = p.x.clamp(view.min.x, (view.max.x - size.x).max(view.min.x));
            p.y = p.y.clamp(view.min.y, (view.max.y - size.y).max(view.min.y));
            let frac = vec2(
                (p.x - view.min.x) / view.width(),
                (p.y - view.min.y) / view.height(),
            );
            ctx.data_mut(|d| d.insert_persisted(at_id, frac));
        }
        if close {
            if set.is_some() {
                lines.push("operandi.channelSet.current set:none".into());
            } else {
                ctx.data_mut(|d| d.insert_temp(show_animated_id(), false));
            }
        }
        // The rows' key clicks and label picks (UIC-112/119).
        lines.extend(crate::form::rows::take_anim_requests());
        for l in lines {
            self.execute(&l);
        }
        Some(shown.inner)
    }

    /// The timeline's Channel Set menu (CP-10/20/22/30/34).
    pub(super) fn channel_set_menu(&mut self, ui: &mut Ui) {
        if !crate::anim_switch::enabled() {
            return;
        }
        let s = &self.editor.scene;
        let current = s.current_channel_set().map(|c| (c.id, c.name.clone()));
        let sets: Vec<(u64, String)> = s
            .animation
            .channel_sets
            .iter()
            .map(|c| (c.id, c.name.clone()))
            .collect();
        let selected = self.editor.anim.selected_channels.len();
        let text = format!(
            "Channel Set: {} ⏷",
            current.as_ref().map_or("(none)", |c| c.1.as_str())
        );
        let r = ui
            .button(text)
            .on_hover_text("The channel panel over the 3D view (CP-30): pick a set, make one from the selected channels (click rows' labels to select them), or show the selected items' animated channels");
        let mut lines: Vec<String> = Vec::new();
        let draft = Id::new("channel-set-name-draft");
        egui::Popup::menu(&r)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                if ui.radio(current.is_none(), "(none)").clicked() {
                    lines.push("operandi.channelSet.current set:none".into());
                }
                for (id, name) in &sets {
                    if ui
                        .radio(current.as_ref().is_some_and(|c| c.0 == *id), name)
                        .clicked()
                    {
                        lines.push(format!("operandi.channelSet.current set:{id}"));
                    }
                }
                ui.separator();
                let mut show = ui.data(|d| d.get_temp::<bool>(show_animated_id()).unwrap_or(false));
                if ui
                    .checkbox(&mut show, "Show Animated")
                    .on_hover_text(
                        "With no set picked, list the selected items' animated channels (CP-34)",
                    )
                    .changed()
                {
                    ui.data_mut(|d| d.insert_temp(show_animated_id(), show));
                }
                ui.separator();
                let mut name: String =
                    ui.data_mut(|d| d.get_temp(draft).unwrap_or_else(|| "Channel Set".into()));
                ui.horizontal(|ui| {
                    ui.label("Name");
                    let field = ui.add(egui::TextEdit::singleline(&mut name).desired_width(140.));
                    self.text_editors.insert(field.id);
                    crate::numeric_tab::register(ui, &field, Some(&name));
                });
                ui.data_mut(|d| d.insert_temp(draft, name.clone()));
                let quoted = name.replace('"', "'");
                if ui
                    .button(format!("Create Channel Set ({selected} selected)"))
                    .on_hover_text("operandi.channelSet.create")
                    .clicked()
                {
                    lines.push(format!("operandi.channelSet.create name:\"{quoted}\""));
                    ui.close();
                }
                if let Some((id, _)) = &current {
                    if ui
                        .gated(
                            crate::unavailable::unless(selected > 0, NO_PICK),
                            egui::Button::new("Add Selected Channels"),
                        )
                        .clicked()
                    {
                        lines.push(format!("operandi.channelSet.add set:{id}"));
                    }
                    if ui
                        .gated(
                            crate::unavailable::unless(selected > 0, NO_PICK),
                            egui::Button::new("Remove Selected Channels"),
                        )
                        .clicked()
                    {
                        lines.push(format!("operandi.channelSet.remove set:{id}"));
                    }
                    if ui.button("Rename to Name").clicked() {
                        lines.push(format!(
                            "operandi.channelSet.rename set:{id} name:\"{quoted}\""
                        ));
                    }
                    if ui.button("Delete Set").clicked() {
                        lines.push(format!("operandi.channelSet.delete set:{id}"));
                        ui.close();
                    }
                }
            });
        for l in lines {
            self.execute(&l);
        }
    }
}
