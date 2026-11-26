//! Help, Command History and tool commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- help
    script(
        "cmds.showHelp",
        Help,
        "Lists help topics, or shows one command's arguments and help.",
    )
    .spelled("help")
    .args(&[opt("command", TEXT, N, None)])
    .query()
    .example("cmds.showHelp poly.flip"),
    script(
        "command.list",
        Help,
        "Lists every command (optionally those whose name or area contains a word) in the log.",
    )
        .operandi()
    .args(&[opt("filter", TEXT, N, None)])
    .query(),
    // ---------------------------------------------------------- history
    // Command History (HIS-11, HIS-22, HIS-50, CMD-40). Argument names
    // are provisional (CH-Q03).
    script("cmds.batch", Help, "Runs up to 9 command lines (c1:..c9:) as one undo step named label:, its lines kept as sub-commands.")
        .args(&[
            opt("label", TEXT, N, Some("Batch")),
            opt("c1", TEXT, N, None),
            opt("c2", TEXT, N, None),
            opt("c3", TEXT, N, None),
            opt("c4", TEXT, N, None),
            opt("c5", TEXT, N, None),
            opt("c6", TEXT, N, None),
            opt("c7", TEXT, N, None),
            opt("c8", TEXT, N, None),
            opt("c9", TEXT, N, None),
        ])
        .undo()
        .example("cmds.batch label:Two c1:poly.flip c2:poly.flip"),
    script("cmds.undoTo", Help, "Undoes or redoes until index: steps are done (0 undoes everything the undo list holds); the Undos tab's drag row runs it.")
        .args(&[arg("index", INT, N)])
        .quiet()
        .example("cmds.undoTo 0"),
    script("history.clear", Help, "Empties the Command History's History tab (results:true empties the Results tab instead).")
        .args(&[opt("results", BOOL, N, Some("false"))])
        .side_effect(),
    script("history.max", Help, "Sets or asks (?) how many lines the History tab keeps; the oldest go first.")
        .args(&[arg("count", TEXT, N)])
        .ui()
        .query()
        .example("history.max 1000"),
    script("history.resultsMax", Help, "Sets or asks (?) how many lines the Results tab keeps.")
        .args(&[arg("count", TEXT, N)])
        .ui()
        .query()
        .example("history.resultsMax 200"),
    // ---------------------------------------------------------- tools
    script("tool.set", Tools, "Starts (on), stops (off) or queries (?) a tool by id.")
        .args(&[
            arg("preset", ArgType::Choice(TOOL_IDS), N),
            arg("mode", ON_OFF_QUERY, N),
        ])
        .tool()
        .query()
        .example("tool.set prim.cube ?"),
    script("tool.attr", Tools, "Sets or queries (?) one setting of the live tool (Spike, Smooth Shift, Bridge, Edge Bevel/Chamfer, Polygon Reduction).")
        .args(&[arg("tool", TEXT, N), arg("attr", TEXT, N), arg("value", TEXT, N)])
        .query()
        .example("tool.attr poly.spikey factor ?").undo(),
    script("tool.reset", Tools, "Resets the active tool's settings.")
        .args(&[opt("tool", TEXT, N, None)]).undo(),
    // MN-05: Edit > Reset Tool Attributes and its key (SC-083).
    action("Reset Tool Attributes", Tools, "Puts the active tool's values back to the factory ones; the tool stays active.")
        .modo("tool.reset").undo(),
    script("tool.apply", Tools, "Applies the active tool and keeps it active.")
        .aliases(&["tool.doApply"])
        .undo(),
    script("tool.drop", Tools, "Drops the active tool, keeping its edit.").undo(),
    // MP-01: opens a form as a pop-over; Space runs it for Mini Properties.
    script("attr.formPopover", View, "Opens a form as a pop-over at the pointer (form:MiniProps is Mini Properties); again closes it.")
        .args(&[
            arg("form", TEXT, N),
            opt("proficiency", TEXT, N, None),
            opt("pin", BOOL, N, Some("true")),
            opt("lockProficiency", BOOL, N, Some("false")),
            opt("focusFirstField", BOOL, N, Some("false")),
            opt("open", BOOL, N, None),
        ])
        .example("attr.formPopover form:MiniProps proficiency:mini lockProficiency:true pin:true")
        .ui(),
];
