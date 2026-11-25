//! Ctrl+Tab / Ctrl+Shift+Tab page through the tool tabs (spec
//! tool-tab-cycling v1.1, Jason 2026-11-25).
//!
//! - Each press opens the next (or previous) tab of the strip at once, in
//!   on-screen order, wrapping at the ends. No pop-up, no confirm step.
//! - Only the strip's tabs count: a tab the set hides (Rig without the
//!   animation switch, the painting tabs in a build without painting) is
//!   skipped because it isn't in the strip. One tab, or no strip: nothing.
//! - Opening a tab this way is a click on it (UIC-20): fields are confirmed
//!   first, the active tool stays. The user asked for the switch, so this
//!   keeps UIC-21.
//! - The keys come from the Input Map ("Next Tool Tab", "Previous Tool
//!   Tab"), so tooltips show them and they can be rebound (UIC-04). "Ctrl"
//!   is the real Control key on a Mac; Cmd+Tab is the system's.
//! - They are taken out of the raw input before egui sees them, so egui's
//!   own Ctrl+Tab focus travel never runs in the main window, also while a
//!   text or number field has focus. Plain Tab is untouched (subdivision in
//!   the main window, next field in fields, UIC-51). A modal dialog, a
//!   menu, a pop-up or a pie keeps every key, Ctrl+Tab included.
//!
//! Provisional (questions for Jason): Q1 Ctrl+Tab in a multi-line text
//! field pages tabs; Q2 a hidden or collapsed tool panel is not reopened.
//! The keys used to be egui's focus stepping (Ctrl+Tab) and the Layout
//! Switcher (Ctrl+Shift+Tab, UIC-18, amended with Jason's OK); it keeps Layout > Recent
//! Layouts… and the bottom bar's button, and gets no key by default.
use super::*;

/// The Input Map actions.
pub(crate) const NEXT: &str = "Next Tool Tab";
pub(crate) const PREVIOUS: &str = "Previous Tool Tab";

/// Where the pending steps wait between the raw-input hook and the frame.
fn steps_id() -> Id {
    Id::new("tool-tab-cycle-steps")
}

/// Asks the horizontal strip to scroll the open tab into sight.
pub(crate) fn reveal_id() -> Id {
    Id::new("tool-tab-cycle-reveal")
}

impl Operandi {
    /// Raw-input hook: take the Next/Previous Tool Tab presses (and their
    /// releases) out of the main window's input and keep their steps for
    /// [`Self::tool_tab_keys`]. Key repeats count, one tab each.
    pub(super) fn take_tool_tab_keys(&mut self, ctx: &Context, input: &mut RawInput) {
        // Dialogs, menus, pop-ups, pies and the layout switcher keep their
        // keys (spec 2: Ctrl+Tab there never changes the tab behind).
        if self.has_modal()
            || egui::Popup::is_any_open(ctx)
            || native_menu::tracking()
            || self.pie.is_some()
            || self.layouts.switcher.is_some()
            || self.corner.properties.owns_input()
        {
            return;
        }
        let next = self.bindings.get(NEXT).cloned().unwrap_or_default();
        let previous = self.bindings.get(PREVIOUS).cloned().unwrap_or_default();
        if next.is_empty() && previous.is_empty() {
            return;
        }
        // A rebound plain key (no Ctrl/Alt/Cmd) still types in a field.
        let typing = self.text_editor_focused(ctx);
        let mut steps = 0i32;
        input.events.retain(|event| {
            let Event::Key {
                key,
                modifiers,
                pressed,
                ..
            } = event
            else {
                return true;
            };
            if typing && !(modifiers.ctrl || modifiers.alt || modifiers.mac_cmd) {
                return true;
            }
            let step = if input::matches_chord(&next, *key, *modifiers) {
                1
            } else if input::matches_chord(&previous, *key, *modifiers) {
                -1
            } else {
                return true;
            };
            if *pressed {
                steps += step;
            }
            false
        });
        if steps != 0 {
            ctx.data_mut(|d| {
                let pending = d.get_temp::<i32>(steps_id()).unwrap_or(0);
                d.insert_temp(steps_id(), pending + steps);
            });
        }
    }

