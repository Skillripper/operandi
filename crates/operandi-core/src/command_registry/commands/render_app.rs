//! Render and application phrases (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- render
    script("render", Render, "Renders the scene (or brings the Render window forward).").ui().side_effect(),
    action("Open Render Window", Render, "Opens the Render window.")
        .operandi().ui(),
    action("Preview", Render, "Opens Preview.")
        .operandi().ui(),
    // Render outputs group D (RO-Q27): one Light Path Expressions output per light.
    script(crate::per_light_outputs::NAME, Render, "Add Per-Light Render Outputs: one Light Path Expressions output per renderable light (C.*L'<label>'); a light with no LPE Label gets its name as its label. One undo step.")
        .operandi()
        .undo()
        .on_editor(
            "Add one Light Path Expressions Render Output per renderable light, skipping lights that already have one.",
            &[],
            crate::per_light_outputs::run,
        ),
    // ---------------------------------------------------------- application
    action("Preferences", Preferences, "Opens Preferences.")
        .operandi().ui(),
    action("Reset Preferences", Preferences, "Asks to reset every preference.")
        .operandi().ui(),
];
