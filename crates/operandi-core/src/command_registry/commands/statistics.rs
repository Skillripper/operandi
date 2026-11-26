//! Statistics panel commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- statistics
    // The Statistics panel (statistics_edit.rs, development/specs/
    // statistics-panel); Operandi-only names (UIC-165).
    script("operandi.statistics.count", Selection, "Counts a Statistics panel row: how many elements, how many selected.")
        .operandi()
        .on_editor(
            "Count a Statistics panel row (e.g. polygons.vertices:3) on the active meshes: \"n (m selected)\".",
            STATS_ROW_ARGS,
            crate::statistics_edit::run_command,
        )
        .args(&[arg("row", TEXT, N)])
        .quiet()
        .example("operandi.statistics.count polygons.vertices:3"),
    script("operandi.statistics.select", Selection, "Adds a Statistics row's elements to the selection, removes them, or selects only them; + switches to the row's mode.")
        .operandi()
        .on_editor(
            "Add (default), remove or select only a Statistics panel row's elements, one undo step.",
            STATS_SELECT_ARGS,
            crate::statistics_edit::run_command,
        )
        .args(&[arg("row", TEXT, N), opt("action", ArgType::Choice(&["add", "remove", "only"]), N, Some("add"))])
        .undo()
        .example("operandi.statistics.select polygons.vertices:3 action:add"),
    script("operandi.statistics.hide", Selection, "Hides or shows a Statistics row's elements.")
        .operandi()
        .on_editor(
            "Hide (default) or unhide a Statistics panel row's elements, one undo step.",
            STATS_HIDE_ARGS,
            crate::statistics_edit::run_command,
        )
        .args(&[arg("row", TEXT, N), opt("action", ArgType::Choice(&["hide", "unhide"]), N, Some("hide"))])
        .undo()
        .example("operandi.statistics.hide polygons.vertices:3"),
    script("operandi.statistics.rename", Selection, "Renames a part, material, selection set, vertex map, smoothing group or item group everywhere.")
        .operandi()
        .on_editor(
            "Rename a named Statistics panel row everywhere, one undo step.",
            STATS_RENAME_ARGS,
            crate::statistics_edit::run_command,
        )
        .args(&[arg("row", TEXT, N), arg("name", TEXT, N)])
        .undo()
        .example("operandi.statistics.rename polygons.part:Wheel name:Tyre"),
    script("operandi.statistics.delete", Selection, "Removes a part, material, selection set, vertex map, smoothing group or item group from every element; the elements stay.")
        .operandi()
        .on_editor(
            "Delete a named Statistics panel row's tag, set or map (elements stay), one undo step.",
            STATS_ROW_ARGS,
            crate::statistics_edit::run_command,
        )
        .args(&[arg("row", TEXT, N)])
        .undo()
        .example("operandi.statistics.delete polygons.set:Top"),
    script("operandi.statistics.paste", Selection, "Pastes a copied selection set (adds members) or vertex map (overwrites values) onto another.")
        .operandi()
        .on_editor(
            "Paste row from: onto row: (sets add members, maps overwrite values), one undo step.",
            STATS_PASTE_ARGS,
            crate::statistics_edit::run_command,
        )
        .args(&[arg("row", TEXT, N), arg("from", TEXT, N)])
        .undo()
        .example("operandi.statistics.paste polygons.set:Top from:polygons.set:Front"),
    script("operandi.statistics.clear", Selection, "Sets every value of a vertex map to zero; the map stays.")
        .operandi()
        .on_editor(
            "Clear a vertex map's values to zero (the map stays), one undo step.",
            STATS_ROW_ARGS,
            crate::statistics_edit::run_command,
        )
        .args(&[arg("row", TEXT, N)])
        .undo()
        .example("operandi.statistics.clear vertices.map:weight:Mask"),
];
