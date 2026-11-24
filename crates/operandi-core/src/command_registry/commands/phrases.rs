//! Command phrases (Input Map, menus and pies) (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- commands
    script("cut", Modeling, "Cuts the selection to the clipboard.").spelled("Cut Selection").undo(),
    script("copy", Modeling, "Copies the selection to the clipboard.").spelled("Copy Selection").side_effect(),
    script("paste", Modeling, "Pastes the clipboard.").spelled("Paste Selection").undo(),
    script("layer.mergeMeshes", Items, "Merges the selected meshes into one.")
        .aliases(&["Merge Meshes"])
        .spelled("Item Merge")
        .undo(),
    script("layer.unmerge", Items, "Splits a mesh into its separate pieces.")
        .aliases(&["Unmerge Mesh"])
        .spelled("Item Unmerge")
        .undo(),
    action("Delete Item", Items, "Deletes the selection (in the Shading tab, the selected layers).")
        .operandi()
        .aliases(&["Delete Mesh"])
        .undo(),
    action("Delete Item Forward", Items, "Deletes the selection (forward delete key).")
        .operandi().undo(),
    action("Duplicate Hierarchy", Items, "Copies each selected item with all its children, keeping the links; the copies are selected.")
        .operandi().undo(),
    action("Duplicate Item", Items, "Copies the selected item in place, named Name (2), and selects the copy.")
        .operandi().undo(),
    action("New Mesh", Items, "Adds an empty mesh.")
        .operandi().undo(),
    action("Merge Instance", Items, "Bakes the selected mesh's one instance back into it, placed where the instance is; the instance item goes (missing-tools-g3 MT12).")
        .operandi().undo(),
    action("Instance", Items, "Adds an instance of each selected mesh item in the same place: it shares its source's mesh (DX-38).")
        .operandi().undo(),
    action("Freeze Instancer", Items, "Turns the selected Instancer into a group of instance items, one per copy, at the same places; the Instancer and its point cloud go (missing-tools-g8 RP-5).")
        .aliases(&["Freeze Replicator"]).operandi().undo(),
    action("Flip", Modeling, "Reverses the selected polygons.")
        .modo("poly.flip").undo(),
    action("Flip It", Modeling, "Mirrors the selected polygons across an axis through the selection's centre, facing out; with no axis, asks for one.")
        .operandi()
        .args(&[opt("axis", ArgType::Choice(&["x", "y", "z"]), N, None)])
        .example("flip it x")
        .undo(),
    action("Flip It Origin", Modeling, "Mirrors the selected polygons across an axis through the world origin, facing out; with no axis, asks for one.")
        .operandi()
        .args(&[opt("axis", ArgType::Choice(&["x", "y", "z"]), N, None)])
        .example("flip it origin x")
        .undo(),
    action("Toggle Subdivision", Modeling, "Toggles subdivision surfaces on the selection.")
        .operandi().undo(),
    action("Toggle Catmull-Clark", Modeling, "Toggles Catmull-Clark subdivision on the selection.")
        .operandi().undo(),
    action("Freeze", Modeling, "Freezes curves and subdivision surfaces into plain geometry with the last Freeze settings.")
        .operandi().undo(),
    action("SDS Subdivide", Modeling, "Subdivides the selection once (Catmull-Clark).")
        .modo("poly.subdivide ccsds").undo(),
    action("SDS Subdivide 2X", Modeling, "Subdivides the selection twice (Catmull-Clark), as one undo step.")
        .operandi().undo(),
    action("Faceted Subdivide", Modeling, "Subdivides the selection once (faceted).")
        .modo("poly.subdivide flat").undo(),
    action("Smooth Subdivide", Modeling, "Subdivides the selection once (smooth).")
        .modo("poly.subdivide smooth").undo(),
    action("Subdivide Options", Modeling, "Opens the Subdivide options.")
        .modo("poly.subdivide").ui(),
    action("UnSubdivide", Modeling, "Opens the UnSubdivide dialog.")
        .modo("poly.unsubdiv").ui(),
    action("Center X", Modeling, "Centres the selection on X.")
        .modo("vert.center x").undo(),
    action("Center Y", Modeling, "Centres the selection on Y.")
        .modo("vert.center y").undo(),
    action("Center Z", Modeling, "Centres the selection on Z.")
        .modo("vert.center z").undo(),
    action("Center All", Modeling, "Centres the selection on all axes.")
        .modo("vert.center all").undo(),
    action("Set Part", Modeling, "Opens Polygon Set Part.")
        .modo("poly.setPart").ui(),
    action("Set Position", Modeling, "Opens Set Vertex Position.")
        .modo("vert.set").ui(),
    action("Absolute Scaling", Modeling, "Opens or closes the Absolute Scaling window.")
        .operandi().ui(),
    action("Snaps and Precision", Tools, "Opens or closes the Snaps and Precision panel.")
        .operandi().ui(),
    action("Preset Browser", Presets, "Opens or closes the Preset Browser's window (layout.window hash:presetBrowser).")
        .operandi().ui(),
    action("Preset Browser Popover", Presets, "Opens the Preset Browser as a popover under the pointer (F6).")
        .operandi().ui(),
    action("Drag Snap", Tools, "Element Move with snapping: drag an element onto another.")
        .modo("tool.set SnapDrag on").tool(),
    action("Drag Snap Rigid", Tools, "Drag Snap moving everything connected rigidly.")
        .modo("tool.set SnapDragRigid on").tool(),
    action("Planar Rotate Snap", Tools, "Rotate about the clicked element's normal, snapping a rest and a target point.")
        .modo("tool.set PlanarRotateSnap on").tool(),
    action("Planar Rotate Snap Axis", Tools, "Planar Rotate Snap about an axis set from two snapped points.")
        .modo("tool.set PlanarRotateSnapAxis on").tool(),
    action("Scale Absolute", Tools, "Scale by snapping a base handle and a scale handle to points.")
        .modo("tool.set ScaleAbsolute on").tool(),
    action("Size Absolute", Tools, "Scale from a snapped base size to a typed or snapped size.")
        .modo("tool.set SizeAbsolute on").tool(),
    action("Advanced Handles", Tools, "Sets the tool handle style to Advanced.")
        .modo("tool.handleStyle advanced").ui(),
    action("Set Material", Modeling, "Opens Set Material.")
        .modo("poly.setMaterial").ui(),
    action("Unify", Modeling, "Opens Unify Polygons.")
        .modo("poly.unify").ui(),
    action("Scale Offset", Modeling, "Opens Scale Offset.")
        .modo("@ppi_PolyScale_Offset.py").ui(),
    action("Flatten Ngons", Modeling, "Flattens nonplanar polygons.")
        .modo("@ppi_flatten_ngons.py").undo(),
    action("Split", Modeling, "Connects selected vertex pairs with edges.")
        .modo("poly.split").undo(),
    action("Quadruple", Modeling, "Quadruple on the selected polygons.")
        .modo("poly.quadruple").undo(),
    action("Triple Fan", Modeling, "Splits the selected polygons into triangles fanned from the selected vertex.")
        .operandi().undo(),
    action("Collapse Selection", Modeling, "Replaces the selected polygons with triangles around one new point at the middle of their outline.")
        .operandi().undo(),
    action("Detriangulate", Modeling, "Opens Detriangulate.")
        .operandi().ui(),
    action("Relax Mesh", Modeling, "Opens Relax Mesh: evens out the selected points (whole mesh with nothing selected); open borders stay put.")
        .operandi().aliases(&["Relax Mesh..."]).ui(),
    script("sculpt.relaxMesh", Modeling, "Relax Mesh with these values and no dialog; bare, opens the dialog.")
        .args(&[
            opt("iter", INT, N, Some("5")),
            opt("constrain", BOOL, N, Some("false")),
        ])
        .example("sculpt.relaxMesh iter:5 constrain:true")
        .undo(),
    action("UV Unwrap", Modeling, "Opens UV Unwrap (uv.unwrap; a dialog until the UV viewport exists).")
        .operandi().ui(),
    action("UV View", Modeling, "Shows UV space in the active view, or 3D again (UV spec v1.1 §4).")
        .operandi().ui(),
    action("Open UV Editor", Modeling, "Texture > Open UV Editor: the UV Editor palette, the selected UV map in its own window; again closes it (UVT-200, LAY-54).")
        .operandi().ui(),
    action("Export UVs to SVG...", Modeling, "Texture > Export UVs to SVG...: the UV map's edges as an SVG drawing of the 0-1 tile (operandi.uv.export).")
        .operandi().ui(),
    action("Export UVs to EPS...", Modeling, "Texture > Export UVs to EPS...: the UV map's edges as an EPS drawing of the 0-1 tile (operandi.uv.export).")
        .operandi().ui(),
    action("Pack UVs", Modeling, "Opens Pack UVs.")
        .modo("uv.pack").ui(),
    action("Remesh", Modeling, "Opens Remesh: Voxel (sculpt.remesh) or Blocks, Smooth and Sharp (operandi.remesh).")
        .operandi().ui(),
    action("Automatic Retopology", Modeling, "Opens Automatic Retopology (bare mesh.retopo).")
        .modo("mesh.retopo").ui(),
    action("Boolean", Modeling, "Opens Boolean (bare poly.boolean).")
        .modo("poly.boolean").ui(),
    action("Solid Drill", Modeling, "Opens Solid Drill (bare poly.solidDrill).")
        .modo("poly.solidDrill").ui(),
    action("Axis Drill", Modeling, "Opens Axis Drill (bare poly.drill).")
        .modo("poly.drill").ui(),
    action("Spin Quad", Modeling, "Spins the shared edge of two polygons.")
        .modo("poly.spinQuads").undo(),
    action("Polygon Align", Modeling, "Makes the selected polygons face the same way.")
        .modo("poly.align").undo(),
    action("Triple", Modeling, "Triangulates the selected polygons.")
        .modo("poly.triple").undo(),
    action("Remove", Modeling, "Removes the selected polygons.")
        .modo("poly.remove").undo(),
    action("Polygon Merge", Modeling, "Merges the selected polygons.")
        .modo("poly.merge").undo(),
    action("Polygon Collapse", Modeling, "Collapses the selected polygons.")
        .modo("poly.collapse").undo(),
    action("Make", Modeling, "Builds a quad from two boundary edges.")
        .modo("poly.make auto").undo(),
    action("Assign Selection Set", Selection, "Opens Assign Selection Set.")
        .modo("select.editSet").ui(),
    action("Use Selection Set", Selection, "Opens Use Selection Set.")
        .modo("select.useSet").ui(),
    action("Delete Selection Set", Selection, "Opens Delete Selection Set.")
        .modo("select.deleteSet").ui(),
    action("Create Selection Set Group", Selection, "Opens Create Selection Set Group.")
        .modo("select.convertGroup").ui(),
    action("Toggle Symmetry", Modeling, "Turns Symmetry off, or back on with the last axis (X the first time).")
        .operandi().ui(),
    action("Symmetrize Positive", Modeling, "Copies the positive side over the negative.")
        .operandi().undo(),
    action("Symmetrize Negative", Modeling, "Copies the negative side over the positive.")
        .operandi().undo(),
    action("Symmetrize Polygons Positive", Modeling, "Deletes the negative half, mirrors the positive half across and merges the seam.")
        .operandi().undo(),
    action("Symmetrize Polygons Negative", Modeling, "Deletes the positive half, mirrors the negative half across and merges the seam.")
        .operandi().undo(),
    action("Edge Subdivide", Modeling, "Subdivides the selected edges (count and percent when given).")
        .modo("edge.subdivide")
        .args(&[opt("count", INT, N, None), opt("percent", FLOAT, N, None)])
        .undo(),
    action("Grow Quads", Modeling, "Grow Quads.")
        .modo("edge.growQuads").undo(),
    action("Fill Quads", Modeling, "Fill Quads.")
        .modo("edge.fillQuads").undo(),
    action("Edge Split", Modeling, "Splits along the selected edges.")
        .modo("edge.split").undo(),
    action("Split Normals", Modeling, "Splits normals along the selected edges.")
        .modo("edge.splitNormals").undo(),
    // Vertex Normal Tools (missing-tools-g9 VN-0..VN-10, vertex_normals.rs).
    action("Vertex Normal Tools...", Modeling, "Opens the Vertex Normal Tools palette: Smoothing, Tools and Cage (missing-tools-g9 VN-0).")
        .operandi().ui(),
    action("Recalculate Vertex Normals", Modeling, "Rebuilds the vertex normal map of the selected meshes (creating it if missing), smooth except across hard edges (VN-2).")
        .operandi().undo(),
    action("Harden Vertex Normals", Modeling, "Adds edges to the HardEdges set and rebuilds the normal map: mode vertices, edges, polygons, angle, uvAuto, uvEdge or uvPolygon (VN-3).")
        .operandi()
        .args(&[
            opt("mode", ArgType::Choice(&["vertices", "edges", "polygons", "angle", "uvAuto", "uvEdge", "uvPolygon"]), N, Some("edges")),
            opt("softenOthers", BOOL, N, Some("false")),
            opt("connected", BOOL, N, Some("false")),
        ])
        .example("harden vertex normals mode:angle")
        .undo(),
    action("Soften Vertex Normals", Modeling, "Takes edges out of the HardEdges set and rebuilds the normal map: mode vertices, edges, polygons, or all (VN-4).")
        .operandi()
        .args(&[
            opt("mode", ArgType::Choice(&["vertices", "edges", "polygons", "all"]), N, Some("all")),
            opt("connected", BOOL, N, Some("false")),
        ])
        .example("soften vertex normals mode:all")
        .undo(),
    action("Select Hard Edges", Modeling, "Selects the mesh's hard edges: mode set, add or remove (VN-5).")
        .operandi()
        .args(&[opt("mode", ArgType::Choice(&["set", "add", "remove"]), N, Some("set"))])
        .example("select hard edges")
        .undo(),
    action("Select Soft Edges", Modeling, "Selects the mesh's soft edges: mode set, add or remove (VN-5).")
        .operandi()
        .args(&[opt("mode", ArgType::Choice(&["set", "add", "remove"]), N, Some("set"))])
        .example("select soft edges")
        .undo(),
    action("Toggle Vertex Normal Maps", Modeling, "Auto removes the selected meshes' normal maps if they have any, else recreates one; on recreates; off removes (VN-7).")
        .operandi()
        .args(&[opt("mode", ArgType::Choice(&["auto", "on", "off"]), N, Some("auto"))])
        .example("toggle vertex normal maps")
        .undo(),
    action("Transfer Normals", Modeling, "Copies normals from the background mesh onto the active mesh at the nearest surface point (VN-8).")
        .operandi()
        .args(&[opt("world", BOOL, N, Some("true"))])
        .example("transfer normals")
        .undo(),
    action("Convert to Hard Edges", Modeling, "Every edge where shading splits joins the HardEdges set; removeGroups deletes the smoothing groups after (VN-9).")
        .operandi()
        .args(&[opt("removeGroups", BOOL, N, Some("false"))])
        .example("convert to hard edges")
        .undo(),
    action("Rename Normal Maps", Modeling, "Renames the selected meshes' normal maps to the custom name in Vertex Normal Tools options (VN-10).")
        .operandi()
        .args(&[opt("name", TEXT, N, None)])
        .example("rename normal maps name:Custom")
        .undo(),
    action("Spin Polygon Vertex Indices", Modeling, "Turns the stored point order of each selected polygon by offset (negative the other way); shape unchanged, corner data follows (VN-10).")
        .operandi()
        .args(&[opt("offset", INT, N, Some("1"))])
        .example("spin polygon vertex indices offset:1")
        .undo(),
    action("Morph to Normals", Modeling, "Each corner normal points along the morph map's offset at its point (VN-10).")
        .operandi()
        .args(&[opt("morph", TEXT, N, None)])
        .example("morph to normals morph:Morph")
        .undo(),
    action("Edge Spin", Modeling, "Spins the selected edges.")
        .modo("edge.spinQuads").undo(),
    action("Join", Modeling, "Joins the selected edges.")
        .modo("edge.join").undo(),
    action("Join Averaged", Modeling, "Joins the selected edges at their average.")
        .modo("edge.join average:true").undo(),
    action("Edge Collapse", Modeling, "Collapses the selected edges.")
        .modo("edge.collapse").undo(),
    action("Edge Remove", Modeling, "Removes the selected edges.")
        .modo("edge.remove").undo(),
    action("Edges To Curves", Modeling, "Makes curves from the selected edges.")
        .modo("edges.toCurve").undo(),
    action("Unbevel", Modeling, "Unbevels the selected edges.")
        .modo("edge.unbevel").undo(),
    action("New Vertex", Modeling, "Opens Create New Vertex.")
        .modo("vert.new").ui(),
    action("Show Vertices", ViewportOptions, "Toggles vertex display.")
        .modo("view3d.showVertices").ui(),
    action("Instancers", ViewportOptions, "Steps the view's Instancers option: None (every copy a box), Selected (the selected Instancer's copies shaded), All (missing-tools-g8 RP-3).")
        .aliases(&["Replicators"]).operandi().ui(),
    action("Vertex Split", Modeling, "Splits the selected points.")
        .modo("vert.split").undo(),
    action("Vertex Split Edge", Modeling, "Splits the selected points along edges.")
        .operandi().undo(),
    action("Vertex Set Position", Modeling, "Opens Set Vertex Position.")
        .modo("vert.set").ui(),
    action("Vertex Align", Modeling, "Opens Vertex Align.")
        .modo("vert.align").ui(),
    action("Vertex Merge", Modeling, "Opens Vertex Merge.")
        .modo("vert.merge").ui(),
    action("Vertex Join", Modeling, "Opens Vertex Join.")
        .modo("vert.join").ui(),
    action("Vertex Join Averaged", Modeling, "Joins the selected points at their average.")
        .modo("vert.join average:true").undo(),
    action("Vertex Collapse", Modeling, "Collapses the selected points.")
        .modo("vert.collapse").undo(),
    action("Vertex Remove", Modeling, "Removes the selected points.")
        .modo("vert.remove").undo(),
    action("Vertex Remove Edge", Modeling, "Removes the selected points and edges.")
        .operandi().undo(),
    action("Reorder Index", Modeling, "Opens Reorder Index.")
        .modo("vert.reorder").ui(),
    action("Sync Index Order", Modeling, "Syncs point order to polygon order.")
        .modo("vert.syncOrder").undo(),
    script("lock.sel", Modeling, "Locks the selected elements.").spelled("Lock Selected").undo(),
    script("lock.unsel", Modeling, "Locks the unselected elements.").spelled("Lock Unselected").undo(),
    script("lock.invert", Modeling, "Inverts which elements are locked.").spelled("Lock Invert").undo(),
    script("unlock", Modeling, "Unlocks every element.").undo(),
    // BRQ-110: Hide and Unhide run headless too (in the selection mode).
    script("hide.sel", Modeling, "Hides the selected elements.").spelled("Hide Selected")
        .headless("Hide the selected elements of the active mesh (everything when nothing is selected), one undo step; hidden polygons drop out of the UV tools.", &[])
        .undo(),
    script("hide.unsel", Modeling, "Hides the unselected elements.").spelled("Hide Unselected")
        .headless("Hide the elements of the active mesh that aren't selected, one undo step.", &[])
        .undo(),
    script("hide.invert", Modeling, "Inverts which elements are hidden.").spelled("Hide Invert")
        .headless("Swap which elements of the active mesh are hidden, one undo step.", &[])
        .undo(),
    script("unhide", Modeling, "Shows every hidden element.")
        .headless("Show every hidden element of the selected meshes (every mesh when none is selected), one undo step.", &[])
        .undo(),
];
