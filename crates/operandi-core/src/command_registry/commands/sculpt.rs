//! Sculpt commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- sculpt
    script("sculpt.smooth", Sculpt, "Starts the Smooth sculpt brush.").tool(),
    script("sculpt.push", Sculpt, "Starts the Push sculpt brush.").tool(),
    script("sculpt.inflate", Sculpt, "Starts the Inflate sculpt brush.").tool(),
    script("sculpt.move", Sculpt, "Starts the Move sculpt brush.").tool(),
    script("sculpt.fold", Sculpt, "Starts the Fold sculpt brush.").tool(),
    script("sculpt.clay", Sculpt, "Starts the Clay sculpt brush.").tool(),
    script("sculpt.smudge", Sculpt, "Starts the Smudge sculpt brush.").tool(),
    script("sculpt.carve", Sculpt, "Starts the Carve sculpt brush.").tool(),
    script("sculpt.scar", Sculpt, "Starts the Scar sculpt brush.").tool(),
    script("sculpt.flatten", Sculpt, "Starts the Flatten sculpt brush.").tool(),
    script("sculpt.tangentPinch", Sculpt, "Starts the Tangent Pinch sculpt brush.").tool(),
    script("sculpt.centerPinch", Sculpt, "Starts the Center Pinch sculpt brush.").tool(),
    script("sculpt.emboss", Sculpt, "Starts the Emboss sculpt brush.").tool(),
    script("sculpt.shrink", Sculpt, "Starts the Attenuate sculpt brush.").tool(),
    script("sculpt.spin", Sculpt, "Starts the Spin sculpt brush.").tool(),
    script("sculpt.vmapPaint", Sculpt, "Starts the Paint Vertex Map Tool.").tool(),
    script("sculpt.vmapSmooth", Sculpt, "Starts the Smooth Vertex Map Tool.").tool(),
    script("sculpt.vmapScale", Sculpt, "Starts the Scale Down Vertex Map Tool.").tool(),
    script("sculpt.vmapErase", Sculpt, "Starts the Erase Vertex Map Tool.").tool(),
    script("sculpt.vmapSetVal", Sculpt, "Starts the Set Value Vertex Map Tool.").tool(),
    script("vertMap.new", Sculpt, "Adds a vertex map (type weight, or uv for a UV map) to the active meshes and selects it.")
        .headless(
            "Add a weight map to the active meshes and select it (the one the Vertex Map Tools and the Vertex Map falloff use), or with type uv a UV map to the active mesh that becomes its selected UV map; one undo step.",
            &[
            jreq("name", "string", "The new map's name."),
            jopt(
                "type",
                "string",
                "\"weight\" (the default) or \"uv\" (also \"txuv\").",
            ),
            jopt(
                "init",
                "boolean",
                "Fill every point with value (0 when value is left out); false leaves the map empty. Default: filled only when value is given.",
            ),
            jopt("color", "string", "Accepted and ignored."),
            jopt("value", "number", "The weight every point gets, e.g. 0.5."),
        ],
        )
        .args(&[
            arg("name", TEXT, N),
            opt("type", ArgType::Choice(&["weight", "uv", "txuv"]), N, Some("weight")),
            opt("init", BOOL, N, Some("false")),
            opt("color", TEXT, N, None),
            opt("value", FLOAT, N, None),
        ])
        .undo()
        .example("vertMap.new Mask weight true value:0.5"),
    script("select.vertexMap", Sculpt, "Selects the weight map the Vertex Map Tools and falloff use.")
        .headless(
            "Select the weight map the Vertex Map Tools and the Vertex Map falloff use (one at a time).",
            &[
            jopt("name", "string", "The map's name; left out selects none."),
            jopt("type", "string", "\"weight\" (default) or \"wght\"."),
            jopt(
                "mode",
                "string",
                "\"set\" (default) or \"add\" select it; \"remove\" selects none.",
            ),
        ],
        )
        .args(&[
            opt("name", TEXT, N, None),
            opt("type", ArgType::Choice(&["weight", "wght"]), N, Some("weight")),
            opt("mode", ArgType::Choice(&["set", "add", "remove"]), N, Some("set")),
        ])
        .example("select.vertexMap Mask weight set").ui(),
    script("sculpt.liveWeightEdit", Sculpt, "Updates what a weight map drives during a stroke.")
        .args(&[opt("live", BOOL, N, None)])
        .example("sculpt.liveWeightEdit live:false").ui(),
    script("brush.smooth", Sculpt, "Gives the sculpt brush a smooth tip.").ui(),
    script("brush.hard", Sculpt, "Gives the sculpt brush a hard tip.").ui(),
    script("sculpt.borderLock", Sculpt, "Locks mesh borders while sculpting (toggles when bare).")
        .args(&[opt("state", ON_OFF_QUERY, N, None)])
        .query()
        .example("sculpt.borderLock ?"),
    script("sculpt.cornerLock", Sculpt, "Locks mesh corners while sculpting (toggles when bare).")
        .args(&[opt("state", ON_OFF_QUERY, N, None)])
        .query()
        .example("sculpt.cornerLock ?"),
];
