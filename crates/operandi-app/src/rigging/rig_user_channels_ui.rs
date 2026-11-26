//! The active item's rig user channels (RG-53) as sliders on the User
//! Channels tab: an imported rig's controls (an arm angle, wings, landing
//! gear) and the ones Add User Channel makes in the Channel Links window.
//! The field is the channel panel's (channel_row.rs): a drag is live with
//! the pointer locked (UIC-51), the release or a typed value on Enter is
//! one undo step that Auto Key keys like any channel edit.
use super::*;
use operandi_core::animation::ChannelKey;
use operandi_core::rig::UserType;

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
            if u.ty == UserType::String {
                ui.horizontal(|ui| {
                    crate::option_row::label(ui, &label).on_hover_text(&u.name);
                    ui.label(&u.text);
                });
                continue;
            }
            for name in u.channel_names() {
                let part = name.strip_prefix(&u.name).unwrap_or("");
                let key = ChannelKey::new(item, &name);
                // UIC-112/119: the status light keys it; the label picks
                // it for keying and for a channel set (CP-10).
                self.anim_row_keys(std::slice::from_ref(&key), false);
                crate::form::rows::row(ui, &format!("{label}{part}"), |ui| {
                    self.channel_field(ui, &key, &mut lines)
                });
            }
        }
        for l in lines {
            self.execute(&l);
        }
        true
    }
}
