//! The Sculpt tab's Multiresolution section (multires.rs in core): its
//! buttons' lit/greyed answers (UIC-223) and the level read-out.
use super::*;
use crate::command_handlers::Commands;
use operandi_core::command_runtime::CommandState;
use operandi_core::multires::{self, LevelChange};

/// The level command a form line runs (`operandi.multires.level step:up`).
fn change(name: &str, rest: &str) -> Option<LevelChange> {
    let words = operandi_core::command_line::tokens(rest).ok()?;
    let args = operandi_core::command_line::json_args(name, &words).ok()?;
    multires::level_change(name, &args).ok()
}

/// Greyed with the reason a level command can't run on the active mesh.
pub(super) fn multires_answers(r: &mut Commands) {
    for name in [
        "operandi.multires.subdivide",
        "operandi.multires.level",
        "operandi.multires.deleteHigher",
        "operandi.multires.apply",
    ] {
        r.answer(name, |a, c| {
            let unavailable = match change(c.entry.name, c.rest) {
                Some(change) => a.editor.multires_unavailable(change, None),
                None => Some("Give level:<n> or step:up / step:down".into()),
            };
            CommandState {
                unavailable,
                on: false,
                title: None,
            }
        })
        .unwrap_or_else(|e| panic!("{e}"));
    }
}

impl Operandi {
    /// The Sculpt tab's level read-out (escape hatch `sculpt.multiresLevel`):
    /// "Level 2 of 3 · 98,304 polygons", or that the mesh has no levels.
    pub(super) fn multires_readout(&mut self, ui: &mut Ui) {
        let text = match self.editor.scene.active().filter(|i| i.is_mesh()) {
            None => "No active mesh".to_owned(),
            Some(item) => match multires::readout(item) {
                Some(text) if multires::in_step(item) => text,
                Some(text) => format!("{text} · polygons changed"),
                None => "Not multiresolution".to_owned(),
            },
        };
        ui.label(RichText::new(text).color(theme::weak_text()));
    }
}
