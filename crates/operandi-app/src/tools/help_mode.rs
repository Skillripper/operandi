//! Help mode (UIC-06; Jason 2026-11-24): F1 or Help > What's This? turns
//! it on. The pointer becomes a question mark and the next click opens the
//! User Guide page for whatever it lands on instead of using it. Esc or a
//! right-click leaves without opening anything.
//!
//! What a click names (its help id, UIC-301 / guide RULES.md section 5):
//! - a toolbox or form button: the command-registry name of the command it
//!   runs (`form_rects`, `toolbox_rects` keys that are commands);
//! - a menu entry: its command (the menu runs through `menu_action`, which
//!   asks `help_mode_takes_menu` first);
//! - a Tool Properties row: the active tool's registry name; its header,
//!   or the panel with no tool on: "Tool Properties";
//! - anything else in a panel: the panel's id (`panel_help_id`), from the
//!   panel drawn there this frame (`help_region`);
//! - a 3D or UV view: "3D View"; anything else: "Interface".
//!
//! The click is taken out of the window's input before the frame runs
//! (`help_mode_take_input`, from `raw_input_hook`), so no control sees it,
//! and resolved after the panels have drawn (`help_mode_after_compose`).
//! The page opens at /guide/<version>/go/<help-id>/ (test builds:
//! /guide/test/go/<help-id>/) with Help > User Guide's helper.
//!
//! Provisional: only the main window takes part. A floating palette in its
//! own OS window, and the in-window menu bar (Linux, Windows), still act on
//! a click while help mode is on.
use super::*;
use crate::update_check::Channel;

/// The registry command (UIC-220).
pub(crate) const COMMAND: &str = "operandi.help.whatsThis";
/// Its key-map action and alias: F1 by default.
pub(crate) const ACTION: &str = "What's This?";
/// The page for anything help mode can't name more closely.
pub(crate) const INTERFACE: &str = "Interface";
/// A 3D (or UV) view.
pub(crate) const VIEW: &str = "3D View";
/// The toolbox and the tool tabs above it.
pub(crate) const TOOLBOX: &str = "Toolbox";
/// The Tool Properties panel itself (its header, or no tool on).
pub(crate) const TOOL_PROPERTIES: &str = "Tool Properties";

/// The message line while help mode is on.
pub(crate) const HINT: &str =
    "What's This?: click anything to open its page in the User Guide · Esc or right-click: cancel";

#[derive(Default)]
pub(crate) struct HelpMode {
    pub(crate) on: bool,
    /// Where the click that picks landed; resolved after the panels draw.
    pick: Option<Pos2>,
    /// Esc or a right-click: leave without opening anything.
    cancel: bool,
    /// Buttons whose press help mode took: their release is taken too,
    /// even after help mode has ended, so nothing sees half a click.
    held: [bool; 2],
    /// Esc's press was taken: take its release as well.
    escape_held: bool,
    /// Panels drawn this frame and their help ids, while help mode is on.
    regions: Vec<(Rect, String)>,
}

impl HelpMode {
    /// Takes help mode's clicks and Esc out of one frame's raw events.
    pub(crate) fn take(&mut self, events: &mut Vec<Event>) {
        events.retain(|event| match *event {
            Event::PointerButton {
                pos,
                button,
                pressed,
                ..
            } => {
                let index = match button {
                    PointerButton::Primary => 0,
                    PointerButton::Secondary => 1,
                    _ => return true,
                };
                if pressed && self.on {
                    if index == 0 {
                        self.pick.get_or_insert(pos);
                    } else {
                        self.cancel = true;
                    }
                    self.held[index] = true;
                    false
                } else if !pressed && self.held[index] {
                    self.held[index] = false;
                    false
                } else {
                    true
                }
            }
            Event::Key {
                key: Key::Escape,
                pressed,
                ..
            } => {
                if pressed && self.on {
                    self.cancel = true;
                    self.escape_held = true;
                    false
                } else if !pressed && self.escape_held {
                    self.escape_held = false;
                    false
                } else {
                    true
                }
            }
            _ => true,
        });
    }
}

/// The help id of a toolbox / form control by its key: the registry name
/// of the command it runs, or `None` when the key isn't a command (a
/// section heading, a tab), so the panel answers instead.
pub(crate) fn control_id(key: &str) -> Option<String> {
    let key = key.strip_prefix("flyout:").unwrap_or(key);
    if key.starts_with("Category ") {
        return None;
    }
    operandi_core::command_registry::find_line(key).map(|(entry, _)| entry.name.to_owned())
}

/// The help id of a registry panel (the ids the guide's panel pages claim).
pub(crate) fn panel_help_id(kind: panel_registry::PanelKind) -> String {
    match kind {
        panel_registry::PanelKind::Items => "Items List".into(),
        panel_registry::PanelKind::Missing(_) => INTERFACE.into(),
        kind => panel_registry::panel_type(kind).name.into(),
    }
}

/// The help id for Tool Properties' rows while the tool titled `title`
/// shows there (`tool_properties_title`).
pub(crate) fn tool_help_id(title: &str) -> String {
    if title == TOOL_PROPERTIES {
        return TOOL_PROPERTIES.into();
    }
    operandi_core::command_registry::find(title)
        .map(|entry| entry.name.to_owned())
        .unwrap_or_else(|| title.to_owned())
}

