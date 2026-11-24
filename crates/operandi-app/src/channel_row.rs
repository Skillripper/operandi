//! One channel's value field, shared by the User Channels tab and the
//! channel panel (CP-40..43): a checkbox for an on/off channel, otherwise
//! a number field you click to type (UIC-51: waits for Enter) or drag to
//! haul live (CP-41). While dragging, the pointer locks in place on macOS
//! so the screen edge never stops a scrub, each frame writes the value
//! straight into the channel (the rig and views follow), and the release
//! puts the old value back and runs the channel's own command once: the
//! whole drag is one undo step and one Auto Key (CP-60, UIC-182).
use super::*;
use crate::unavailable::Gate;
use operandi_core::animation::{self, ChannelKey};
use operandi_core::channels::{ChannelType, Target};
use operandi_core::rig::UserType;

/// The pointer locks in place for a drag on macOS (as the view corner
/// buttons do, VBF-2); elsewhere the field follows the pointer.
const LOCK_POINTER: bool = cfg!(target_os = "macos") && !cfg!(test);

/// How a channel's value is shown and hauled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Kind {
    /// Shown = stored × scale.
    pub scale: f64,
    pub suffix: &'static str,
    /// Shown units per point of mouse travel (Shift ×0.1, Ctrl ×10).
    pub rate: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub whole: bool,
    pub boolean: bool,
}

/// Why a driven channel's field is greyed (CP-54).
const DRIVEN: &str = "Driven by a link, constraint or modifier";

const DEG: f64 = 180.0 / std::f64::consts::PI;

fn of(scale: f64, suffix: &'static str, rate: f64) -> Kind {
    Kind {
        scale,
        suffix,
        rate,
        min: None,
        max: None,
        whole: false,
        boolean: false,
    }
}

/// The kind of `key`'s value.
pub(crate) fn kind(scene: &operandi_core::Scene, key: &ChannelKey) -> Kind {
    if let Some(u) = scene.animation.rig.user_channel(key.item, &key.channel) {
        let mut k = match u.ty {
            UserType::Distance => of(1., " m", 0.01),
            UserType::Angle => of(DEG, "°", 0.5),
            UserType::Percentage => of(100., " %", 0.5),
            UserType::Time => of(1., " s", 0.01),
            UserType::Integer | UserType::Choice => Kind {
                whole: true,
                ..of(1., "", 0.1)
            },
            UserType::Boolean => Kind {
                boolean: true,
                ..of(1., "", 0.)
            },
            _ => of(1., "", 0.01),
        };
        k.min = u.min;
        k.max = u.max;
        return k;
    }
    if let Some((group, _)) = key.transform_part() {
        return match group {
            "pos" => of(1., " m", 0.01),
            "rot" => of(DEG, "°", 0.5),
            _ => of(100., " %", 0.5),
        };
    }
    let base = key.channel.split('.').next().unwrap_or("");
    match operandi_core::channels::find(scene, Target::Item(key.item), base).map(|c| c.ty) {
        Some(ChannelType::Bool) => Kind {
            boolean: true,
            ..of(1., "", 0.)
        },
        Some(ChannelType::Int | ChannelType::Enum(_)) => Kind {
            whole: true,
            ..of(1., "", 0.1)
        },
        Some(ChannelType::Distance) => of(1., " m", 0.01),
        Some(ChannelType::Angle) => of(DEG, "°", 0.5),
        Some(ChannelType::Percent) => Kind {
            min: Some(0.),
            max: Some(1.),
            ..of(100., " %", 0.5)
        },
        Some(ChannelType::Colour) => Kind {
            min: Some(0.),
            ..of(1., "", 0.005)
        },
        _ => of(1., "", 0.01),
    }
}

