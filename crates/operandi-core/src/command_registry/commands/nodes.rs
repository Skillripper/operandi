//! Node graph and MaterialX commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // Nodes N1 (NM spec). Names confirmed by the registry owner
    // 2026-11-11; Operandi-only, the map has no node-graph commands. Shared
    // graphs are scene data, not items, so their defaults stay here rather
    // than in the item.channel table.
    script("operandi.nodeMaterial.import", Shading, "Imports a node graph file (JSON) as Node Material layers, all or nothing (NM-05).")
        .operandi()
        .headless(
            "Import a node graph file (JSON) as Node Material layers, placed as Add Layer places one; all or nothing per material. One undo step.",
            &[
            jreq("filename", "string", "The node graph file."),
            jopt(
                "layer",
                "integer",
                "A shader-tree layer the new layers go next to; default as Add Layer with nothing selected.",
            ),
        ],
        )
        .args(&[arg("filename", ArgType::Path, N), opt("layer", INT, N, None)])
        .undo(),
    script("operandi.nodeMaterial.showGraph", Shading, "Shows a Node Material's graph read-only in the Node View panel, docking it under the views if it isn't open (NM-15); bare, the selected Node Material layer's. A Material Group shows its own MaterialX graph (Show as Nodes).")
        .operandi()
        .ui()
        .args(&[opt("layer", INT, N, None), opt("graph", INT, N, None)]),
    script("operandi.nodeMaterial.setDefault", Shading, "Sets (or with ? reads) a node graph's shared exposed default; layers without their own value follow (NM-04). Graphs aren't items, so this isn't an item.channel.")
        .operandi()
        .headless(
            "Set a node graph's shared exposed default (layers without their own value follow), one undo step; value \"?\" answers the current default.",
            &[
            jreq(
                "graph",
                "integer",
                "The graph's id (shader_tree_get lists node graphs).",
            ),
            jreq("input", "string", "The exposed input's name, e.g. Tint."),
            jreq(
                "value",
                "any",
                "A number, r,g,b text or [r, g, b] (0..1 components); \"?\" reads it.",
            ),
        ],
        )
        .args(&[arg("graph", INT, N), arg("input", TEXT, N), arg("value", TEXT, N)])
        .undo()
        .example("operandi.nodeMaterial.setDefault graph:1 input:Tint value:1,0.5,0.5"),
    // Node editing (node_view_commands.rs; placeholder names, NAMES-NEEDED.md).
    script("operandi.nodes.add", Shading, "Adds a free MaterialX node from the node view's Add menu to a Material Group (NE-02). One undo step.")
        .operandi()
        .headless("Add a free MaterialX node (noise3d, multiply... with its output type) to a Material Group's node view.", &[
            jreq("group", "integer", "The Material Group's layer id."),
            jreq("node", "string", "The MaterialX category, e.g. noise3d."),
            jopt("type", "string", "Its output type, e.g. color3 (the default) or float."),
            jopt("x", "number", "Canvas x."),
            jopt("y", "number", "Canvas y."),
        ])
        .args(&[arg("group", INT, N), arg("node", TEXT, N), opt("type", TEXT, N, None), opt("x", FLOAT, N, None), opt("y", FLOAT, N, None)])
        .undo()
        .example("operandi.nodes.add group:5 node:noise3d type:color3"),
    script("operandi.nodes.connect", Shading, "Wires a free node's output into a free node's input or a Material layer's channel, which it then drives (NE-03). One undo step.")
        .operandi()
        .headless("Wire a free node into <node>.<input> or layer:<id>.<channel>; the input's own value is kept but stops applying.", &[
            jreq("group", "integer", "The Material Group's layer id."),
            jreq("from", "string", "The free node."),
            jopt("output", "string", "Its output (out)."),
            jreq("to", "string", "<node>.<input> or layer:<id>.<channel>, e.g. layer:4.diffuse_color."),
        ])
        .args(&[arg("group", INT, N), arg("from", TEXT, N), opt("output", TEXT, N, None), arg("to", TEXT, N)])
        .undo()
        .example("operandi.nodes.connect group:5 from:Noise to:layer:4.diffuse_color"),
    script("operandi.nodes.delete", Shading, "Deletes node-view boxes: a free node with its wires, or a layer's box (deleting the layer); mixing nodes are locked (NE-05, NE-06). One undo step.")
        .operandi()
        .headless("Delete free nodes or layer boxes (layer:<id>) from a Material Group's node view; mixing nodes are refused.", &[
            jreq("group", "integer", "The Material Group's layer id."),
            jreq("node", "string", "Box names, comma-separated."),
        ])
        .args(&[arg("group", INT, N), arg("node", TEXT, N)])
        .undo(),
    script("operandi.nodes.disconnect", Shading, "Removes the wire into an input: its own value applies again (NE-03). One undo step.")
        .operandi()
        .headless("Remove the wire into <node>.<input> or layer:<id>.<channel>.", &[
            jreq("group", "integer", "The Material Group's layer id."),
            jreq("to", "string", "<node>.<input> or layer:<id>.<channel>."),
        ])
        .args(&[arg("group", INT, N), arg("to", TEXT, N)])
        .undo(),
    script("operandi.nodes.hide", Shading, "Hides node-view boxes in the view only (NE-07). One undo step.")
        .operandi()
        .headless("Hide boxes in a Material Group's node view; renders don't change.", &[
            jreq("group", "integer", "The Material Group's layer id."),
            jreq("node", "string", "Box names, comma-separated."),
        ])
        .args(&[arg("group", INT, N), arg("node", TEXT, N)])
        .undo(),
    script("operandi.nodes.move", Shading, "Places a node-view box (NE-09, NE-16). One undo step per drag; renders don't change.")
        .operandi()
        .headless("Place a box in a Material Group's node view.", &[
            jreq("group", "integer", "The Material Group's layer id."),
            jreq("node", "string", "The box."),
            jreq("x", "number", "Canvas x."),
            jreq("y", "number", "Canvas y."),
        ])
        .args(&[arg("group", INT, N), arg("node", TEXT, N), arg("x", FLOAT, N), arg("y", FLOAT, N)])
        .undo(),
    script("operandi.nodes.setValue", Shading, "Sets a free node's input value (NE-01). One undo step; a drag is one step on release.")
        .operandi()
        .headless("Set a free node's input value (a number, or comma-separated numbers).", &[
            jreq("group", "integer", "The Material Group's layer id."),
            jreq("node", "string", "The free node."),
            jreq("input", "string", "Its input."),
            jreq("value", "any", "A number or comma-separated numbers."),
        ])
        .args(&[arg("group", INT, N), arg("node", TEXT, N), arg("input", TEXT, N), arg("value", TEXT, N)])
        .undo(),
    script("operandi.nodes.show", Shading, "Shows hidden node-view boxes (NE-07). One undo step.")
        .operandi()
        .headless("Show hidden boxes in a Material Group's node view.", &[
            jreq("group", "integer", "The Material Group's layer id."),
            jreq("node", "string", "Box names, comma-separated."),
        ])
        .args(&[arg("group", INT, N), arg("node", TEXT, N)])
        .undo(),
    script("operandi.nodes.showAll", Shading, "Shows every hidden node-view box (NE-07). One undo step.")
        .operandi()
        .headless("Show every hidden box in a Material Group's node view.", &[jreq("group", "integer", "The Material Group's layer id.")])
        .args(&[arg("group", INT, N)])
        .undo(),
    script("preset.saveAs", Presets, "Saves a preset; without a filename opens the save panel. mtlx:true (an Operandi extra, default off; the Preset Browser gear toggle sets it) also writes a .mtlx beside a material preset. kind:environment or item:<environment> saves an environment preset (.openv) of every environment; lights:, meshes: and cameras: (Operandi extras, default false) add every light, the selected meshes and every camera; centerOnGround:true (an Operandi extra, default off) saves an item preset's copy centred on X/Z and resting on Y = 0; bare, its options dialog opens.")
        .args(&[
            opt(
                "kind",
                ArgType::Choice(&["item", "assembly", "environment", "renderSettings", "material"]),
                N,
                None,
            ),
            opt("filename", ArgType::Path, N, None),
            opt("mtlx", BOOL, N, Some("false")),
            opt("item", INT, N, None),
            opt("lights", BOOL, N, Some("false")),
            opt("meshes", BOOL, N, Some("false")),
            opt("cameras", BOOL, N, Some("false")),
            opt("centerOnGround", BOOL, N, Some("false")),
        ]).side_effect(),
    script("preset.do", Presets, "Loads a preset as one undo step; also what the Preset Browser shows (texture sets, HDRs, pictures, scenes). target: (an Operandi extra) names the material group a material goes into; without a preset path opens the file panel. mode: (an Operandi extra) applies an environment preset as replace (default) or add.")
        .args(&[
            opt("preset", ArgType::Path, N, None),
            opt("handler", TEXT, N, None),
            opt("target", TEXT, N, None),
            opt("mode", ArgType::Choice(&["replace", "add", "0", "1"]), N, Some("replace")),
        ])
        .undo(),
    // MaterialX (docs/specs/openpbr-materialx): Operandi's own names
    // (registry ruling 2026-11-12).
    script("operandi.materialx.import", Presets, "Imports a MaterialX material as a Material and Image Map layers, directly above layer: or at the top of the shader tree without one; one that needs Node Material is refused whole. One undo step. Without a filename opens the file panel.")
        .operandi()
        .args(&[opt("filename", ArgType::Path, N, None), opt("layer", INT, N, None)])
        .undo(),
    script("operandi.materialx.export", Presets, "Writes material or material group item: (id or name; without one, the selected one) as an OpenPBR MaterialX material, listing what it couldn't write. Not undoable. Without a filename opens the save panel.")
        .operandi()
        .args(&[opt("filename", ArgType::Path, N, None), opt("item", TEXT, N, None)])
        .side_effect(),
];
