//! The active item's rig user channels (RG-53) as sliders on the User
//! Channels tab: an imported rig's controls (an arm angle, wings, landing
//! gear) and the ones Add User Channel makes in the Channel Links window.
//! Dragging is live (UIC-51); the release, or a typed value on Enter, is
//! `operandi.rig.userChannelSet`, one undo step that Auto Key keys like
//! any channel edit (UIC-112: keyed rows show their state the
//! way the rest of the panel does is a follow-up).
use super::*;
use operandi_core::rig::UserType;

/// The pointer is locked in place for a slider drag on macOS (as the view
/// corner buttons do, VBF-2); elsewhere the field follows the pointer.
const LOCK_POINTER: bool = cfg!(target_os = "macos") && !cfg!(test);

/// How far a slider moves per point of mouse travel, in the units it shows:
/// 0.5 % or 0.5° a point, 0.01 m or 0.01 a point (Shift ×0.1, Ctrl ×10).
fn drag_rate(ty: UserType) -> f64 {
    match ty {
        UserType::Percentage | UserType::Angle => 0.5,
        UserType::Integer | UserType::Choice => 0.1,
        _ => 0.01,
    }
}

impl Operandi {
    /// Draws the rows; returns whether there were any.
    pub(super) fn rig_user_channel_rows(&mut self, ui: &mut Ui) -> bool {
        let Some(item) = self.editor.scene.active().map(|i| i.id) else {
            return false;
        };
        let channels: Vec<operandi_core::rig::UserChannel> = self
            .editor
            .scene
            .animation
            .rig
            .user
            .iter()
            .filter(|u| u.item == item)
            .cloned()
            .collect();
        if channels.is_empty() {
            return false;
        }
        let mut lines = Vec::new();
        for u in &channels {
            let label = if u.display.is_empty() {
                u.name.clone()
            } else {
                u.display.clone()
            };
            if u.ty == UserType::Divider {
                ui.separator();
                ui.strong(label);
                continue;
            }
            for name in u
                .channel_names()
                .into_iter()
                .chain((u.ty == UserType::String).then(|| u.name.clone()))
            {
                let part = name.strip_prefix(&u.name).unwrap_or("");
                ui.horizontal(|ui| {
                    crate::option_row::label(ui, &format!("{label}{part}"))
                        .on_hover_text(format!("{name} ({})", u.ty.name()));
                    if u.ty == UserType::String {
                        ui.label(&u.text);
                        return;
                    }
                    let value = operandi_core::animation::read(
                        &self.editor.scene,
                        &operandi_core::animation::ChannelKey::new(item, &name),
                    )
                    .unwrap_or(u.default);
                    let set = |v: f64| {
                        format!("operandi.rig.userChannelSet item:{item} name:{name} value:{v}")
                    };
                    if u.ty == UserType::Boolean {
                        let mut on = value >= 0.5;
                        if ui.checkbox(&mut on, "").changed() {
                            lines.push(set(if on { 1. } else { 0. }));
                        }
                        return;
                    }
                    let (scale, suffix) = match u.ty {
                        UserType::Distance => (1.0, " m"),
                        UserType::Angle => (180.0 / std::f64::consts::PI, "°"),
                        UserType::Percentage => (100.0, " %"),
                        UserType::Time => (1.0, " s"),
                        _ => (1.0, ""),
                    };
                    let key = egui::Id::new(("rig-user-channel", item, &name));
                    let mut shown: f64 = ui.data_mut(|d| d.get_temp(key).unwrap_or(value * scale));
                    // The drag is read here, not by the field (speed 0):
                    // the pointer is locked in place on macOS so the screen
                    // edge never stops a scrub, and its own movement drives
                    // the value; it comes back where it was pressed.
                    let mut drag = crate::numeric_tab::DragValue::new(&mut shown)
                        .speed(0.0)
                        .suffix(suffix);
                    if matches!(u.ty, UserType::Integer | UserType::Choice) {
                        drag = drag.fixed_decimals(0);
                    }
                    let r = ui.add(drag);
                    if r.changed() && !r.dragged() {
                        ui.data_mut(|d| d.insert_temp(key, shown));
                    }
                    // UIC-51: a drag stays live. Each frame writes the value
                    // straight into the channel (the rig and the views
                    // follow); the value from before the drag is kept so
                    // the release makes the whole drag one undo step and
                    // one Auto Key. Typing still waits for Enter.
                    let before = egui::Id::new(("rig-user-channel-drag", item, &name));
                    let home = egui::Id::new(("rig-user-channel-home", item, &name));
                    if r.drag_started() {
                        if let Some(p) = r.interact_pointer_pos() {
                            ui.data_mut(|d| d.insert_temp(home, p));
                        }
                        if LOCK_POINTER {
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::CursorGrab(
                                    egui::CursorGrab::Locked,
                                ));
                        }
                    }
                    if r.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::None);
                        let dx = if LOCK_POINTER {
                            crate::viewport_buttons::raw_motion(ui).map_or(0., |d| d.x)
                        } else {
                            r.drag_delta().x
                        };
                        if dx != 0. {
                            let m = ui.input(|i| i.modifiers);
                            let fine = if m.shift {
                                0.1
                            } else if m.command || m.ctrl {
                                10.
                            } else {
                                1.
                            };
                            shown += f64::from(dx) * drag_rate(u.ty) * fine;
                            if let Some(lo) = u.min {
                                shown = shown.max(lo * scale);
                            }
                            if let Some(hi) = u.max {
                                shown = shown.min(hi * scale);
                            }
                            ui.data_mut(|d| d.insert_temp(key, shown));
                            if ui.data(|d| d.get_temp::<f64>(before)).is_none() {
                                ui.data_mut(|d| d.insert_temp(before, value));
                            }
                            if let Some(c) = self
                                .editor
                                .scene
                                .animation
                                .rig
                                .user_channel_mut(item, &name)
                            {
                                let _ = c.set(&name, shown / scale);
                            }
                        }
                    }
                    if r.drag_stopped() {
                        if LOCK_POINTER {
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::CursorGrab(
                                    egui::CursorGrab::None,
                                ));
                        }
                        if let Some(p) = ui.data_mut(|d| d.remove_temp::<egui::Pos2>(home)) {
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::CursorPosition(p));
                        }
                    }
                    if r.drag_stopped()
                        || r.lost_focus()
                        || (r.changed() && !r.dragged() && !r.has_focus())
                    {
                        ui.data_mut(|d| d.remove::<f64>(key));
                        let old = ui
                            .data_mut(|d| d.remove_temp::<f64>(before))
                            .unwrap_or(value);
                        if let Some(c) = self
                            .editor
                            .scene
                            .animation
                            .rig
                            .user_channel_mut(item, &name)
                        {
                            let _ = c.set(&name, old);
                        }
                        if (shown / scale - old).abs() > 1e-12 {
                            lines.push(set(shown / scale));
                        }
                    }
                });
            }
        }
        for l in lines {
            self.execute(&l);
        }
        true
    }
}
