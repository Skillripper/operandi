//! Edge and vertex commands (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- edges
    script("edge.addLoop", Modeling, "Adds a loop across each selected edge's polygon band.")
        .headless(
            "Add an edge loop across the edge a–b.",
            &[
            jreq("a", "integer", "Edge start vertex index."),
            jreq("b", "integer", "Edge end vertex index."),
            jreq("t", "number", "Position along a→b, 0–1."),
        ],
        )
        .args(&[opt("position", FLOAT, Unit::Percent, Some("0.5"))])
        .undo(),
    script("edge.addPoint", Modeling, "Splits each selected edge with a new point.")
        .headless(
            "Add a point on the edge a–b.",
            &[
            jreq("a", "integer", "Edge start vertex index."),
            jreq("b", "integer", "Edge end vertex index."),
            jreq("t", "number", "Position along a→b, 0–1."),
        ],
        )
        .args(&[opt("position", FLOAT, Unit::Percent, Some("0.5"))])
        .undo(),
    script("edge.growQuads", Modeling, "Grow Quads on the selected edges.")
        .headless(
            "Grow quads from the selected boundary edges.",
            &[],
        ).undo(),
    script("edge.fillQuads", Modeling, "Fill Quads on the selected edges.")
        .headless(
            "Fill the hole bounded by the selected edges with quads.",
            &[],
        ).undo(),
    script("edge.split", Modeling, "Splits the mesh along the selected edges.")
        .headless(
            "Split the mesh along the selected edges.",
            &[],
        ).undo(),
    script("edge.splitNormals", Modeling, "Splits normals along the selected edges.").undo(),
    script("edge.spinQuads", Modeling, "Spins the selected edges.")
        .headless(
            "Spin the selected edges.",
            &[],
        ).undo(),
    script("edge.join", Modeling, "Joins the selected edges (average:true averages them).")
        .headless(
            "Join each selected edge's ends.",
            &[jopt(
            "average",
            "boolean",
            "At their average position (true) or not (false, default).",
        )],
        )
        .args(&[opt("average", BOOL, N, Some("false"))])
        .undo(),
    script("edge.collapse", Modeling, "Collapses the selected edges.")
        .headless(
            "Collapse the selected edges.",
            &[],
        ).undo(),
    script("edge.remove", Modeling, "Removes the selected edges.")
        .headless(
            "Remove the selected edges.",
            &[],
        ).undo(),
    script("edges.toCurve", Modeling, "Makes curves from the selected edges.").undo(),
    script("edge.unbevel", Modeling, "Unbevels the selected edges.")
        .headless(
            "Undo a bevel on the selected edges.",
            &[],
        ).undo(),
    script("edge.subdivide", Modeling, "Subdivides the selected edges.")
        .headless(
            "Subdivide the selected edges.",
            &[
            jopt(
                "count",
                "integer",
                "New points per edge; omit for the plain one-cut Subdivide.",
            ),
            jopt(
                "percent",
                "number",
                "With count 1: where the point goes along the edge, 0–1 (0.5 = middle).",
            ),
        ],
        ).undo(),
    // ---------------------------------------------------------- vertices
    script("vert.new", Modeling, "Adds one point at a world position; bare opens Create New Vertex.")
        .headless(
            "Add a vertex to the active mesh.",
            &[
            jopt(
                "position",
                "[x, y, z]",
                "Item-space position, metres (or give x, y and z).",
            ),
            jopt("x", "number", "Metres, with y and z."),
            jopt("y", "number", "Metres."),
            jopt("z", "number", "Metres."),
        ],
        )
        .args(&[
            arg("x", FLOAT, Unit::Distance),
            arg("y", FLOAT, Unit::Distance),
            arg("z", FLOAT, Unit::Distance),
        ])
        .undo()
        .example("vert.new 0 0 0"),
    script("vert.split", Modeling, "Gives each polygon its own copy of the selected points.")
        .headless(
            "Split the selected vertices.",
            &[],
        ).undo(),
    script("vert.splitEdge", Modeling, "Splits the selected points along edges.")
        .headless(
            "Split the edges at the selected vertices.",
            &[],
        ).operandi()
        .undo(),
    script("vert.align", Modeling, "Aligns the selected points.")
        .headless(
            "Align the selected vertices.",
            &[
            jopt(
                "axes",
                "[bool, bool, bool]",
                "Which of X, Y, Z to align; default all.",
            ),
            jopt("uniform", "boolean", "Uniform spacing; default false."),
        ],
        )
        .args(&[
            opt("axis", AXES, N, Some("all")),
            opt("uniform", BOOL, N, None),
        ])
        .undo(),
    script("vert.merge", Modeling, "Merges the selected points (auto or by fixed distance).")
        .headless(
            "Merge selected vertices closer than a distance.",
            &[jopt(
            "dist",
            "number",
            "Metres; default 0 (coincident only).",
        )],
        )
        .args(&[
            opt("range", ArgType::Choice(&["auto", "fixed"]), N, Some("auto")),
            opt("dist", FLOAT, Unit::Distance, None),
            opt("keep", BOOL, N, Some("false")),
        ])
        .undo(),
    script("vert.join", Modeling, "Joins the selected points into one.")
        .headless(
            "Join the selected vertices into one.",
            &[jopt(
            "average",
            "boolean",
            "At their average position (true) or the first (false, default).",
        )],
        )
        .args(&[opt("average", BOOL, N, None)])
        .undo(),
    script("vert.collapse", Modeling, "Collapses the selected points.")
        .headless(
            "Collapse the selected vertices.",
            &[],
        ).undo(),
    script("vert.remove", Modeling, "Removes the selected points.")
        .headless(
            "Remove the selected vertices (and their polygons).",
            &[],
        ).undo(),
    script("vert.removeEdge", Modeling, "Removes the selected points and their edges.")
        .headless(
            "Remove the selected vertices, keeping polygons.",
            &[],
        ).operandi()
        .undo(),
    script("delete", Modeling, "Deletes the selection of the current mode: points take their polygons, edges dissolve, polygons go, leaving no unused points; items are deleted.")
        .headless(
            "Delete the selection of the current mode: points take their polygons, edges dissolve, polygons go; unused points go too; items are deleted (the Delete and Backspace keys).",
            &[],
        )
        .undo(),
    script("vert.reorder", Modeling, "Reorders point indices; bare opens the dialog.")
        .args(&[arg("mode", ArgType::Choice(&["swap", "top", "bottom"]), N)])
        .undo()
        .example("vert.reorder mode:swap"),
    script("vert.syncOrder", Modeling, "Syncs point order to polygon order.").undo(),
    script("vert.weld", Modeling, "Welds the selected points onto one point index.")
        .operandi()
        .headless(
            "Weld vertices onto a target vertex.",
            &[
            jreq("sources", "array of integers", "Vertex indices to weld."),
            jreq("target", "integer", "Vertex index they move to."),
        ],
        )
        .args(&[arg("target", INT, N)])
        .undo()
        .example("vert.weld target:0"),
];
