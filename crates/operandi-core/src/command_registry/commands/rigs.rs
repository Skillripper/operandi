//! Character rig commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // Character rigs (character_rig.rs, BEHAVIOR-ACS-FEATURES; after ACS
    // by Lukasz Pazera, MIT). ACS-00: each is reachable over MCP.
    script("operandi.acs.rigAdd", Items, "New Biped Rig (ACS-10, ACS-12): a rig root with a Biped's guides in the Guide context, ready to fit to a character.")
        .on_editor(
            "Add a Biped rig: a root Group Locator named name holding a Guides group with the Biped's guides (spine, head, arms and legs, left and right), height tall, standing at position facing +Z. One undo step; the rig is selected.",
            &[
            jopt("preset", "string", "biped (default; the only preset so far)."),
            jopt("name", "string", "The root's name (default Biped)."),
            jopt("height", "number", "Reference size: the character's height in metres (default 1.8)."),
            jopt("position", "string", "Where its feet stand, x,y,z (default 0,0,0)."),
        ],
            crate::character_rig_commands::run,
        )
        .undo()
        .operandi()
        .example("operandi.acs.rigAdd height:1.8"),
    script("operandi.acs.bind", Items, "Bind Meshes (ACS-41): binds meshes to the rig's bind skeleton in the Form context (Setup mode).")
        .on_editor(
            "Bind the meshes (items, default the selected meshes) to the rig's skeleton from its root joint, one undo step; the rig goes to the Form context. Takes operandi.skin.bind's options (type heat by default, falloff, segments, limitWeights, weightsPerVertex, minimumWeight, dualQuaternion).",
            &[
            jopt("rig", "integer", "The rig's root (default the selected rig, else the newest)."),
            jopt("items", "string", "Mesh item ids, comma separated (default the selected meshes)."),
            jopt("type", "string", "rigid, distance, visible or heat (default heat)."),
            jopt("falloff", "string", "low, medium (default) or high."),
            jopt("limitWeights", "boolean", "Keep at most weightsPerVertex joints a point (default false)."),
            jopt("weightsPerVertex", "integer", "1 to 8 (default 4)."),
            jopt("dualQuaternion", "boolean", "The skin blends by dual quaternions (default false)."),
        ],
            crate::character_rig_commands::run,
        )
        .undo()
        .operandi()
        .example("operandi.acs.bind type:distance"),
    script("operandi.acs.build", Items, "Build Skeleton / Apply Guides (ACS-32, ACS-40): the bind skeleton snaps to the guides, made the first time.")
        .on_editor(
            "Build the rig's bind skeleton from its guides under a Skeleton group, or move the existing joints onto the guides, one undo step in Setup mode. Joints keep their ids, so bound meshes keep their weights; joints a lower option count drops are deleted unless a mesh is bound to them.",
            &[jopt("rig", "integer", "The rig's root (default the selected rig, else the newest).")],
            crate::character_rig_commands::run,
        )
        .undo()
        .operandi()
        .example("operandi.acs.build"),
    script("operandi.acs.context", Items, "Rig context (ACS-01, ACS-02): Assemble, Guide, Meshes, Form or Animate; sets what the rig shows. Not an undo step.")
        .on_editor(
            "Switch the rig's context: guide shows only its guides, the others its skeleton; every context but animate is Setup mode. A view state saved with the scene: not an undo step.",
            &[
            jopt("rig", "integer", "The rig's root (default the selected rig, else the newest)."),
            jreq("context", "string", "assembly, guide, meshes, form or animate."),
        ],
            crate::character_rig_commands::run,
        )
        .ui()
        .operandi()
        .example("operandi.acs.context context:form"),
    script("operandi.acs.guideFit", Items, "Fit Guides to Mesh (ACS-34): the rig takes the mesh's height and place, and each guide moves into the middle of its limb.")
        .on_editor(
            "Fit the rig's guides to a character mesh (mesh, default the selected mesh), one undo step: the rig stands on the mesh's floor at its centre, its reference size becomes the mesh's height and the guides go to their places; with embed (default true) each guide moves into the middle of the mesh where it is (limb centres, head top, finger and toe tips). Best on a T-pose character facing +Z.",
            &[
            jopt("rig", "integer", "The rig's root (default the selected rig, else the newest)."),
            jopt("mesh", "integer", "The character's mesh item (default the selected mesh)."),
            jopt("embed", "boolean", "Move guides into the mesh (default true)."),
        ],
            crate::character_rig_commands::run,
        )
        .undo()
        .operandi()
        .example("operandi.acs.guideFit embed:true"),
    script("operandi.acs.guideMirror", Items, "Mirror Guides (ACS-21): one side's guides copied to the other, mirrored across the rig's centre.")
        .on_editor(
            "Mirror the guides from one side to the other across the rig root's centre plane, one undo step. Centre guides go onto the plane.",
            &[
            jopt("rig", "integer", "The rig's root (default the selected rig, else the newest)."),
            jopt("from", "string", "left (default) or right."),
        ],
            crate::character_rig_commands::run,
        )
        .undo()
        .operandi()
        .example("operandi.acs.guideMirror from:left"),
    script("operandi.acs.guideMove", Items, "Moves a guide (ACS-30, ACS-31): with guide symmetry on, its twin on the other side follows mirrored.")
        .on_editor(
            "Move a guide (an id or a role such as leg.L.knee) to a world position, one undo step; its child guides move with it. With symmetry on, the other side's twin follows and a centre guide stays on the centre plane.",
            &[
            jopt("rig", "integer", "The rig's root (default the selected rig, else the newest)."),
            jreq("guide", "string", "The guide's id or role."),
            jreq("position", "string", "World position x,y,z."),
        ],
            crate::character_rig_commands::run,
        )
        .undo()
        .operandi()
        .example("operandi.acs.guideMove guide:leg.L.knee position:0.12,0.5,0.03"),
    script("operandi.acs.guideReset", Items, "Reset Guides: every guide back to the Biped's proportions at the rig's reference size; deleted guides come back.")
        .on_editor(
            "Put the rig's guides back at the Biped template for its reference size, relative to the root, one undo step. Guides deleted by hand are made again.",
            &[jopt("rig", "integer", "The rig's root (default the selected rig, else the newest).")],
            crate::character_rig_commands::run,
        )
        .undo()
        .operandi()
        .example("operandi.acs.guideReset"),
    script("operandi.acs.options", Items, "Rig options (ACS-11, ACS-22): vertebrae, neck and twist joint counts, guide symmetry and reference size; a built skeleton rebuilds.")
        .on_editor(
            "Set the rig's options, one undo step: spine (vertebrae 1-8), neck (1-4), upperArmTwist, forearmTwist, thighTwist, shinTwist (0-4, both sides), symmetry (guide symmetry) and referenceSize (scales the guides about the root). A built skeleton rebuilds to the new counts.",
            &[
            jopt("rig", "integer", "The rig's root (default the selected rig, else the newest)."),
            jopt("spine", "string", "Vertebrae between hips and chest, 1 to 8 (default 3); more or fewer steps it, as for the other counts."),
            jopt("neck", "string", "Neck joints, 1 to 4 (default 1)."),
            jopt("upperArmTwist", "string", "Upper arm twist joints, 0 to 4."),
            jopt("forearmTwist", "string", "Forearm twist joints, 0 to 4."),
            jopt("thighTwist", "string", "Thigh twist joints, 0 to 4."),
            jopt("shinTwist", "string", "Shin twist joints, 0 to 4."),
            jopt("symmetry", "string", "Guide symmetry (ACS-31): true, false or toggle."),
            jopt("referenceSize", "number", "The character's height in metres; guides scale to it."),
        ],
            crate::character_rig_commands::run,
        )
        .undo()
        .operandi()
        .example("operandi.acs.options spine:4"),
    script("operandi.acs.query", Items, "Reads the scene's character rigs as JSON (UIC-163): context, options, modules, guides, joints and bound meshes.")
        .on_editor(
            "Read rigs as JSON: root, name, preset, context, symmetry, referenceSize, options, modules (with their guides), guides and joints (role, id, name, side, parent, world position) and boundMeshes. rig limits it to one.",
            &[jopt("rig", "integer", "Only this rig.")],
            crate::character_rig_commands::run,
        )
        .query()
        .quiet()
        .operandi()
        .example("operandi.acs.query"),
    script("operandi.weight.adjust", Sculpt, "Adjust Weights (SK-42): adds Weight to a map over the selected points (or all), or sets it.")
        .on_editor(
            "Change a weight map over the points, one undo step: additive (default) adds weight (negative lowers), off sets it; normalize lowers the other maps so totals stay at most 100%.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jopt("points", "string", "Point indices, comma separated (default the selected points, else all)."),
            jreq("weight", "number", "The amount (0-1; negative lowers when additive)."),
            jopt("additive", "boolean", "Add (default true) or set."),
            jopt("normalize", "boolean", "Keep totals at most 100% by lowering the other maps."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.adjust weight:0.1"),
    script("operandi.weight.isolate", Sculpt, "Isolate Weights (SK-42): clears a map outside the selected points.")
        .on_editor(
            "Remove the map's values outside the points (default the selected points), one undo step.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jopt("points", "string", "Point indices, comma separated (default the selected points, else all)."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.isolate map:Elbow1"),
    script("operandi.weight.merge", Sculpt, "Merge Weights (SK-42): adds one map into another (capped at 100%).")
        .on_editor(
            "Add map from into map into (default the selected map), capped at 1, and remove from unless remove is false; one undo step.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jreq("from", "string", "The map merged in."),
            jopt("into", "string", "The map it goes into (default the selected one)."),
            jopt("remove", "boolean", "Remove from afterwards (default true)."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.merge from:Hand1 into:Wrist1"),
    script("operandi.weight.mirror", Sculpt, "Mirror Weights (SK-42): copies weights across the mirror plane, swapping _L and _R maps.")
        .on_editor(
            "Mirror every weight map (or map) across axis (default X) from the + side to the − side (from:minus the other way), matching points by position within tolerance, _L/_R maps swapped; one undo step.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jopt("axis", "string", "X (default), Y or Z."),
            jopt("from", "string", "plus (default) or minus: the side copied."),
            jopt("tolerance", "number", "Matching distance (default 0.0001)."),
            jopt("suffixPlus", "string", "Default _L."),
            jopt("suffixMinus", "string", "Default _R."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.mirror"),
    script("operandi.weight.normalize", Sculpt, "Normalize Weights (SK-42): each point's weights add up to 100%.")
        .on_editor(
            "Scale each point's weights (the skin's joint maps on a bound mesh, else every weight map) to add up to 1, over the points (default selected, else all); one undo step.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jopt("points", "string", "Point indices, comma separated (default the selected points, else all)."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.normalize"),
    script("operandi.weight.query", Sculpt, "Reads weight maps as JSON (UIC-163): the mesh's maps, one map's values, or every map's value at a point.")
        .on_editor(
            "Read weight maps as JSON: item, maps, skin (its joint maps), selected; with map, values (point: weight) or with vertex its value; with vertex alone, weights (map: weight) and total.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jopt("vertex", "integer", "A point index."),
        ],
            crate::skin_weights::run,
        )
        .query()
        .quiet()
        .operandi()
        .example("operandi.weight.query vertex:0"),
    script("operandi.weight.set", Sculpt, "Writes weight values per point (§7 MCP), or erases them (no value).")
        .on_editor(
            "Set map's value at points to value (0-1), or erase:true removes it; normalize lowers the other maps; one undo step.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jreq("points", "string", "Point indices, comma separated."),
            jopt("value", "number", "0 to 1."),
            jopt("erase", "boolean", "Remove the value (no weight at all)."),
            jopt("normalize", "boolean", "Keep totals at most 100%."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.set map:Elbow1 points:0,1 value:0.5"),
    script("operandi.weight.smooth", Sculpt, "Smooth Selected (SK-42): one blur pass of a map towards its neighbours.")
        .on_editor(
            "Blur the map once over the points (default selected, else all): each moves amount (default 0.5) of the way to its neighbours' mean; one undo step.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jopt("points", "string", "Point indices, comma separated (default the selected points, else all)."),
            jopt("amount", "number", "0 to 1 (default 0.5)."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.smooth"),
    script("operandi.weight.split", Sculpt, "Split Left/Right (SK-42): one map into _L and _R maps by side.")
        .on_editor(
            "Split map by the side of axis (default X): + side into map_L, − side into map_R, points on the plane half to each; removes the map unless remove is false; one undo step.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jopt("axis", "string", "X (default), Y or Z."),
            jopt("remove", "boolean", "Remove the original (default true)."),
            jopt("suffixPlus", "string", "Default _L."),
            jopt("suffixMinus", "string", "Default _R."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.split map:Arm1"),
    script("operandi.weight.transfer", Sculpt, "Transfer Weights (SK-42): weights from another mesh, by nearest point.")
        .on_editor(
            "Copy every weight map (or map) of mesh from onto this mesh's points (default selected, else all): each takes the nearest source point's weights in world space; one undo step.",
            &[
            jopt("item", "integer", "The mesh (default the selected one)."),
            jopt("map", "string", "The weight map (default the selected one)."),
            jopt("points", "string", "Point indices, comma separated (default the selected points, else all)."),
            jreq("from", "integer", "The source mesh item."),
        ],
            crate::skin_weights::run,
        )
        .undo()
        .operandi()
        .example("operandi.weight.transfer from:3"),
    script("operandi.eval.graph", Items, "Reads the scene's evaluation graph (who reads whom: time, channels, fields, transforms, shown meshes) as JSON nodes and edges (UIC-163; the Schematic's data).")
        .on_editor(
            "Read the evaluation graph as JSON: nodes (id, kind time/channel/field/transform/shownMesh, item, name, label, dirty) and edges [from, to] (to reads from). item limits it to one item's nodes and the edges touching them.",
            &[jopt("item", "integer", "Only this item's nodes.")],
            crate::eval_state::run,
        )
        .query()
        .quiet()
        .operandi()
        .example("operandi.eval.graph"),
    script("operandi.render.frameRange", Render, "Renders a frame range with Cycles, one numbered image per frame, into the project's renders folder (TL-70); needs the running app.")
        .on_editor(
            "Render frames start..end (default the current range) with Cycles into folder (default the project's renders/) as <name>_0000.png; needs the running app. operandi.anim.query reports progress (job).",
            &[
            jopt("start", "number", "First frame (default the current range's)."),
            jopt("end", "number", "Last frame (default the current range's)."),
            jopt("range", "string", "current (default) or scene: which range start and end default to."),
            jopt("folder", "string", "Folder to write to."),
            jopt("name", "string", "File name before the frame number (default Render)."),
            jopt("samples", "integer", "Samples per pixel (default the render settings')."),
            jopt("width", "integer", "Width in pixels (with height)."),
            jopt("height", "integer", "Height in pixels (with width)."),
            jopt("camera", "integer", "Camera item (default the render camera)."),
        ],
            crate::anim_commands::run,
        )
        .side_effect()
        .operandi()
        .example("operandi.render.frameRange start:0 end:24"),
    script("operandi.render.cancelRange", Render, "Stops a running frame render or playblast (TL-70).")
        .on_editor(
            "Stop the running frame range render or playblast.",
            &[],
            crate::anim_commands::run,
        )
        .side_effect()
        .operandi()
        .example("operandi.render.cancelRange"),
    script("operandi.view.playblast", Render, "Records the active 3D view over a frame range, HUD hidden, to a movie or an image sequence and opens it (TL-71); needs the running app.")
        .on_editor(
            "Record the active 3D view for frames start..end (default the current range) at scale 1, 0.5 or 0.25 to an image sequence in folder (default the project's renders/), then open it unless open is false; needs the running app.",
            &[
            jopt("start", "number", "First frame."),
            jopt("end", "number", "Last frame."),
            jopt("range", "string", "current (default) or scene."),
            jopt("scale", "number", "1, 0.5 or 0.25 of the view's size (default 1)."),
            jopt("folder", "string", "Folder to write to."),
            jopt("name", "string", "File name before the frame number (default Playblast)."),
            jopt("open", "boolean", "Open the result in the system player after (default true)."),
            jopt("format", "string", "movie (default: an .mp4, through ffmpeg when it is installed; else the images) or images (a PNG sequence)."),
        ],
            crate::anim_commands::run,
        )
        .side_effect()
        .operandi()
        .example("operandi.view.playblast scale:0.5"),
];