    /// Run the steps the raw-input hook kept. Called from `keyboard`
    /// after the frame's panels are drawn, so the strip is known.
    pub(super) fn tool_tab_keys(&mut self, ctx: &Context) {
        let steps = ctx.data_mut(|d| d.remove_temp::<i32>(steps_id()).unwrap_or(0));
        if steps == 0 {
            return;
        }
        // Spec 6: ignored while a mouse drag is in progress.
        if self.pointer_held || ctx.input(|i| i.pointer.any_down()) {
            return;
        }
        for _ in 0..steps.unsigned_abs() {
            self.step_tool_tab(Some(ctx), steps.signum());
        }
        ctx.request_repaint();
    }

    /// Open the tab `step` (+1 or -1) places along the strip, wrapping.
    /// False (and nothing changes) without a strip of two or more tabs.
    /// Also the "Next Tool Tab" / "Previous Tool Tab" commands.
    pub(crate) fn step_tool_tab(&mut self, ctx: Option<&Context>, step: i32) -> bool {
        // Spec 7 / Q2: a hidden or collapsed tool panel isn't reopened. The
        // strip registers a hit rect per tab it drew this frame.
        let tabs = self.layouts.tool_set.tabs();
        if tabs.len() <= 1 || !self.tool_tab_strip_shown() {
            return false;
        }
        let Some(shown) = self.shown_category() else {
            return false;
        };
        let at = tabs.iter().position(|&t| t == shown).unwrap_or(0) as i32;
        let n = tabs.len() as i32;
        let target = tabs[(at + step).rem_euclid(n) as usize];
        // The same as a click on the tab (category_response).
        if !self.confirm_fields() {
            return false;
        }
        if let Some(ctx) = ctx {
            self.leave_old_tab_focus(ctx);
            ctx.data_mut(|d| d.insert_temp(reveal_id(), true));
            // Spec 5: screen readers hear the opened tab's name.
            let label = self.category_tab_label(target);
            ctx.output_mut(|o| {
                o.events.push(egui::output::OutputEvent::ValueChanged(
                    WidgetInfo::selected(
                        WidgetType::SelectableLabel,
                        true,
                        true,
                        format!("{label} tab"),
                    ),
                ))
            });
        }
        self.category = target;
        self.status = format!("{} tab", self.category_tab_label(target));
        true
    }

    /// Whether the tool tab strip was drawn this frame.
    fn tool_tab_strip_shown(&self) -> bool {
        self.toolbox_rects
            .keys()
            .any(|k| k.starts_with("Category ") && !k.ends_with(" surface") && !k.ends_with(" ink"))
    }

    /// Spec 4: focus on a control in the old tab's drawer goes back to the
    /// viewport (no widget focused), so it never sits on a hidden control.
    /// Focus anywhere else stays.
    fn leave_old_tab_focus(&self, ctx: &Context) {
        let Some(drawer) = self.toolbox_rects.get(DRAWER).copied() else {
            return;
        };
        let Some(id) = ctx.memory(|m| m.focused()) else {
            return;
        };
        let inside = ctx
            .read_response(id)
            .is_some_and(|r| drawer.intersects(r.rect));
        if inside {
            ctx.memory_mut(|m| m.surrender_focus(id));
        }
    }

    /// The tab tooltip's key hint (UIC-04): " · Next tab Ctrl+Tab ·
    /// Previous tab Ctrl+Shift+Tab", from the Input Map.
    pub(super) fn tool_tab_keys_hint(&self) -> String {
        [(NEXT, "Next tab"), (PREVIOUS, "Previous tab")]
            .iter()
            .filter_map(|(action, name)| {
                let binding = self.bindings.get(*action).filter(|b| !b.is_empty())?;
                Some(format!(" · {name} {}", input::display_chord(binding)))
            })
            .collect()
    }
}

/// The toolbox drawer's rect in `toolbox_rects` (the tab's form; Tool
/// Properties below it is not part of a tab).
pub(crate) const DRAWER: &str = "Toolbox drawer";