/// What a click at `at` names: the smallest control under it, else the
/// smallest region (panel) under it, else the Interface page.
pub(crate) fn pick(
    at: Pos2,
    controls: impl IntoIterator<Item = (Rect, String)>,
    regions: &[(Rect, String)],
) -> String {
    let smallest = |items: &mut dyn Iterator<Item = (Rect, String)>| {
        items
            .filter(|(rect, _)| rect.contains(at))
            .min_by(|(a, _), (b, _)| a.area().total_cmp(&b.area()))
            .map(|(_, id)| id)
    };
    smallest(&mut controls.into_iter())
        .or_else(|| smallest(&mut regions.iter().cloned()))
        .unwrap_or_else(|| INTERFACE.into())
}

/// `id` as one URL path segment: everything but unreserved characters
/// percent-encoded ("Items List" is `Items%20List`).
fn encode(id: &str) -> String {
    let mut out = String::new();
    for b in id.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The page for help id `id` in a build of `version` on `channel`.
pub(crate) fn help_url_for(version: &str, channel: Channel, id: &str) -> String {
    format!(
        "{}go/{}/",
        help_menu::guide_url_for(version, channel),
        encode(id)
    )
}

/// The page for `id` in this build's guide.
pub(crate) fn help_url(id: &str) -> String {
    help_url_for(env!("CARGO_PKG_VERSION"), Channel::build_default(), id)
}

impl Operandi {
    /// `operandi.help.whatsThis`: help mode on, or off again.
    pub(super) fn toggle_help_mode(&mut self) {
        if self.help_mode.on {
            self.leave_help_mode("Left What's This?");
        } else {
            self.help_mode.on = true;
            self.help_mode.pick = None;
            self.help_mode.cancel = false;
            self.status = HINT.into();
        }
    }

    fn leave_help_mode(&mut self, status: &str) {
        let held = self.help_mode.held;
        let escape_held = self.help_mode.escape_held;
        self.help_mode = HelpMode {
            held,
            escape_held,
            ..Default::default()
        };
        self.status = status.into();
    }

    /// Opens help id `id`'s page and leaves help mode.
    fn open_help(&mut self, id: &str) {
        let url = help_url(id);
        let status = match self.open_in_browser(&url) {
            Ok(()) => format!("Opened the User Guide page for {id}"),
            Err(e) => {
                self.copy_next = Some(url);
                format!("Couldn't open the browser ({e}); the link is on the clipboard")
            }
        };
        self.leave_help_mode(&status);
    }

    /// The raw-input step: help mode's clicks and Esc never reach the UI.
    pub(super) fn help_mode_take_input(&mut self, input: &mut RawInput) {
        if input.viewport_id == ViewportId::ROOT {
            self.help_mode.take(&mut input.events);
        }
    }

    /// Notes that the panel drawn in `rect` this frame has help id `id`.
    pub(super) fn help_region(&mut self, rect: Rect, id: impl Into<String>) {
        if self.help_mode.on {
            self.help_mode.regions.push((rect, id.into()));
        }
    }

    /// After the panels drew: cancel, or open the picked control's page,
    /// or keep the question-mark pointer.
    pub(super) fn help_mode_after_compose(&mut self, ctx: &Context) {
        let regions = std::mem::take(&mut self.help_mode.regions);
        if !self.help_mode.on {
            return;
        }
        if std::mem::take(&mut self.help_mode.cancel) {
            self.leave_help_mode("Left What's This?");
            return;
        }
        let Some(at) = self.help_mode.pick.take() else {
            ctx.set_cursor_icon(egui::CursorIcon::Help);
            return;
        };
        let id = self.help_id_at(at, regions);
        self.open_help(&id);
    }

    /// What a click at `at` names, from what drew this frame.
    fn help_id_at(&self, at: Pos2, mut regions: Vec<(Rect, String)>) -> String {
        let controls = self
            .form_rects
            .iter()
            .chain(&self.toolbox_rects)
            .filter_map(|(key, rect)| control_id(key).map(|id| (*rect, id)));
        regions.extend(self.view_rects.values().map(|r| (*r, VIEW.to_owned())));
        if !self.viewport_rect.any_nan() && self.viewport_rect.is_positive() {
            regions.push((self.viewport_rect, VIEW.into()));
        }
        pick(at, controls, &regions)
    }

    /// A menu entry chosen while help mode is on opens its command's page
    /// instead of running it (true: the menu should do nothing else).
    pub(super) fn help_mode_takes_menu(&mut self, action: &crate::app_menu::Action) -> bool {
        if !self.help_mode.on {
            return false;
        }
        let crate::app_menu::Action::Line(index) = *action else {
            return false;
        };
        let Some(line) = crate::app_menu::action_line(index) else {
            return false;
        };
        let id = operandi_core::command_registry::find_line(&line)
            .map(|(entry, _)| entry.name.to_owned())
            .unwrap_or_else(|| INTERFACE.into());
        // Choosing What's This? again just leaves help mode.
        if id == COMMAND {
            return false;
        }
        self.open_help(&id);
        true
    }
}
