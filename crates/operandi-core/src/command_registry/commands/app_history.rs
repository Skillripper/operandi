//! Undo, redo and the app's history phrases (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- history
    script("app.undo", Modeling, "Undoes the last step.")
        .spelled("Undo")
        .quiet(),
    script("app.redo", Modeling, "Redoes the last undone step.")
        .spelled("Redo")
        .quiet(),
    action("Apply", Tools, "Applies the active tool.")
        .modo("tool.apply")
        .aliases(&[
            "Bridge Apply",
            "Cone Apply",
            "Ellipsoid Apply",
            "Spike Apply",
            "Torus Apply",
            "Capsule Apply",
            "Arc Apply",
            "Tube Apply",
            "Smooth Shift Apply",
            "Thicken Apply",
            "Apply Last Cube",
        ])
        .undo(),
    action("Drop", Tools, "Drops the active tool.")
        .modo("tool.drop")
        .aliases(&["Drop Cube", "Smooth Shift Drop", "Thicken Drop"])
        .undo(),
    action("Cancel", Tools, "Cancels the active tool's edit.")
        .operandi()
        .aliases(&["Smooth Shift Cancel", "Thicken Cancel"])
        .undo(),
    action("Continue", Tools, "Starts the last tool again.")
        .operandi()
        .aliases(&["Repeat"])
        .tool(),
    action("Tool Properties", Tools, "Opens Tool Properties.")
        .operandi()
        .ui(),
];
