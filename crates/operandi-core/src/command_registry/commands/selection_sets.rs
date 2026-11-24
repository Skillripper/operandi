//! Selection set commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- selection sets
    script(
        "select.editSet",
        Selection,
        "Saves the selection as a named set; bare opens the dialog.",
    )
    .headless(
        "Store the current selection as a named selection set.",
        &[jreq("name", "string", "Set name.")],
    )
    .args(&[
        arg("name", TEXT, N),
        opt("mode", ArgType::Choice(&["add"]), N, None),
    ])
    .undo()
    .example("select.editSet Front"),
    script(
        "select.useSet",
        Selection,
        "Replaces, adds or removes a set's members in the selection; bare opens the dialog.",
    )
    .headless(
        "Select a named selection set.",
        &[
            jreq("name", "string", "Set name."),
            jopt(
                "mode",
                "string",
                "\"replace\" (default), \"add\" or \"remove\".",
            ),
        ],
    )
    .args(&[
        arg("name", TEXT, N),
        opt(
            "mode",
            ArgType::Choice(&["replace", "add", "remove"]),
            N,
            Some("replace"),
        ),
    ])
    .undo()
    .example("select.useSet Front"),
    script(
        "select.deleteSet",
        Selection,
        "Deletes a selection set (all:true deletes every set); bare opens the dialog.",
    )
    .headless(
        "Delete a selection set.",
        &[
            jopt(
                "name",
                "string",
                "Set name; omit to delete every set of the current mode.",
            ),
            jopt(
                "all",
                "boolean",
                "true deletes every set of the current mode.",
            ),
        ],
    )
    .args(&[
        opt("name", TEXT, N, None),
        opt("all", BOOL, N, Some("false")),
    ])
    .undo()
    .example("select.deleteSet Front"),
    script(
        "selset.ungroup",
        Selection,
        "Deletes a selection set group; its sets stay.",
    )
    .operandi()
    .headless(
        "Delete a selection-set group (the sets stay).",
        &[jreq("name", "string", "Group name.")],
    )
    .args(&[arg("group", TEXT, N)])
    .undo()
    .example("selset.ungroup Group"),
    script(
        "select.convertGroup",
        Selection,
        "Makes a selection set group of named sets; bare opens the dialog.",
    )
    .headless(
        "Group selection sets.",
        &[
            jreq("name", "string", "Group name."),
            jopt(
                "sets",
                "array of strings",
                "Set names; default every set of the current mode.",
            ),
        ],
    )
    .args(&[arg("group", TEXT, N), opt("sets", TEXT, N, None)])
    .undo()
    .example("select.convertGroup Group"),
];