/// The command line that sets `key` to `value` (stored units), one undo
/// step that Auto Key keys like any edit of that channel.
pub(crate) fn set_line(scene: &operandi_core::Scene, key: &ChannelKey, value: f64) -> String {
    let (item, c) = (key.item, &key.channel);
    if scene.animation.rig.user_channel(item, c).is_some() {
        format!("operandi.rig.userChannelSet item:{item} name:{c} value:{value}")
    } else if scene.animation.rig.piece(item).is_some() {
        format!("operandi.rig.set piece:{item} channel:{c} value:{value}")
    } else if key.transform_part().is_some() {
        format!("transform.channel item:{item} name:{c} value:{value}")
    } else {
        format!("item.channel item:{item} name:{c} value:{value}")
    }
}

impl Operandi {
    /// `key`'s value field (CP-41..43); lines to run are pushed on
    /// `lines`. Driven channels show their value, not editable (CP-54).
    pub(crate) fn channel_field(&mut self, ui: &mut Ui, key: &ChannelKey, lines: &mut Vec<String>) {
        let scene = &self.editor.scene;
        let Some(value) = animation::read(scene, key) else {
            ui.weak("—");
            return;
        };
        let k = kind(scene, key);
        let driven = operandi_core::rig::driven_by(scene, key.item, &key.channel).is_some();
        if k.boolean {
            let mut on = value >= 0.5;
            let r = ui.gated(
                crate::unavailable::unless(!driven, DRIVEN),
                egui::Checkbox::without_text(&mut on),
            );
            if r.changed() {
                lines.push(set_line(scene, key, if on { 1. } else { 0. }));
            }
            return;
        }
        let id = egui::Id::new(("channel-field", key));
        let before = id.with("before");
        let home = id.with("home");
        let mut shown: f64 = ui.data_mut(|d| d.get_temp(id).unwrap_or(value * k.scale));
        // The drag is read here, not by the field (speed 0).
        let mut drag = crate::numeric_tab::DragValue::new(&mut shown)
            .speed(0.0)
            .suffix(k.suffix);
        drag = if k.whole {
            drag.fixed_decimals(0)
        } else {
            drag.max_decimals(3)
        };
        let r = ui.gated(crate::unavailable::unless(!driven, DRIVEN), drag);
        if r.changed() && !r.dragged() {
            ui.data_mut(|d| d.insert_temp(id, shown));
        }
        if r.drag_started() {
            if let Some(p) = r.interact_pointer_pos() {
                ui.data_mut(|d| d.insert_temp(home, p));
            }
            if LOCK_POINTER {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::CursorGrab(egui::CursorGrab::Locked));
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
                shown += f64::from(dx) * k.rate * fine;
                if let Some(lo) = k.min {
                    shown = shown.max(lo * k.scale);
                }
                if let Some(hi) = k.max {
                    shown = shown.min(hi * k.scale);
                }
                ui.data_mut(|d| d.insert_temp(id, shown));
                if ui.data(|d| d.get_temp::<f64>(before)).is_none() {
                    ui.data_mut(|d| d.insert_temp(before, value));
                }
                // Live: straight into the channel, no undo step yet.
                let _ = animation::write(&mut self.editor.scene, key, shown / k.scale);
            }
        }
        if r.drag_stopped() {
            if LOCK_POINTER {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::CursorGrab(egui::CursorGrab::None));
            }
            if let Some(p) = ui.data_mut(|d| d.remove_temp::<egui::Pos2>(home)) {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::CursorPosition(p));
            }
        }
        if r.drag_stopped() || r.lost_focus() || (r.changed() && !r.dragged() && !r.has_focus()) {
            ui.data_mut(|d| d.remove::<f64>(id));
            let old = ui
                .data_mut(|d| d.remove_temp::<f64>(before))
                .unwrap_or(value);
            let _ = animation::write(&mut self.editor.scene, key, old);
            let new = if k.whole {
                (shown / k.scale).round()
            } else {
                shown / k.scale
            };
            if (new - old).abs() > 1e-12 {
                lines.push(set_line(&self.editor.scene, key, new));
            }
        }
    }
}
