//! Polygon commands: Pen, Part and the UV tools (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- polygon
    script("operandi.pen.create", Modeling, "Creates Pen geometry in one step from JSON positions.")
        .operandi()
        .aliases(&["pen"])
        .args(&[arg("json", ArgType::Json, N)])
        .undo()
        .example("operandi.pen.create {}"),
    script("poly.setPart", Modeling, "Assigns a part name to the selected polygons; bare opens the dialog.")
        .headless(
            "Set the Part tag of the selected polygons.",
            &[jreq("name", "string", "Part name.")],
        )
        .args(&[opt("name", TEXT, N, None)])
        .undo()
        .example("poly.setPart Part"),
    // UV names from the registry thread (2026-11-11). uv.unwrap is a tool
    // in the reference (`tool.set uv.unwrap on`); until Operandi has a UV
    // viewport it runs one-shot with the tool's attributes (UV-Q20).
    script("uv.unwrap", Modeling, "UV Unwrap of the visible polygons into the UV map (one-shot for now); unnamed attributes keep the last values.")
        .headless(
            "UV Unwrap of the visible polygons into the UV map (one-shot for now); unnamed attributes keep the last values.",
            &[
            jopt(
                "iter",
                "integer",
                "0 keeps the Initial Projection (default 100).",
            ),
            jopt("gaps", "number", "Relative Gap Size, 0-1 (default 0.2)."),
            jopt("matr", "boolean", "Segment by Material."),
            jopt("part", "boolean", "Segment by Part."),
            jopt(
                "project",
                "integer|string",
                "0 planar, 1 cylindrical, 2 spherical, 3 current, 4 groupNormal.",
            ),
            jopt("pinning", "integer|string", "0 u, 1 v."),
            jopt("axis", "integer|string", "Projection Axis: 0 x, 1 y, 2 z."),
            jopt("angle", "number", "Offset Angle in radians."),
            jopt("seal", "boolean", "Seal Holes."),
            jopt("layout", "boolean", "Update Layout."),
            jopt("mode", "integer|string", "0 conformal, 1 angleBased."),
            jopt(
                "symmetrize",
                "boolean",
                "Symmetrize (not built yet; must be false).",
            ),
            jopt("symmAxis", "integer|string", "0 x, 1 y, 2 z."),
            jopt("symmComp", "integer|string", "Mirror Axis: 0 u, 1 v."),
            jopt(
                "name",
                "string",
                "UV map; made when missing (default Texture).",
            ),
            jopt(
                "seam",
                "integer|string",
                "0 selectedEdges, 1 seamMap, 2 selectionSet.",
            ),
            jopt("seamName", "string", "Seam map name."),
            jopt("pickName", "string", "Edge selection set name."),
        ],
        )
        .args(&[
            opt("iter", INT, N, Some("100")),
            opt("gaps", FLOAT, N, Some("0.2")),
            opt("matr", BOOL, N, Some("false")),
            opt("part", BOOL, N, Some("false")),
            opt("project", ArgType::Choice(&["planar", "cylindrical", "spherical", "current", "groupNormal"]), N, Some("planar")),
            opt("pinning", ArgType::Choice(&["u", "v"]), N, Some("u")),
            opt("axis", ArgType::Choice(&["x", "y", "z"]), N, Some("y")),
            opt("angle", FLOAT, N, Some("0")),
            opt("seal", BOOL, N, Some("true")),
            opt("layout", BOOL, N, Some("true")),
            opt("mode", ArgType::Choice(&["conformal", "angleBased"]), N, Some("conformal")),
            opt("symmetrize", BOOL, N, Some("false")),
            opt("symmAxis", ArgType::Choice(&["x", "y", "z"]), N, Some("x")),
            opt("symmComp", ArgType::Choice(&["u", "v"]), N, Some("u")),
            opt("name", TEXT, N, Some("Texture")),
            opt("seam", ArgType::Choice(&["selectedEdges", "seamMap", "selectionSet"]), N, Some("selectedEdges")),
            opt("seamName", TEXT, N, None),
            opt("pickName", TEXT, N, None),
        ])
        .undo()
        .example("uv.unwrap mode:angleBased"),
    // BRQ-110: UV Relax's command form, the tool's Tool Properties as
    // arguments over its Reset values (UVS-1 §3; names from the
    // reference's uv.relax tool attributes).
    script("uv.relax", Modeling, "UV Relax of the UV map on the visible polygons, one undo step; unnamed attributes take the tool's Reset values.")
        .headless(
            "UV Relax of the active mesh's UV map on its visible polygons, one undo step. A polygon selection doesn't limit it; with noEdge false, islands holding a selected edge stay still. Unnamed attributes take the tool's Reset values; iter 0 (the Reset value) changes nothing.",
            &[
            jopt("mode", "integer|string", "0 spring, 1 conformal (default), 2 angleBased, 3 adaptive."),
            jopt("iter", "integer", "Iterations, the amount of relaxing (default 0: no change)."),
            jopt("seal", "boolean", "Seal Holes (Conformal and Angle Based only; default false)."),
            jopt("boundary", "integer|string", "0 smooth (default), 1 locked, 2 maintain, 3 straighten; not used in Spring mode."),
            jopt("area", "number", "Area Weight, 0-1 (Adaptive only; default 0.1)."),
            jopt("uniform", "boolean", "Uniform Spans (Straighten only; default false)."),
            jopt("noEdge", "boolean", "Ignore Edge Selection (default true); false holds islands with a selected edge still."),
            jopt("name", "string", "UV map (default: the selected one)."),
        ],
        )
        .args(&[
            opt("mode", ArgType::Choice(&["spring", "conformal", "angleBased", "adaptive"]), N, Some("conformal")),
            opt("iter", INT, N, Some("0")),
            opt("seal", BOOL, N, Some("false")),
            opt("boundary", ArgType::Choice(&["smooth", "locked", "maintain", "straighten"]), N, Some("smooth")),
            opt("area", FLOAT, N, Some("0.1")),
            opt("uniform", BOOL, N, Some("false")),
            opt("noEdge", BOOL, N, Some("true")),
            opt("name", TEXT, N, None),
        ])
        .undo()
        .example("uv.relax iter:10"),
    script("operandi.uv.atlas", Modeling, "Atlas Unwrap: an automatic UV layout of the visible polygons with no seams to mark (charts of polygons within angle: of each other, packed at one density). One undo step.")
        .operandi()
        .headless(
            "Atlas Unwrap of the active mesh's visible polygons into the UV map: charts of neighbouring polygons whose normals stay within angle of the chart's first, packed into 0-1 at one texel density. One undo step. Unnamed attributes keep the last values.",
            &[
            jopt("name", "string", "UV map; made when missing (default Texture)."),
            jopt("angle", "number", "Most a chart bends, radians (default 1.0472, 60°)."),
            jopt("gaps", "number", "Space around each chart in texels at resolution (default 4)."),
            jopt("resolution", "integer", "Texture size the gaps are measured at (default 1024)."),
        ],
        )
        .args(&[
            opt("name", TEXT, N, Some("Texture")),
            opt("angle", FLOAT, Unit::Angle, Some("1.0472")),
            opt("gaps", FLOAT, N, Some("4")),
            opt("resolution", INT, N, Some("1024")),
        ])
        .undo()
        .example("operandi.uv.atlas angle:0.7854"),
    script("uv.pack", Modeling, "Pack UVs of the UV map; only the selected polygons' islands move when polygons are selected.")
        .headless(
            "Pack UVs of the UV map; only the selected polygons' islands move when polygons are selected.",
            &[
            jopt("pack", "boolean", "Islands may move."),
            jopt(
                "stretch",
                "boolean",
                "Match each island's UV area to its 3D area first.",
            ),
            jopt("orient", "boolean", "Islands may rotate."),
            jopt("keepSize", "boolean", "Islands keep their size."),
            jopt(
                "direction",
                "integer|string",
                "0 horizontal, 1 vertical, 2 auto.",
            ),
            jopt("gaps", "number", "Gaps, 0-1 (default 0.2)."),
            jopt("byPixel", "boolean", "Set Gaps by Pixel."),
            jopt(
                "gapsByPixel",
                "number",
                "Pixel Size (gap in pixels of a 1024 texture).",
            ),
            jopt("bbox", "boolean", "Based on Bounding Box."),
            jopt("stack", "boolean", "Lock Stacked."),
            jopt(
                "region",
                "integer|string",
                "0 normalized, 1 nearest, 2 udim, 3 manual, 4 tiles, 5 polygonTag, 6 byItem.",
            ),
            jopt("udim", "integer", "Target UDIM tile (from 1001)."),
            jopt("regionX", "number", "Manual region corner U."),
            jopt("regionY", "number", "Manual region corner V."),
            jopt("regionW", "number", "Manual region width."),
            jopt("regionH", "number", "Manual region height."),
            jopt("tileU", "integer", "Tiles wide."),
            jopt("tileV", "integer", "Tiles high."),
            jopt(
                "polygonTag",
                "integer|string",
                "0 material, 1 part, 2 smoothingGroup, 3 selectionSet: the Polygon Tag region's groups.",
            ),
            jopt(
                "background",
                "boolean",
                "Background Constraint: other visible meshes' islands stay put and are packed round.",
            ),
            jopt(
                "writeNew",
                "boolean",
                "Pack UVs to New Map: keep the original map.",
            ),
            jopt("newName", "string", "The new map's name (with writeNew)."),
            jopt("bestrot", "boolean", "Find Best Box Rotation."),
        ],
        )
        .args(&[
            opt("pack", BOOL, N, Some("true")),
            opt("stretch", BOOL, N, Some("true")),
            opt("orient", BOOL, N, Some("false")),
            opt("keepSize", BOOL, N, Some("false")),
            opt("direction", ArgType::Choice(&["horizontal", "vertical", "auto"]), N, Some("auto")),
            opt("gaps", FLOAT, N, Some("0.2")),
            opt("byPixel", BOOL, N, Some("false")),
            opt("gapsByPixel", FLOAT, N, Some("10.24")),
            opt("bbox", BOOL, N, Some("false")),
            opt("stack", BOOL, N, Some("false")),
            opt("region", ArgType::Choice(&["normalized", "nearest", "udim", "manual", "tiles", "polygonTag", "byItem"]), N, Some("normalized")),
            opt("udim", INT, N, Some("1001")),
            opt("regionX", FLOAT, N, Some("0")),
            opt("regionY", FLOAT, N, Some("0")),
            opt("regionW", FLOAT, N, Some("1")),
            opt("regionH", FLOAT, N, Some("1")),
            opt("tileU", INT, N, Some("1")),
            opt("tileV", INT, N, Some("1")),
            opt("polygonTag", ArgType::Choice(&["material", "part", "smoothingGroup", "selectionSet"]), N, Some("material")),
            opt("background", BOOL, N, Some("false")),
            opt("writeNew", BOOL, N, Some("false")),
            opt("newName", TEXT, N, None),
            opt("bestrot", BOOL, N, Some("true")),
        ])
        .undo()
        .example("uv.pack orient:true"),
    // UV editing stage 1 (uv-editing-2026-11-16). The map has no names for
    // these yet: `operandi.*` placeholders (docs/NAMES-NEEDED.md).
    script("operandi.uvMap.select", Modeling, "Selects a UV map of the active mesh: the one the UV view, UV tools and Unwrap use.")
        .operandi()
        .headless(
            "Select a UV map of the active mesh (the UV view, the UV tools and Unwrap use it), one undo step.",
            &[jreq("name", "string", "The map's name.")],
        )
        .args(&[arg("name", TEXT, N)])
        .undo()
        .example("operandi.uvMap.select Iris"),
    script("operandi.uvMap.rename", Modeling, "Renames a UV map of the active mesh.")
        .operandi()
        .headless(
            "Rename a UV map of the active mesh, one undo step.",
            &[
                jreq("name", "string", "The map's name now."),
                jreq("newName", "string", "Its new name."),
            ],
        )
        .args(&[arg("name", TEXT, N), arg("newName", TEXT, N)])
        .undo()
        .example("operandi.uvMap.rename Texture Iris"),
    script("operandi.uvMap.delete", Modeling, "Deletes UV maps of the active mesh (names separated by commas).")
        .operandi()
        .headless(
            "Delete one or several UV maps of the active mesh in one undo step.",
            &[jreq("name", "string", "The map's name, or several separated by commas.")],
        )
        .args(&[arg("name", TEXT, N)])
        .undo()
        .example("operandi.uvMap.delete \"Texture 2,Texture 3\""),
    script("operandi.uvMap.copy", Modeling, "Copies a whole UV map of the active mesh to the UV map clipboard.")
        .operandi()
        .headless(
            "Copy a whole UV map's values to the UV map clipboard (no undo step).",
            &[jreq("name", "string", "The map's name.")],
        )
        .args(&[arg("name", TEXT, N)])
        .example("operandi.uvMap.copy Vest").side_effect(),
    script("operandi.uvMap.paste", Modeling, "Replaces a UV map's values with the copied map's.")
        .operandi()
        .headless(
            "Replace a UV map's values with the copied map's (same polygons needed), one undo step.",
            &[jreq("name", "string", "The map pasted into.")],
        )
        .args(&[arg("name", TEXT, N)])
        .undo()
        .example("operandi.uvMap.paste \"Vest 2\""),
    script("operandi.uv.fit", Modeling, "Fit UVs: scales and moves the selected UVs to fill a UDIM tile.")
        .operandi()
        .headless(
            "Fit UVs: scale and move the selected UVs (all visible ones with nothing selected) to fill a UDIM tile, one undo step. Unnamed attributes keep the last values.",
            &[
                jopt("fit", "string", "entire (default), island or polygon."),
                jopt("keepProportion", "boolean", "Scale uniformly (default true)."),
                jopt("gaps", "number", "Margin inside the tile, percent (default 0)."),
                jopt("gapsByPixel", "number", "The margin in pixels of a 1024 texture instead."),
                jopt("udim", "integer", "The tile (default 1001)."),
            ],
        )
        .args(&[
            opt("fit", ArgType::Choice(&["entire", "island", "polygon"]), N, Some("entire")),
            opt("keepProportion", BOOL, N, Some("true")),
            opt("gaps", FLOAT, N, Some("0")),
            opt("gapsByPixel", FLOAT, N, None),
            opt("udim", INT, N, Some("1001")),
        ])
        .undo()
        .example("operandi.uv.fit fit:island"),
    script("operandi.uv.flip", Modeling, "Flip UVs: mirrors the selected UVs about their own centre.")
        .operandi()
        .headless(
            "Flip U or V: mirror the selected UVs about their bounding-box centre, or each polygon about its own, one undo step.",
            &[
                jopt("axis", "string", "u (default, left to right) or v (top to bottom)."),
                jopt("perPolygon", "boolean", "Each polygon about its own centre."),
            ],
        )
        .args(&[
            opt("axis", ArgType::Choice(&["u", "v"]), N, Some("u")),
            opt("perPolygon", BOOL, N, Some("false")),
        ])
        .undo()
        .example("operandi.uv.flip axis:v"),
    // UV tab stage 3 (uv_ops.rs; BEHAVIOR-UV-TOOLS §7.5-7.9, §9, §11, §12).
    script("operandi.uv.quickAlign", Modeling, "Quick Align: turns each island so its first selected edge lies along U or V.")
        .operandi()
        .headless(
            "Quick Align (UVT-54): turn each island holding the selection so its first selected edge (or the line between its first two selected points) lies along an axis, one undo step.",
            &[jopt("mode", "string", "auto (default: the nearer axis), horizontal or vertical.")],
        )
        .args(&[opt("mode", ArgType::Choice(&["auto", "horizontal", "vertical"]), N, Some("auto"))])
        .undo()
        .example("operandi.uv.quickAlign mode:horizontal"),
    script("operandi.uv.islandAlign", Modeling, "Island Align: lines the selected islands up by their sides or centres.")
        .operandi()
        .headless(
            "Island Align (UVT-55): move each selected island so their left sides, right sides or centres (u) and bottoms, tops or middles (v) line up, one undo step.",
            &[
                jopt("u", "string", "min, max, center or unchanged (default)."),
                jopt("v", "string", "min, max, center or unchanged (default)."),
            ],
        )
        .args(&[
            opt("u", ArgType::Choice(&["min", "max", "center", "unchanged"]), N, Some("unchanged")),
            opt("v", ArgType::Choice(&["min", "max", "center", "unchanged"]), N, Some("unchanged")),
        ])
        .undo()
        .example("operandi.uv.islandAlign u:min"),
    script("operandi.uv.orient", Modeling, "Orient Pieces: turns each selected island to sit squarely.")
        .operandi()
        .headless(
            "Orient Pieces (UVT-56): turn each island in its smallest box (auto), or with its longer side along U (horizontal) or V (vertical), one undo step.",
            &[jopt("mode", "string", "auto (default), horizontal or vertical.")],
        )
        .args(&[opt("mode", ArgType::Choice(&["auto", "horizontal", "vertical"]), N, Some("auto"))])
        .undo()
        .example("operandi.uv.orient mode:horizontal"),
    script("operandi.uv.toOrigin", Modeling, "To Origin: moves the selected islands to a UDIM tile's corner.")
        .operandi()
        .headless(
            "To Origin (UVT-57): move each selected island so its lower-left corner sits on the tile's lower-left corner, one undo step.",
            &[jopt("udim", "integer", "The tile (default 1001).")],
        )
        .args(&[opt("udim", INT, N, Some("1001"))])
        .undo()
        .example("operandi.uv.toOrigin udim:1002"),
    script("operandi.uv.texel", Modeling, "Texel Density: scales the selected islands to a texel density.")
        .operandi()
        .headless(
            "Texel Density (UVT-58/59): density = Image Size x sqrt(UV area / 3D area). Apply individually or overall, or normalize, one undo step.",
            &[
                jopt("action", "string", "individually (default), overall or normalize."),
                jopt("imageSize", "number", "Texture size in pixels (the panel's value)."),
                jopt("density", "number", "Target pixels per metre (the panel's value)."),
                jopt("worldSpace", "boolean", "Item scale counts (the panel's value)."),
                jopt("upright", "boolean", "Turn islands upright first (Alt)."),
            ],
        )
        .args(&[
            opt("action", ArgType::Choice(&["individually", "overall", "normalize"]), N, Some("individually")),
            opt("imageSize", FLOAT, N, None),
            opt("density", FLOAT, N, None),
            opt("worldSpace", BOOL, N, None),
            opt("upright", BOOL, N, Some("false")),
        ])
        .undo()
        .example("operandi.uv.texel action:overall density:512"),
    script("operandi.uv.realWorldSize", Modeling, "Real World Size: one UV unit stands for a length.")
        .operandi()
        .headless(
            "Real World Size: scale the selected islands together so one UV unit covers size metres, one undo step.",
            &[
                jopt("size", "number", "Metres one UV unit stands for (default 1)."),
                jopt("worldSpace", "boolean", "Item scale counts (default true)."),
            ],
        )
        .args(&[opt("size", FLOAT, N, Some("1")), opt("worldSpace", BOOL, N, Some("true"))])
        .undo()
        .example("operandi.uv.realWorldSize size:2"),
    script("operandi.uv.split", Modeling, "Split UVs: splits the selected UVs from the rest.")
        .operandi()
        .headless(
            "Split UVs (UVT-60): split along the selected edges, round a polygon selection, or at selected points, the sides moved apart by distance, one undo step.",
            &[
                jopt("size", "string", "plain (default, invisibly apart), medium or large."),
                jopt("distance", "number", "A custom distance in UV units instead."),
            ],
        )
        .args(&[
            opt("size", ArgType::Choice(&["plain", "medium", "large"]), N, Some("plain")),
            opt("distance", FLOAT, N, None),
        ])
        .undo()
        .example("operandi.uv.split size:large"),
    script("operandi.uv.sew", Modeling, "Sew UVs: joins UV seams at the selected edges or points.")
        .operandi()
        .headless(
            "Sew UVs (UVT-61): join each selected seam's two sides into one UV position, one undo step.",
            &[
                jopt("connect", "string", "average (default), selected or unselected."),
                jopt("selectedOnly", "boolean", "Keep the unselected side still (Move Selected Only)."),
            ],
        )
        .args(&[
            opt("connect", ArgType::Choice(&["average", "selected", "unselected"]), N, Some("average")),
            opt("selectedOnly", BOOL, N, Some("false")),
        ])
        .undo()
        .example("operandi.uv.sew connect:selected"),
    script("operandi.uv.moveSew", Modeling, "Move and Sew: moves an island to the seam, then joins it.")
        .operandi()
        .headless(
            "Move and Sew (UVT-62): move (turn, and scale) the island across each selected seam so the seam lines up, then join it, one undo step.",
            &[
                jopt("connect", "string", "average (default), selected or unselected."),
                jopt("scale", "boolean", "Scale the island to fit (default true)."),
            ],
        )
        .args(&[
            opt("connect", ArgType::Choice(&["average", "selected", "unselected"]), N, Some("average")),
            opt("scale", BOOL, N, Some("true")),
        ])
        .undo()
        .example("operandi.uv.moveSew connect:selected"),
    script("operandi.uv.selectBorder", Modeling, "Select Border: selects every UV island's outline edges.")
        .operandi()
        .headless("Select UV Border (UVT-80): select the edges on every island's outline.", &[])
        .undo()
        .example("operandi.uv.selectBorder"),
    script("operandi.uv.selectErrors", Modeling, "Select Error UVs: selects polygons failing the ticked tests.")
        .operandi()
        .headless(
            "Select Error UVs (UVT-81): select the polygons that fail any ticked test.",
            &[
                jopt("cross", "boolean", "Overlapping another UV polygon."),
                jopt("self", "boolean", "Twisted (bow-tie) UV polygons."),
                jopt("flipped", "boolean", "Wound clockwise in UV."),
                jopt("stacked", "boolean", "Exactly on top of another."),
                jopt("udim", "boolean", "Crossing a UDIM tile border."),
                jopt("zero", "boolean", "Zero UV area."),
            ],
        )
        .args(&[
            opt("cross", BOOL, N, Some("true")),
            opt("self", BOOL, N, Some("true")),
            opt("flipped", BOOL, N, Some("true")),
            opt("stacked", BOOL, N, Some("true")),
            opt("udim", BOOL, N, Some("false")),
            opt("zero", BOOL, N, Some("true")),
        ])
        .undo()
        .example("operandi.uv.selectErrors flipped:true cross:false"),
    script("operandi.uv.selectUnmapped", Modeling, "Select Unmapped: selects polygons with no UVs in the map.")
        .operandi()
        .headless(
            "Select Unmapped (UVT-82): select the polygons with no UVs in the selected map, or the mapped ones with invert.",
            &[jopt("invert", "boolean", "Select the mapped polygons instead.")],
        )
        .args(&[opt("invert", BOOL, N, Some("false"))])
        .undo()
        .example("operandi.uv.selectUnmapped"),
    script("operandi.uv.copy", Modeling, "Copy UVs: puts the selected polygons' UVs on the UV clipboard.")
        .operandi()
        .headless("Copy UVs (UVT-90): the selected polygons' UVs onto the UV clipboard.", &[])
        .example("operandi.uv.copy").side_effect(),
    script("operandi.uv.cut", Modeling, "Cut UVs: copies the selected polygons' UVs, then removes them.")
        .operandi()
        .headless("Cut UVs (UVT-90): copy, then remove the selected polygons' UVs, one undo step.", &[])
        .undo()
        .example("operandi.uv.cut"),
    script("operandi.uv.paste", Modeling, "Paste UVs: writes the UV clipboard onto the selected polygons.")
        .operandi()
        .headless(
            "Paste UVs (UVT-91): write the clipboard onto the selected polygons, matched by topology or by selection order, one undo step.",
            &[jopt("mode", "string", "connectivity (default) or selection.")],
        )
        .args(&[opt("mode", ArgType::Choice(&["connectivity", "selection"]), N, Some("connectivity"))])
        .undo()
        .example("operandi.uv.paste mode:selection"),
    script("operandi.uv.delete", Modeling, "Delete UVs: removes the selected polygons' UVs from the map.")
        .operandi()
        .headless("Delete UVs (UVT-93): the selected polygons become unmapped, one undo step.", &[])
        .undo()
        .example("operandi.uv.delete"),
    script("operandi.uv.setValue", Modeling, "Set UV Value: sets U or V of the selected points.")
        .operandi()
        .headless(
            "Set UV Value (UVT-94): set U, V or both of the selected UVs, one undo step.",
            &[jopt("u", "number", "The U value."), jopt("v", "number", "The V value.")],
        )
        .args(&[opt("u", FLOAT, N, None), opt("v", FLOAT, N, None)])
        .undo()
        .example("operandi.uv.setValue u:0.5"),
    script("operandi.uv.rotatePolygons", Modeling, "Rotate Polygons: shifts each selected polygon's UVs one corner along.")
        .operandi()
        .headless("Rotate Polygon UVs (UVT-95): each selected polygon's corners take the next corner's UV, one undo step.", &[])
        .undo()
        .example("operandi.uv.rotatePolygons"),
    script("operandi.uv.mergeMaps", Modeling, "Merge UV Maps: merges maps into the first, deleting the others.")
        .operandi()
        .headless(
            "Merge UV Maps (UVT-96): fill the first map from the others where it has no UVs, then delete the others, one undo step.",
            &[jopt("names", "string", "The maps, comma separated, the target first.")],
        )
        .args(&[opt("names", TEXT, N, None)])
        .undo()
        .example("operandi.uv.mergeMaps names:Texture,Texture 2"),
    script("operandi.uv.seam", Modeling, "Cut Map: adds, clears, selects or deletes a UV seam map's edges; marker grows a Conditional Loop into it.")
        .operandi()
        .headless(
            "UV seam maps (UVT-100/101, UVS-1 §1.6): add the selected edges (all with none selected), clear them, select the map's edges, delete the map, or mark the Conditional Loop from the selected edges, one undo step.",
            &[
                jopt("action", "string", "add (default), clear, select, delete, marker, or use (pick the map the buttons use)."),
                jopt("name", "string", "The seam map (default: the one in use, UV Seam to start)."),
            ],
        )
        .args(&[
            opt("action", ArgType::Choice(&["add", "clear", "select", "delete", "marker", "use"]), N, Some("add")),
            opt("name", TEXT, N, None),
        ])
        .undo()
        .example("operandi.uv.seam action:add name:Body"),
    script("operandi.uv.symmetry", Modeling, "UV Symmetry: off, or mirror UV edits across a U or V line.")
        .operandi()
        .headless(
            "The UV Symmetry layer (UVT-70): in a UV view, an edit to a UV is copied, mirrored, to its partner across the line.",
            &[
                jopt("axis", "string", "off (default), u or v."),
                jopt("center", "number", "The line's position (default 0.5, the last used)."),
            ],
        )
        .args(&[
            opt("axis", ArgType::Choice(&["off", "u", "v"]), N, Some("off")),
            opt("center", FLOAT, N, None),
        ])
        .quiet()
        .example("operandi.uv.symmetry axis:u center:0.5"),
    script("operandi.uv.symmetryCenter", Modeling, "UV Symmetry Center: welds the UVs lying on the symmetry line.")
        .operandi()
        .headless("Center (UVT-70): UVs on the UV symmetry line land on it, and one point's UVs there become one, one undo step.", &[])
        .undo()
        .example("operandi.uv.symmetryCenter"),
    script("operandi.uv.symmetrize", Modeling, "Symmetrize UVs: copies UVs from 3D mirror polygons, mirrored across the UV line.")
        .operandi()
        .headless(
            "Symmetrize (UVT-71): each polygon on the target side gets its 3D mirror partner's UVs, mirrored across the UV symmetry line, one undo step.",
            &[
                jopt("axis", "string", "The 3D mirror axis: x (default), y or z."),
                jopt("positiveToNegative", "boolean", "Copy from the + side to the - side (default true)."),
                jopt("mirror", "string", "The UV axis mirrored: u (default) or v."),
            ],
        )
        .args(&[
            opt("axis", ArgType::Choice(&["x", "y", "z"]), N, Some("x")),
            opt("positiveToNegative", BOOL, N, Some("true")),
            opt("mirror", ArgType::Choice(&["u", "v"]), N, Some("u")),
        ])
        .undo()
        .example("operandi.uv.symmetrize axis:x mirror:u"),
    script("operandi.uv.selectTile", Modeling, "UDIM Indicator: selects the polygons whose UVs lie inside a tile.")
        .operandi()
        .headless(
            "Select UVs in the Region (UVT-113): select the polygons whose UVs lie wholly inside the UDIM tile.",
            &[jopt("udim", "integer", "The tile (default 1001).")],
        )
        .args(&[opt("udim", INT, N, Some("1001"))])
        .undo()
        .example("operandi.uv.selectTile udim:1002"),
    script("operandi.uv.nudgeTile", Modeling, "UDIM Indicator arrows: moves the selected islands whole tiles.")
        .operandi()
        .headless(
            "Move the selected islands whole tiles over (UVT-113 arrows), one undo step.",
            &[
                jopt("u", "integer", "Tiles right (negative left)."),
                jopt("v", "integer", "Tiles up (negative down)."),
            ],
        )
        .args(&[opt("u", INT, N, Some("0")), opt("v", INT, N, Some("0"))])
        .undo()
        .example("operandi.uv.nudgeTile u:1"),
    script("operandi.uv.export", Modeling, "Export UVs: the UV map's edges as an SVG or EPS drawing of the 0-1 tile.")
        .operandi()
        .headless(
            "Export UVs to SVG or EPS (UVT-120): the current map's edges, the 0-1 tile size units square, V up.",
            &[
                jopt("filename", "string", "Where to write it."),
                jopt("format", "string", "svg (default) or eps."),
                jopt("size", "number", "The drawing's width and height (default 1024)."),
            ],
        )
        .args(&[
            arg("filename", TEXT, N),
            opt("format", ArgType::Choice(&["svg", "eps"]), N, Some("svg")),
            opt("size", FLOAT, N, Some("1024")),
        ])
        .side_effect()
        .example("operandi.uv.export filename:layout.svg"),
    script("operandi.uv.toMesh", Modeling, "Convert UVs to Mesh: a flat mesh item shaped like the UV layout.")
        .operandi()
        .headless("Convert UVs to Mesh (UVT-120): a new mesh with a point per UV position (U to X, V to Y) and a polygon per mapped polygon, one undo step.", &[])
        .undo()
        .example("operandi.uv.toMesh"),
    script("operandi.uv.transfer", Modeling, "Transfer UVs: copies the primary mesh's UV map to the other selected meshes of the same topology.")
        .operandi()
        .headless("Transfer UVs (UVT-92): the primary active mesh's current map onto every other active mesh with the same polygons, one undo step.", &[])
        .undo()
        .example("operandi.uv.transfer"),
    script("operandi.uv.mirror", Modeling, "Mirror UVs across a line on U or V.")
        .operandi()
        .headless(
            "Mirror the selected UVs across the line axis = center, one undo step.",
            &[
                jopt("axis", "string", "u (default) or v."),
                jopt("center", "number", "The line's position (default 0.5, the last used)."),
            ],
        )
        .args(&[
            opt("axis", ArgType::Choice(&["u", "v"]), N, Some("u")),
            opt("center", FLOAT, N, Some("0.5")),
        ])
        .undo()
        .example("operandi.uv.mirror axis:u center:0.5"),
    script("operandi.uv.align", Modeling, "Align UVs: moves the selected UVs onto one line.")
        .operandi()
        .headless(
            "Align UVs: set the selected UVs' U (left, right) or V (up, down) to one value of their bounding box, one undo step.",
            &[
                jopt("direction", "string", "up, down, left or right."),
                jopt("alignTo", "string", "low, average, center or high (default the direction's far side)."),
                jopt("perRow", "boolean", "Each run of connected selected edges on its own."),
            ],
        )
        .args(&[
            opt("direction", ArgType::Choice(&["up", "down", "left", "right"]), N, Some("down")),
            opt("alignTo", ArgType::Choice(&["low", "average", "center", "high"]), N, None),
            opt("perRow", BOOL, N, Some("false")),
        ])
        .undo()
        .example("operandi.uv.align direction:down"),
    // Layouts stage 3 (UIC-210): the toolbox's E7 island buttons as one
    // command line each, so a form can hold them. `click` is the button's
    // modifier: shift and alt pick the rule's other groupings, options
    // opens its dialog first (uv_edit_ui.rs).
    // The UV tab's stage 3 dialogs (uv_ops_ui.rs): a button's options
    // first (UVT-08), then one run.
    script("operandi.uv.dialog", Modeling, "Opens a UV button's options: toOrigin, texel, split, sew, moveSew, errors, setValue, paste, quickAlign or orient.")
        .operandi()
        .args(&[arg("kind", ArgType::Choice(&["toOrigin", "texel", "split", "sew", "moveSew", "errors", "setValue", "paste", "quickAlign", "orient", "seamMap", "symmetrize", "selectTile"]), N)])
        .ui()
        .example("operandi.uv.dialog texel"),
    script("operandi.uv.unwrapSeams", Modeling, "Cut Map's Unwrap Tool (or tool:pelt, Unwrap & Relax) with the seam map in use as its seams.")
        .operandi()
        .args(&[opt("tool", ArgType::Choice(&["unwrap", "pelt"]), N, Some("unwrap"))])
        .ui()
        .example("operandi.uv.unwrapSeams pelt"),
    script("operandi.uv.brush", Modeling, "The Split and Sew brush in a UV view: split, sew or off; bare toggles it.")
        .operandi()
        .args(&[opt("mode", ArgType::Choice(&["split", "sew", "off"]), N, None)])
        .ui()
        .example("operandi.uv.brush sew"),
    script("operandi.uv.preset", Modeling, "Runs a UV island button (uv.fitUVs, uv.flipU, uv.alignUp ...) with the remembered values; click: shift or alt picks its other groupings, options opens its dialog.")
        .operandi()
        .args(&[
            arg("id", TEXT, N),
            opt("click", ArgType::Choice(&["plain", "shift", "alt", "options"]), N, Some("plain")),
        ])
        .undo()
        .example("operandi.uv.preset uv.fitUVs shift"),
];
