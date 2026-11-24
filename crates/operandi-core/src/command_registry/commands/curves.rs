//! Curve tab commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // Curve tab commands (CURVE-COMMANDS-2D-SHAPES v1, curve_commands.rs).
    script("poly.makeCurveOpen", Modeling, "Makes an open Curve through the selected vertices, in selection order (CC-01).")
        .headless("Make an open Curve through the selected vertices in selection order; Polygon mode with the curve selected after.", &[])
        .undo().operandi(),
    script("poly.makeCurveClosed", Modeling, "Makes a closed Curve through the selected vertices, in selection order (CC-01).")
        .headless("Make a closed Curve through the selected vertices in selection order; Polygon mode with the curve selected after.", &[])
        .undo().operandi(),
    script("poly.makeCurveFill", Modeling, "Adds a flat Curve Fill polygon to each selected closed curve, keeping the curve (CC-10).")
        .headless("Add a Curve Fill polygon to each selected closed curve.", &[])
        .undo().operandi(),
    script("curve.winding", Modeling, "Makes the selected closed curves run the same way round (CC-20).")
        .headless(
            "Align the selected curves' winding: counter-clockwise seen from the automatic axis (clockwise with reverse).",
            &[
                jopt("merge", "boolean", "Join open curves that share end points first (default true)."),
                jopt("reverse", "boolean", "Wind the other way (default false)."),
            ],
        )
        .args(&[opt("merge", BOOL, N, Some("true")), opt("reverse", BOOL, N, Some("false"))])
        .undo().operandi(),
    // Batch 5 commands (generator_ops.rs; names provisional, GO-Q5).
    script("operandi.uv.rectangle", Modeling, "Straightens quad-grid UV islands into exact rectangles (GO-20).")
        .headless(
            "UV Rectangle on the selected polygons (all when none): each island that is a full grid of quads becomes a grid of rectangles.",
            &[
                jopt("name", "string", "UV map (default Texture)."),
                jopt("pack", "boolean", "Pack the islands into 0..1 after (default false)."),
                jopt("gaps", "number", "Pack gap, 0-1 (default 0.2)."),
                jopt("uniformX", "boolean", "Equal spans along U (default false)."),
                jopt("uniformY", "boolean", "Equal spans along V (default false)."),
                jopt("orient", "boolean", "Line each island up with U (default false)."),
            ],
        )
        .undo().operandi(),
    // Hands-on curve commands (curve_convert.rs, CC-40..64; names provisional).
    script("operandi.topology.tool", Tools, "Starts a Topology tab tool with Constrain to Background on (and snapping, for the drawing tools) (TB-11).")
        .args(&[opt("tool", TEXT, N, None)])
        .ui().operandi(),
    script("curve.boolean", Modeling, "Slices, unites, intersects or subtracts the selected closed curves with the other curves (CB-xx); bare opens its dialog.")
        .args(&[
            opt("operation", TEXT, N, Some("slice")),
            opt("axis", TEXT, N, Some("auto")),
            opt("clip", BOOL, N, Some("false")),
            opt("merge", BOOL, N, Some("true")),
        ])
        .undo().operandi(),
    script("curve.convert", Modeling, "Converts the selected curves to another curve type in a new item Converted_<type>, then edits it with that curve tool (CC-50).")
        .args(&[opt("from", TEXT, N, None), opt("to", TEXT, N, None)])
        .undo().operandi(),
    script("ppi.curveOffset", Modeling, "Adds an offset copy of the selected curves (CC-40); without offset: opens Enter Offset Amount.")
        .args(&[
            opt("closed", BOOL, N, Some("true")),
            opt("trio", BOOL, N, Some("false")),
            opt("offset", FLOAT, Unit::Distance, None),
        ])
        .undo().operandi(),
    script("bezier.tangent", Modeling, "Sharpens, smooths or evens the handles of the selected Bezier knots (CC-60); bare opens its dialog.")
        .args(&[
            opt("operation", TEXT, N, Some("sharp")),
            opt("tangent", TEXT, N, Some("both")),
            opt("setLength", BOOL, N, Some("false")),
            opt("length", FLOAT, Unit::Distance, Some("1")),
        ])
        .undo().operandi(),
    script("operandi.dicer", Modeling, "Smashes the active mesh into chunks with random planes (Dicer).")
        .headless(
            "Dicer: cut the active (closed) mesh with random planes into closed chunks; the same seed gives the same chunks.",
            &[
                jopt("seed", "integer", "Random Seed (default 40)."),
                jopt("planes", "integer", "Max. Cutting Planes, 1-16 (default 6)."),
                jopt("gap", "number", "Space between chunks (default 0.001)."),
                jopt("innerMaterial", "string", "Material of the new inside faces."),
                jopt("scaleX", "number", "Fragment Scale X (default 1)."),
                jopt("scaleY", "number", "Fragment Scale Y (default 1)."),
                jopt("scaleZ", "number", "Fragment Scale Z (default 1)."),
            ],
        )
        .undo().operandi(),
    script("operandi.vertMap.transfer", Modeling, "Copies a vertex map from the background mesh by sampling its surface (GO-30).")
        .headless(
            "Transfer Vertex Map from the first shown background mesh to the active mesh's selected points (all when none), sampling the source surface (values interpolate across its polygons).",
            &[
                jopt("source", "string", "The map's name (Position needs none)."),
                jopt("type", "string", "position, weight, uv, morph, rgb or rgba (default position)."),
                jopt("space", "string", "world (the items' placements count, default) or local (GO-33)."),
                jopt("method", "string", "distance (nearest spot, default) or raycast (along each point's normal; misses get no value, GO-34)."),
            ],
        )
        .undo().operandi(),
    script("curves.setColor", View, "Sets the display colour of curves in the viewports (CC-30): a view setting, not saved in the scene.")
        .args(&[opt("override", BOOL, N, Some("true")), opt("color", TEXT, N, None)])
        .ui().operandi(),
    script("vert.set", Modeling, "Gives every affected point one coordinate; bare opens Set Vertex Position.")
        .headless(
            "Set Position: give every selected point (or the points of the selected edges or polygons) one coordinate; one undo step.",
            &[
                jreq("axis", "string", "x, y or z."),
                jreq("value", "number", "Metres."),
                jopt("workPlane", "boolean", "Along the work plane's axis from its centre (default false)."),
                jopt("worldSpace", "boolean", "A world coordinate (default false)."),
            ],
        )
        .args(&[
            arg("axis", ArgType::Choice(&["x", "y", "z"]), N),
            arg("value", FLOAT, Unit::Distance),
            opt("workPlane", BOOL, N, Some("false")),
            opt("worldSpace", BOOL, N, Some("false")),
        ])
        .undo()
        .example("vert.set axis:y value:0"),
    script("poly.subdivide", Modeling, "Subdivides the selected polygons once; bare opens the options.")
        .headless(
            "Subdivide the selected polygons (all when none), one undo step (BRQ-10). Options not given take the Subdivide dialog's Reset values (Catmull-Clark, fractal 0, max angle 89.5°, Smooth All), not the last used.",
            &[
            jopt("type", "string", "Method: \"ccsds\" (Catmull-Clark, the default; the SDS Subdivide button), \"flat\" (Faceted), \"smooth\" or \"sds\" (the dialog's SDS Subdivide)."),
            jopt("fractal", "number", "Fractal, 0-1 (default 0)."),
            jopt("maxangle", "number", "Max Smoothing Angle, radians (default 89.5°)."),
            jopt("linear", "boolean", "Linear UVs (default false)."),
            jopt("bound", "string", "Boundary rule (Catmull-Clark): \"smooth\" (Smooth All, default), \"crease\" (Crease All) or \"creaseEdges\"."),
            jopt("noEdgeWeight", "boolean", "Ignore edge weights (default false)."),
            jopt("iterations", "integer", "How many times to subdivide, 1-8 (default 1; an Operandi extra: the button's repeat count)."),
        ],
        )
        .args(&[
            arg("type", ArgType::Choice(&["ccsds", "flat", "smooth", "sds"]), N),
            opt("fractal", FLOAT, Unit::Percent, None),
            opt("maxangle", FLOAT, Unit::Angle, None),
            opt("linear", BOOL, N, None),
            opt("bound", ArgType::Choice(&["smooth", "crease", "creaseEdges"]), N, None),
            opt("noEdgeWeight", BOOL, N, None),
        ])
        .undo()
        .example("poly.subdivide flat"),
    script("poly.unsubdiv", Modeling, "Merges groups of four quads back into one; bare opens the dialog.")
        .headless(
            "UnSubdivide: merge groups of four quads back into one, one undo step (BRQ-10). Options not given take the dialog's Reset values (iteration 1, uv and material hints on).",
            &[
            jopt("iteration", "integer", "Iterations, 0-16 (default 1)."),
            jopt("flat", "boolean", "Flat (default false)."),
            jopt("uv", "boolean", "UV hint (default true)."),
            jopt("material", "boolean", "Material hint (default true)."),
            jopt("part", "boolean", "Part hint (default false)."),
            jopt("smoothingGroup", "boolean", "Smoothing group hint (default false)."),
            jopt("selectionSet", "boolean", "Selection set hint (default false)."),
            jopt("selectionSetName", "string", "The selection set the hint reads."),
        ],
        )
        .args(&[
            opt("iteration", INT, N, None),
            opt("flat", BOOL, N, None),
            opt("uv", BOOL, N, None),
            opt("material", BOOL, N, None),
            opt("part", BOOL, N, None),
            opt("smoothingGroup", BOOL, N, None),
            opt("selectionSet", BOOL, N, None),
            opt("selectionSetName", TEXT, N, None),
        ])
        .undo()
        .example("poly.unsubdiv iteration:1"),
    script("tool.handleStyle", Tools, "Sets how tool handles draw: invisible, basic, advanced or default (basic).")
        .args(&[arg("style", ArgType::Choice(&["invisible", "basic", "advanced", "default"]), N)])
        .ui()
        .example("tool.handleStyle advanced"),
    script("scale.absolute", Modeling, "Absolute Scaling: Grab Size (mode:grab) reads the selection's size; uniform, explicit and reference scale it to typed sizes. Values left out come from the Absolute Scaling window.")
        .headless(
            "Absolute Scaling: scale the selection (or the active mesh) to typed sizes, one undo step. mode grab only reads: it answers the selection's size and changes nothing. Values not given take the Absolute Scaling window's defaults (0 sizes leave that axis alone, centre Center).",
            &[
            jreq(
                "mode",
                "string",
                "\"grab\", \"uniform\", \"explicit\" or \"reference\".",
            ),
            jopt(
                "absolute_size_uniform",
                "number",
                "uniform: the length, metres, the Uniform Axis becomes.",
            ),
            jopt(
                "absolute_size_uniform_pref",
                "string",
                "Uniform Axis: \"longestAxis\" (default), \"x\", \"y\" or \"z\".",
            ),
            jopt(
                "orient_to_selection",
                "boolean",
                "Measure along the selection's own sides; default false.",
            ),
            jopt(
                "absolute_size_x",
                "number",
                "explicit / reference: X size, metres (0 leaves X alone).",
            ),
            jopt(
                "absolute_size_y",
                "number",
                "explicit / reference: Y size, metres (0 leaves Y alone).",
            ),
            jopt(
                "absolute_size_z",
                "number",
                "explicit / reference: Z size, metres (0 leaves Z alone).",
            ),
            jopt(
                "absolute_size_center_x",
                "string",
                "What stays put along X: \"center\" (default), \"high\", \"low\" or \"origin\" (the item's origin).",
            ),
            jopt(
                "absolute_size_center_y",
                "string",
                "Along Y: \"center\", \"high\", \"low\" or \"origin\".",
            ),
            jopt(
                "absolute_size_center_z",
                "string",
                "Along Z: \"center\", \"high\", \"low\" or \"origin\".",
            ),
        ],
        )
        .args(&[
            arg("mode", ArgType::Choice(&["grab", "uniform", "explicit", "reference"]), N),
            opt("absolute_size_uniform", FLOAT, Unit::Distance, None),
            opt("absolute_size_uniform_pref", ArgType::Choice(&["longestAxis", "x", "y", "z"]), N, None),
            opt("orient_to_selection", BOOL, N, None),
            opt("absolute_size_x", FLOAT, Unit::Distance, None),
            opt("absolute_size_y", FLOAT, Unit::Distance, None),
            opt("absolute_size_z", FLOAT, Unit::Distance, None),
            opt("absolute_size_center_x", ArgType::Choice(&["center", "high", "low", "origin"]), N, None),
            opt("absolute_size_center_y", ArgType::Choice(&["center", "high", "low", "origin"]), N, None),
            opt("absolute_size_center_z", ArgType::Choice(&["center", "high", "low", "origin"]), N, None),
        ])
        .undo()
        .example("scale.absolute mode:explicit absolute_size_x:2"),
    script("vert.center", Modeling, "Centres the selection (or mesh) on the origin along the given axes.")
        .headless(
            "Center: move the selected points, edges or polygons (the whole mesh when none) so their world bounding-box centre is 0 on the given axes, one undo step (BRQ-10).",
            &[jreq("axis", "string", "\"all\", \"x\", \"y\", \"z\", \"xy\", \"yz\" or \"zx\".")],
        )
        .args(&[arg("axis", AXES, N)])
        .undo()
        .example("vert.center all"),
    script("poly.freeze", Modeling, "Freezes curves and subdivision surfaces into plain geometry; bare opens the Freeze dialog. face 0 = Faces, 1 = Polylines, 2 = 2 Point Polygons (2 is Operandi-only); drill, axis, disp, deform, morphSave and morphName are accepted and ignored.")
        .headless(
            "Freeze curves and subdivision surfaces of the active mesh into plain polygons (the selected polygons in Polygon mode, else all), one undo step. Arguments not given keep the last Freeze's values (defaults: faces, tess true, fixed true, 5°). Never opens the dialog here.",
            &[
            jopt(
                "face",
                "string",
                "Curves become 0 / \"faces\", 1 / \"polylines\" or 2 / \"twoPointPolygons\" (2 is Operandi's).",
            ),
            jopt(
                "tess",
                "boolean",
                "Freeze subdivision surfaces too (Tessellation); default true.",
            ),
            jopt(
                "fixed",
                "boolean",
                "Fixed spans per curve segment; false uses cangle instead.",
            ),
            jopt(
                "cangle",
                "number",
                "Curve Refinement Angle, radians (above 0, at most π), used when fixed is false.",
            ),
            jopt("drill", "boolean", "Accepted and ignored (not built yet)."),
            jopt("axis", "integer", "Accepted and ignored (not built yet)."),
            jopt("disp", "boolean", "Accepted and ignored (not built yet)."),
            jopt("deform", "boolean", "Accepted and ignored (not built yet)."),
            jopt(
                "morphSave",
                "boolean",
                "Accepted and ignored (not built yet).",
            ),
            jopt(
                "morphName",
                "string",
                "Accepted and ignored (not built yet).",
            ),
        ],
        )
        .args(&[
            // 0 and 1 are the reference's values; 2 (twoPointPolygons) is
            // an Operandi-only addition (registry thread, 2026-11-11).
            opt("face", ArgType::Choice(&["faces", "polylines", "twoPointPolygons"]), N, None),
            opt("drill", BOOL, N, None),
            opt("axis", INT, N, None),
            opt("disp", BOOL, N, None),
            opt("deform", BOOL, N, None),
            opt("tess", BOOL, N, None),
            opt("fixed", BOOL, N, None),
            opt("cangle", FLOAT, Unit::Angle, None),
            opt("morphSave", BOOL, N, None),
            opt("morphName", TEXT, N, None),
        ])
        .undo()
        .example("poly.freeze face:2 fixed:false cangle:5deg"),
];
