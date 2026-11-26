//! Remesh, Multiresolution and bakes (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // Remesh names from the registry thread (2026-11-11): mesh.retopo and
    // sculpt.remesh are the reference's; Blocks / Smooth / Sharp have no
    // reference command, so they are Operandi's. Bare opens the dialog.
    script("mesh.retopo", Modeling, "Automatic Retopology: rebuilds the active mesh as even quads that follow its shape; bare opens the dialog, unnamed attributes keep the last values.")
        .headless(
            "Automatic Retopology: rebuilds the active mesh as even quads that follow its shape (QuadriFlow); unnamed attributes keep the last values.",
            &[
            jopt(
                "countMode",
                "string",
                "number or percent (polyTargetNum or polyTargetPct).",
            ),
            jopt(
                "polyTargetNum",
                "integer",
                "About how many quads (default 5000).",
            ),
            jopt(
                "polyTargetPct",
                "number",
                "About this share of the polygon count, 0-10 (default 0.25).",
            ),
            jopt(
                "creaseAngle",
                "number",
                "Radians; new edges follow sharper creases, 0 is off (default 0).",
            ),
            jopt(
                "boundaryAlign",
                "boolean",
                "New edges follow open borders (default true).",
            ),
            jopt(
                "vertNormals",
                "boolean",
                "Smooth shade the result (default true).",
            ),
            jopt(
                "newMesh",
                "boolean",
                "Put the result in a new mesh item (default false).",
            ),
            jopt(
                "seed",
                "integer",
                "Operandi only: random start; the same seed gives the same quads.",
            ),
        ],
        )
        .args(&[
            opt("countMode", ArgType::Choice(&["number", "percent"]), N, Some("number")),
            opt("polyTargetNum", INT, N, Some("5000")),
            opt("polyTargetPct", FLOAT, Unit::Percent, Some("0.25")),
            opt("creaseAngle", FLOAT, Unit::Angle, Some("0")),
            opt("boundaryAlign", BOOL, N, Some("true")),
            opt("vertNormals", BOOL, N, Some("true")),
            opt("newMesh", BOOL, N, Some("false")),
            // Operandi only (docs/COMMAND_REGISTRY.md).
            opt("seed", INT, N, Some("0")),
        ])
        .undo()
        .example("mesh.retopo polyTargetNum:2000 creaseAngle:30deg"),
    script("sculpt.remesh", Sculpt, "Voxel Remesh: rebuilds the active mesh as an even quad skin on a voxel grid, joining overlaps and closing holes; bare opens the dialog.")
        .headless(
            "Voxel Remesh: rebuilds the active mesh as an even quad skin on a voxel grid; joins overlaps and closes holes. Unnamed attributes keep the last values.",
            &[
            jopt("density", "number", "0-1, higher is finer (default 0.5)."),
            jopt(
                "voxelSize",
                "number",
                "Operandi only: voxel size in metres; above 0 it replaces density.",
            ),
            jopt(
                "keepMaps",
                "boolean",
                "Operandi only: weight and colour maps carry over (default true).",
            ),
            jopt(
                "vertNormals",
                "boolean",
                "Operandi only: smooth shade the result (default true).",
            ),
        ],
        )
        .args(&[
            opt("density", FLOAT, Unit::Percent, Some("0.5")),
            // Operandi only (docs/COMMAND_REGISTRY.md).
            opt("voxelSize", FLOAT, Unit::Distance, Some("0")),
            opt("keepMaps", BOOL, N, Some("true")),
            opt("vertNormals", BOOL, N, Some("true")),
        ])
        .undo()
        .example("sculpt.remesh density:0.6"),
    // Multiresolution (multires.rs): Operandi's own names until the
    // reference name map has some (MR-Q08).
    script("operandi.multires.subdivide", Sculpt, "Add Level: subdivides the active mesh one more multiresolution level and steps to it; the first one makes the mesh multiresolution.")
        .on_editor(
            "Add a multiresolution level above the top one (Catmull-Clark) and step to it; the first one makes the mesh multiresolution, with its current shape as the base. One undo step.",
            &[jopt("item", "integer", "The mesh item (default the active one).")],
            crate::multires::run,
        )
        .undo()
        .operandi()
        .example("operandi.multires.subdivide"),
    script("operandi.multires.level", Sculpt, "Steps the active multiresolution mesh to a level (0 is the base), or one up or down; sculpted detail carries across.")
        .on_editor(
            "Step a multiresolution mesh to level (0 = base), or step:up / step:down. Detail sculpted at higher levels rides along on changes made at lower ones. One undo step.",
            &[
            jopt("level", "integer", "The level to show, 0 (the base) up to the top."),
            jopt("step", "string", "up or down, instead of level."),
            jopt("item", "integer", "The mesh item (default the active one)."),
        ],
            crate::multires::run,
        )
        .undo()
        .operandi()
        .example("operandi.multires.level step:down"),
    script("operandi.multires.deleteHigher", Sculpt, "Delete Higher Levels: removes the multiresolution levels above the current one.")
        .on_editor(
            "Delete the multiresolution levels above the current one (at the base, the mesh becomes plain again). One undo step.",
            &[jopt("item", "integer", "The mesh item (default the active one).")],
            crate::multires::run,
        )
        .undo()
        .operandi()
        .example("operandi.multires.deleteHigher"),
    script("operandi.multires.apply", Sculpt, "Apply Multiresolution: keeps the mesh at its current level as a plain mesh and drops the other levels.")
        .on_editor(
            "Keep the mesh as it is at its current level as a plain mesh, dropping the base and the other levels. One undo step.",
            &[jopt("item", "integer", "The mesh item (default the active one).")],
            crate::multires::run,
        )
        .undo()
        .operandi()
        .example("operandi.multires.apply"),
    script("operandi.multires.query", Sculpt, "Reads the active mesh's multiresolution levels as JSON (UIC-163).")
        .on_editor(
            "Read a mesh's multiresolution state as JSON: multiresolution (bool), level, levels, polygons per level (base first) and in_step (false once the mesh's polygons changed under it).",
            &[jopt("item", "integer", "The mesh item (default the active one).")],
            crate::multires::run,
        )
        .query()
        .quiet()
        .operandi()
        .example("operandi.multires.query"),
    // Bakes from TextureCraft (texturecraft-mining-2026-11-17): Operandi's
    // own names (no map name), placeholders in docs/NAMES-NEEDED.md.
    script("operandi.bake.curvature", Shading, "Bakes the active mesh's Curvature map (convex edges light, cavities dark, flat 50 % grey) into its UVs as a 16-bit greyscale PNG, with padding. Not undoable; bare opens the dialog.")
        .operandi()
        .headless(
            "Bake the active mesh's Curvature map into its UV layout and write it as a 16-bit greyscale PNG: convex edges light, cavities dark, flat 50 % grey. The scene doesn't change.",
            &[
            jreq("filename", "string", "The .png to write."),
            jopt("size", "integer", "Width and height in pixels, 16-16384 (default 1024)."),
            jopt("padding", "integer", "Pixels of padding around each UV island (default 16)."),
            jopt("fill", "boolean", "Fill the rest of the image smoothly (default true)."),
            jopt("uvMap", "string", "UV map baked into (default the mesh's Texture map)."),
            jopt("rays", "integer", "Rays per pixel, 1-1024 (default 32)."),
            jopt("radius", "number", "Sampling radius as a share of the mesh's size (default 0.001)."),
        ],
        )
        .args(&[
            opt("filename", ArgType::Path, N, None),
            opt("size", INT, N, Some("1024")),
            opt("padding", INT, N, Some("16")),
            opt("fill", BOOL, N, Some("true")),
            opt("uvMap", TEXT, N, None),
            opt("rays", INT, N, Some("32")),
            opt("radius", FLOAT, N, Some("0.001")),
        ])
        .side_effect()
        .example("operandi.bake.curvature filename:/tmp/curvature.png size:2048"),
    script("operandi.bake.thickness", Shading, "Bakes the active mesh's Thickness map (thin parts dark, thick parts light) into its UVs as a 16-bit greyscale PNG, with padding. Not undoable; bare opens the dialog.")
        .operandi()
        .headless(
            "Bake the active mesh's Thickness map into its UV layout and write it as a 16-bit greyscale PNG: thin parts dark, thick parts light. The scene doesn't change.",
            &[
            jreq("filename", "string", "The .png to write."),
            jopt("size", "integer", "Width and height in pixels, 16-16384 (default 1024)."),
            jopt("padding", "integer", "Pixels of padding around each UV island (default 16)."),
            jopt("fill", "boolean", "Fill the rest of the image smoothly (default true)."),
            jopt("uvMap", "string", "UV map baked into (default the mesh's Texture map)."),
            jopt("rays", "integer", "Rays per pixel, 1-1024 (default 64)."),
            jopt("distance", "number", "Longest distance measured, as a share of the mesh's size (default 0.1)."),
        ],
        )
        .args(&[
            opt("filename", ArgType::Path, N, None),
            opt("size", INT, N, Some("1024")),
            opt("padding", INT, N, Some("16")),
            opt("fill", BOOL, N, Some("true")),
            opt("uvMap", TEXT, N, None),
            opt("rays", INT, N, Some("64")),
            opt("distance", FLOAT, N, Some("0.1")),
        ])
        .side_effect()
        .example("operandi.bake.thickness filename:/tmp/thickness.png"),
    script("operandi.bake.padImage", Shading, "Pads a PNG texture around the active mesh's UV islands so seams don't show background when filtered; writes output: or the file itself. Not undoable; bare opens the dialog.")
        .operandi()
        .headless(
            "Pad a PNG texture around the active mesh's UV islands (seam padding), writing output or the file itself. The scene doesn't change.",
            &[
            jreq("filename", "string", "The .png to pad."),
            jopt("output", "string", "Where to write the result (default: replace filename)."),
            jopt("padding", "integer", "Pixels of padding (default 16)."),
            jopt("uvMap", "string", "UV map whose islands are used (default the mesh's Texture map)."),
        ],
        )
        .args(&[
            opt("filename", ArgType::Path, N, None),
            opt("output", ArgType::Path, N, None),
            opt("padding", INT, N, Some("16")),
            opt("uvMap", TEXT, N, None),
        ])
        .side_effect()
        .example("operandi.bake.padImage filename:/tmp/color.png padding:8"),
    script("operandi.remesh", Modeling, "Remesh Blocks, Smooth or Sharp: rebuilds the active mesh on an octree; bare opens the dialog.")
        .operandi()
        .headless(
            "Remesh Blocks, Smooth or Sharp: rebuilds the active mesh on an octree-sized grid. Unnamed attributes keep the last values.",
            &[
            jopt(
                "mode",
                "string",
                "blocks, smooth or sharp (default smooth).",
            ),
            jopt("depth", "integer", "Octree Depth 2-12 (default 6)."),
            jopt(
                "scale",
                "number",
                "How much of the octree the mesh fills, 0-1 (default 0.9).",
            ),
            jopt(
                "sharpness",
                "number",
                "Sharp mode: higher keeps more edges sharp (default 1).",
            ),
            jopt(
                "removeDisconnected",
                "boolean",
                "Drop small loose pieces (default false).",
            ),
            jopt(
                "threshold",
                "number",
                "With removeDisconnected: smallest piece kept, as a share of the biggest, 0-1 (default 1).",
            ),
            jopt(
                "vertNormals",
                "boolean",
                "Smooth shade the result (default true).",
            ),
        ],
        )
        .args(&[
            opt("mode", ArgType::Choice(&["blocks", "smooth", "sharp"]), N, Some("smooth")),
            opt("depth", INT, N, Some("6")),
            opt("scale", FLOAT, Unit::Percent, Some("0.9")),
            opt("sharpness", FLOAT, N, Some("1")),
            opt("removeDisconnected", BOOL, N, Some("false")),
            opt("threshold", FLOAT, Unit::Percent, Some("1")),
            opt("vertNormals", BOOL, N, Some("true")),
        ])
        .undo()
        .example("operandi.remesh mode:sharp depth:7"),
    script("poly.boolean", Modeling, "Boolean of the active mesh with its drivers (Union, Intersect, Subtract, Add); bare opens the dialog.")
        .headless(
            "Boolean of the active mesh with its drivers (BOOL-25); the drivers are untouched.",
            &[
            jopt(
                "mode",
                "integer|string",
                "0 union (default), 1 intersect, 2 subtract, 3 add.",
            ),
            jopt(
                "cutmesh",
                "integer|string",
                "Drivers: 0 background (default: visible unselected meshes), 1 lastSelected (the selected pieces of the active mesh), 2 item.",
            ),
            jopt(
                "item",
                "integer",
                "Driver item id for cutmesh item (a group drives with its meshes).",
            ),
            jopt(
                "children",
                "boolean",
                "Add the drivers' child meshes; default false.",
            ),
        ],
        )
        .args(&[
            opt("mode", ArgType::Choice(&["union", "intersect", "subtract", "add"]), N, Some("union")),
            opt("cutmesh", ArgType::Choice(&["background", "lastSelected", "item"]), N, Some("background")),
            opt("item", TEXT, N, None),
            opt("children", BOOL, N, Some("false")),
        ])
        .undo()
        .example("poly.boolean mode:subtract"),
    script("poly.solidDrill", Modeling, "Solid Drill: cuts the active mesh's polygons with a closed driver (Core, Tunnel, Stencil, Slice); bare opens the dialog.")
        .headless(
            "Solid Drill: cut the active mesh's polygons with a closed driver (DRILL-06); no driver faces are added.",
            &[
            jreq(
                "mode",
                "integer|string",
                "0 core, 1 tunnel, 2 stencil, 3 slice (or the word).",
            ),
            jopt(
                "material",
                "string",
                "Stencil's material for the inside pieces; default Stencil, created if missing.",
            ),
            jopt(
                "cutmesh",
                "integer|string",
                "Drivers: 0 background (default), 1 lastSelected, 2 item.",
            ),
            jopt("item", "integer", "Driver item id for cutmesh item."),
            jopt(
                "children",
                "boolean",
                "Add the drivers' child meshes; default false.",
            ),
        ],
        )
        .args(&[
            arg("mode", ArgType::Choice(&["core", "tunnel", "stencil", "slice"]), N),
            opt("material", TEXT, N, Some("Stencil")),
            opt("cutmesh", ArgType::Choice(&["background", "lastSelected", "item"]), N, Some("background")),
            opt("item", TEXT, N, None),
            opt("children", BOOL, N, Some("false")),
        ])
        .undo()
        .example("poly.solidDrill mode:tunnel"),
    script("poly.drill", Modeling, "Axis Drill: cuts the active mesh's polygons with the driver outline pushed along X, Y or Z; bare opens the dialog.")
        .headless(
            "Axis Drill: cut the active mesh's polygons with the driver outline's prism along an axis (DRILL-06).",
            &[
            jreq(
                "mode",
                "integer|string",
                "0 core, 1 tunnel, 2 stencil, 3 slice (or the word).",
            ),
            jreq("axis", "integer|string", "0 x, 1 y, 2 z (or the letter)."),
            jopt(
                "material",
                "string",
                "Stencil's material for the inside pieces; default Stencil, created if missing.",
            ),
            jopt(
                "cutmesh",
                "integer|string",
                "Drivers: 0 background (default), 1 lastSelected, 2 item.",
            ),
            jopt("item", "integer", "Driver item id for cutmesh item."),
            jopt(
                "children",
                "boolean",
                "Add the drivers' child meshes; default false.",
            ),
        ],
        )
        .args(&[
            arg("mode", ArgType::Choice(&["core", "tunnel", "stencil", "slice"]), N),
            arg("axis", ArgType::Choice(&["x", "y", "z"]), N),
            opt("material", TEXT, N, Some("Stencil")),
            opt("cutmesh", ArgType::Choice(&["background", "lastSelected", "item"]), N, Some("background")),
            opt("item", TEXT, N, None),
            opt("children", BOOL, N, Some("false")),
        ])
        .undo()
        .example("poly.drill mode:core axis:y"),
    script("poly.setMaterial", Modeling, "Tags the selected polygons with a material, creating it if new; bare opens the dialog.")
        .headless(
            "Set the material tag of the selected polygons (a new material when the name is new).",
            &[jreq("name", "string", "Material name.")],
        )
        .args(&[opt("name", TEXT, N, None)])
        .undo()
        .example("poly.setMaterial Clay"),
    script("poly.unify", Modeling, "Removes duplicate polygons; bare opens the options.")
        .headless(
            "Remove polygons that duplicate another (same points, either winding), keeping the lowest of each group; one undo step.",
            &[jopt("force", "boolean", "Also remove duplicates whose tags or corner data differ (default false).")],
        )
        .args(&[opt("force", BOOL, N, None)])
        .undo()
        .example("poly.unify false"),
    script("mesh.cleanup", Modeling, "Mesh Cleanup with twelve explicit switches (only floatingVertex is supported).")
        .headless(
            "Mesh Cleanup: run the checked operations on the selected polygons (the whole active mesh when none), one undo step (BRQ-60). Each switch not given takes the dialog's Reset value, on; all off is refused.",
            &[
            jopt("floatingVertex", "boolean", "Remove Floating Vertices (default true)."),
            jopt("onePointPolygon", "boolean", "Remove 1 Point Polygons (default true)."),
            jopt("twoPointPolygon", "boolean", "Remove 2 Point Polygons (default true)."),
            jopt("dupPointPolygon", "boolean", "Fix Duplicate Points in Polygons (default true)."),
            jopt("colinear", "boolean", "Remove Colinear Vertices (default true)."),
            jopt("faceNormal", "boolean", "Make Face Normals Consistent (default true)."),
            jopt("mergeVertex", "boolean", "Merge Overlapping Vertices (default true)."),
            jopt("mergeDisco", "boolean", "Merge Discontinuous UVs (default true)."),
            jopt("unifyPolygon", "boolean", "Unify Polygons (default true)."),
            jopt("forceUnify", "boolean", "Force Unify (default true)."),
            jopt("removeDiscoWeight", "boolean", "Remove Discontinuous Weights (default true)."),
            jopt("fixGaps", "boolean", "Fix Gaps (default true)."),
        ],
        )
        .args(CLEANUP_ARGS)
        .undo(),
    script("chamfer.edit", Modeling, "Applies Chamfer Edit once.")
        .headless(
            "Chamfer Edit once on the selected polygons; one undo step. Options not given take the Reset values.",
            &[
                jopt("offset", "number", "Metres (default 0)."),
                jopt("segments", "integer", "Default 1."),
                jopt("sourceSegs", "integer", "Default 1."),
                jopt("depth", "number", "Factor, 1 = 100 % (default 1)."),
                jopt("mode", "string", "inset (default) or width: what Offset changes."),
                jopt("absSize", "boolean", "Absolute Size: Offset is the inset or width itself (default false)."),
                jopt("swapSide", "boolean", "Swap Side: the strip runs the other way; Offset moves its open ends (default false)."),
                jopt("reverseX", "boolean", "Reverse Direction; no effect without a profile (default false)."),
                jopt("reverseY", "boolean", "Reverse Depth; no effect without a profile (default false)."),
            ],
        )
        .args(&[
            arg("offset", FLOAT, Unit::Distance),
            arg("segments", INT, N),
            arg("sourceSegs", INT, N),
            arg("depth", FLOAT, Unit::Percent),
        ])
        .undo()
        .example("chamfer.edit offset:0.1 segments:2 sourceSegs:1 depth:0.5"),
    script("poly.haul", Modeling, "Applies Poly Haul once.")
        .headless(
            "Poly Haul once on the selected polygons; one undo step.",
            &[jreq("offset", "number", "Metres.")],
        )
        .args(&[arg("offset", FLOAT, Unit::Distance)])
        .undo()
        .example("poly.haul offset:0.2"),
    script("operandi.sketch.sweep", Modeling, "Sketch Extrude along an explicit world path.")
        .operandi()
        .args(&[arg("path", ArgType::Json, N)])
        .undo()
        .example("operandi.sketch.sweep {\"path\":[[0,0,0],[1,0,0]]}"),
    script("@ppi_PolyScale_Offset.py", Modeling, "Opens Enter Inset Amount for Scale Offset.").ui(),
    script("@ppi_PolyScale_Offset_Detach.py", Modeling, "Scale Offset on new, unconnected points (opens its dialog).").ui(),
    script("@ppi_flatten_ngons.py", Modeling, "Flattens nonplanar polygons of the active mesh.")
        .headless(
            "Flatten the selected polygons of more than four sides.",
            &[],
        ).undo(),
    script("operandi.mesh.fixBlackDots", Modeling, "Fix Black Dots: merges polygon pairs that share a run of edges into one polygon, on the whole active mesh.")
        .operandi()
        .headless(
            "Fix Black Dots (MT2): every pair of polygons sharing two or more edges in a row (inner points used by those two only) becomes one polygon; the inner points are kept unused and selected with the run's ends, Vertex mode, one undo step. 'No black dots found.' when there are none.",
            &[],
        ).undo(),
    script("operandi.mesh.setData", Modeling, "Set Mesh Data: replaces a mesh item's points, and its polygons when given, as one undo step (scripts, K9).")
        .on_editor(
            "Set Mesh Data (K9): sets a mesh item's points (a list of [x, y, z] in item space, metres) and, when polygons is given, its polygons (lists of point indices; the mesh is rebuilt and its UV maps, vertex maps and component selection go). Points alone must keep the point count. One undo step; a bad mesh is refused and nothing changes.",
            &[
            jopt("item", "integer", "Mesh item id; default the active item."),
            jreq("points", "array", "Points as [x, y, z] lists."),
            jopt("polygons", "array", "Polygons as lists of point indices."),
        ],
            crate::script_mesh::run,
        )
        .undo()
        .operandi()
        .example("operandi.mesh.setData points:[[0,0,0],[1,0,0],[0,1,0]] polygons:[[0,1,2]]"),
    script("operandi.edge.zip", Modeling, "Edge Zip: welds the edge rows either side of the selected edges onto them, removing the quads between.")
        .operandi()
        .headless(
            "Edge Zip (MT3): each selected edge needs a quad on each side; its points and the points across those quads weld at their average, the quads collapse, one undo step, nothing selected afterwards.",
            &[],
        ).undo(),
    script("operandi.poly.diagonalRedirect", Modeling, "Diagonal Re-Direct: turns the edge flow through two diagonal quads 45 degrees.")
        .operandi()
        .headless(
            "Diagonal Re-Direct (MT4): two selected quads touching at one corner split along the diagonal through it; each side quad off the border gets a centre point and is rebuilt so the diagonal is an edge loop, one undo step.",
            &[],
        ).undo(),
    script("@ppi_vert_tear.py", Tools, "Starts the Tear tool.").tool(),
    script("poly.split", Modeling, "Connects pairs of selected vertices of one polygon with new edges.")
        .headless(
            "Split the selected polygons along selected vertices.",
            &[],
        ).undo(),
    script("poly.quadruple", Modeling, "Turns polygons laid out as two rows into a row of quads.")
        .headless(
            "Turn selected triangle pairs into quads.",
            &[],
        ).undo(),
    script("poly.spinQuads", Modeling, "Spins the shared edge of two selected polygons.")
        .headless(
            "Spin the edge shared by two selected quads.",
            &[],
        ).undo(),
    script("poly.align", Modeling, "Makes the selected polygons face the same way.")
        .headless(
            "Make the selected polygons face the same way as the first.",
            &[],
        ).undo(),
    script("poly.triple", Modeling, "Splits the selected polygons into triangles.")
        .headless(
            "Triangulate the selected polygons.",
            &[],
        ).undo(),
    script("operandi.solidSketch.strip", Modeling, "Removes the Solid Sketch data from the selected meshes, leaving plain polygons the tool can't pick up again.")
        .operandi()
        .undo(),
    script("poly.flip", Modeling, "Reverses the winding of the selected polygons.")
        .headless(
            "Flip the selected polygons' facing.",
            &[],
        ).undo(),
    script("poly.remove", Modeling, "Removes the selected polygons, keeping their vertices.")
        .headless(
            "Remove the selected polygons.",
            &[],
        ).undo(),
    script("poly.merge", Modeling, "Merges each connected region of selected polygons into one.")
        .headless(
            "Merge the selected polygons into one.",
            &[],
        ).undo(),
    script("poly.collapse", Modeling, "Collapses each selected region to a point.")
        .headless(
            "Collapse each selected polygon to a point.",
            &[],
        ).undo(),
    script("poly.make", Modeling, "Builds one quad from two selected boundary edges.")
        .headless(
            "Make a polygon from the selected vertices, in selection order.",
            &[],
        )
        .args(&[arg("type", ArgType::Choice(&["auto"]), N)])
        .undo()
        .example("poly.make auto"),
];
