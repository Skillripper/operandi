//! Phrases that start tools (toolbox buttons) (one section of `COMMANDS`, in table order).
use super::*;

pub(super) const COMMANDS: &[Entry] = &[
    // ---------------------------------------------------------- tool starters
    action("Cube", Tools, "Starts the Cube tool.")
        .modo("tool.set prim.cube on").aliases(&["Box"]).tool(),
    action("Sphere", Tools, "Starts the Sphere tool.")
        .modo("tool.set prim.sphere on").tool(),
    action("Cylinder", Tools, "Starts the Cylinder tool.")
        .modo("tool.set prim.cylinder on").tool(),
    action("Cone", Tools, "Starts the Cone tool.")
        .modo("tool.set prim.cone on").tool(),
    action("Ellipsoid", Tools, "Starts the Ellipsoid tool.")
        .modo("tool.set prim.ellipsoid on").tool(),
    action("Ellipsoid Reset", Tools, "Resets the Ellipsoid tool.")
        .modo("tool.reset").undo(),
    action("Ellipsoid Unit", Modeling, "Adds a unit ellipsoid to the active mesh.")
        .operandi().undo(),
    action("Ellipsoid New Unit", Modeling, "Adds a unit ellipsoid as a new mesh.")
        .operandi().undo(),
    // Layouts stage 3 batch 7 (LAY-Q42): the Create tab's Shift / Ctrl
    // variants as command lines, named like Ellipsoid's.
    action("Cube Unit", Modeling, "Adds a 1 m cube to the active mesh at its origin, no drag (the Create tab's Ctrl-click).")
        .operandi().undo(),
    action("Cube New Unit", Modeling, "Adds a 1 m cube as a new mesh item at the scene origin, no drag (the Create tab's Shift-click).")
        .operandi().undo(),
    action("Sphere Unit", Modeling, "Adds a 1 m sphere to the active mesh at its origin, no drag (the Create tab's Ctrl-click).")
        .operandi().undo(),
    action("Sphere New Unit", Modeling, "Adds a 1 m sphere as a new mesh item at the scene origin, no drag (the Create tab's Shift-click).")
        .operandi().undo(),
    action("Cylinder Unit", Modeling, "Adds a 1 m cylinder to the active mesh at its origin, no drag (the Create tab's Ctrl-click).")
        .operandi().undo(),
    action("Cylinder New Unit", Modeling, "Adds a 1 m cylinder as a new mesh item at the scene origin, no drag (the Create tab's Shift-click).")
        .operandi().undo(),
    action("Cone Unit", Modeling, "Adds a 1 m cone to the active mesh at its origin, no drag (the Create tab's Ctrl-click).")
        .operandi().undo(),
    action("Cone New Unit", Modeling, "Adds a 1 m cone as a new mesh item at the scene origin, no drag (the Create tab's Shift-click).")
        .operandi().undo(),
    action("Torus Unit", Modeling, "Adds a 1 m torus to the active mesh at its origin, no drag (the Create tab's Ctrl-click).")
        .operandi().undo(),
    action("Torus New Unit", Modeling, "Adds a 1 m torus as a new mesh item at the scene origin, no drag (the Create tab's Shift-click).")
        .operandi().undo(),
    action("Capsule Unit", Modeling, "Adds a 1 m capsule to the active mesh at its origin, no drag (the Create tab's Ctrl-click).")
        .operandi().undo(),
    action("Capsule New Unit", Modeling, "Adds a 1 m capsule as a new mesh item at the scene origin, no drag (the Create tab's Shift-click).")
        .operandi().undo(),
    action("Torus", Tools, "Starts the Torus tool.")
        .modo("tool.set prim.toroid on").tool(),
    action("Capsule", Tools, "Starts the Capsule tool.")
        .modo("tool.set prim.capsule on").tool(),
    action("Arc", Tools, "Starts the Arc tool.")
        .modo("tool.set prim.arc on").tool(),
    action("Tube", Tools, "Starts the Tube tool.")
        .modo("tool.set prim.tube on").tool(),
    // Solid Sketch (spec solid-sketch v0.3): the button and menu run the
    // SS Default preset (Subdiv Mode Level 1); `tool.set solid.sketch on`
    // is the bare tool (Level 2).
    action("Solid Sketch", Tools, "Starts Solid Sketch with the SS Default preset: click to drop nodes joined into one closed, subdivided mesh.")
        .modo("tool.set \"SS Default\" on").tool(),
    action("Text", Tools, "Starts the Text tool.")
        .modo("tool.set prim.text on").tool(),
    action("Teapot", Tools, "Starts the Teapot tool: the Utah Teapot, sized by its longest side (missing-tools-g2 MT6).")
        .operandi().tool(),
    action("Teapot Item", Modeling, "Adds a 1 m Utah Teapot as a new mesh item at the scene origin, no drag (MT6-05).")
        .operandi().undo(),
    action("Plane Item", Modeling, "Adds a 1 m square plane facing up as a new mesh item at the scene origin, no drag (the Text button's Shift-click, MT5).")
        .operandi().undo(),
    action("Torus Reset", Tools, "Resets the active Torus, Capsule, Arc, Tube or Text tool.")
        .modo("tool.reset")
        .aliases(&["Capsule Reset", "Arc Reset", "Tube Reset", "Text Reset", "Teapot Reset"]).undo(),
    action("Bevel", Tools, "Starts Bevel.")
        .modo("tool.set poly.bevel on").tool(),
    // Twist Bevel (twist-bevel-2026-11-15): the spec's own id, a plain
    // modelling-tool name rather than an operandi.* one.
    action("Twist Bevel", Tools, "Starts Twist Bevel: bevel polygons out into a tube that can taper, twist and bend.")
        .modo("tool.set poly.twistBevel on").tool(),
    action("Chamfer Edit", Tools, "Starts Chamfer Edit.")
        .modo("tool.set chamfer.edit on").tool(),
    action("Poly Haul", Tools, "Starts Poly Haul.")
        .modo("tool.set poly.haul on").tool(),
    action("Sketch Extrude", Tools, "Starts Sketch Extrude.")
        .modo("tool.set poly.sweep on").tool(),
    action("Extrude", Tools, "Starts Extrude.")
        .modo("tool.set poly.extrude on").tool(),
    action("Smooth Shift", Tools, "Starts Smooth Shift; with an attribute and value, sets it on the live tool.")
        .modo("tool.set poly.smshift on")
        .aliases(&["poly.smshift"])
        .args(&[opt("attr", SMOOTH_SHIFT_ATTR, N, None), opt("value", TEXT, N, None)])
        .tool(),
    action("Thicken", Tools, "Starts Thicken; with an attribute and value, sets it on the live tool.")
        .modo("tool.set Thicken on")
        .args(&[opt("attr", SMOOTH_SHIFT_ATTR, N, None), opt("value", TEXT, N, None)])
        .tool(),
    action("Smooth Shift Reset", Tools, "Resets Smooth Shift or Thicken.")
        .modo("tool.reset").aliases(&["Thicken Reset"]).undo(),
    action("Smooth Shift Clone", Tools, "Clones the Smooth Shift result.")
        .operandi()
        .aliases(&["Thicken Clone"])
        .undo(),
    action("Spike", Tools, "Starts Spike.")
        .modo("tool.set poly.spikey on").tool(),
    action("Spike Reset", Tools, "Resets Spike.")
        .modo("tool.reset").undo(),
    action("Spike Strength", Tools, "Sets the live Spike's strength.")
        .operandi()
        .args(&[arg("value", FLOAT, Unit::Factor)])
        .example("spike strength 1").undo(),
    action("Bridge", Tools, "Starts Bridge; with an attribute and value, sets it on the live tool.")
        .modo("tool.set edge.bridge on")
        .args(&[opt("attr", BRIDGE_ATTR, N, None), opt("value", TEXT, N, None)])
        .tool(),
    action("Bridge Reset", Tools, "Resets Bridge.")
        .modo("tool.reset").undo(),
    action("Polygon Reduction", Tools, "Arms Polygon Reduction.")
        .modo("tool.set poly.reduct on").tool(),
    action("Polygon Pen", Tools, "Starts the interactive Polygon Pen.")
        .modo("tool.set prim.pen on")
        .aliases(&["Pen Interactive"])
        .tool(),
    action("Transform", Tools, "Starts Transform.")
        .operandi().tool(),
    action("Move", Tools, "Starts Move.")
        .operandi().tool(),
    action("Rotate", Tools, "Starts Rotate.")
        .operandi().tool(),
    action("Scale", Tools, "Starts Scale.")
        .operandi().tool(),
    action("Extend", Tools, "Starts Edge Extend.")
        .modo("tool.set edge.extend on").tool(),
    action("Edge Extrude", Tools, "Starts Edge Extrude.")
        .modo("tool.set edge.extrude on").tool(),
    action("Slide", Tools, "Starts Edge Slide.")
        .modo("tool.set edge.slide on").tool(),
    action("Edit Slide", Tools, "The Edit tab's Slide: Edge Slide in Edge mode, Vertex Slide in Vertex mode; unavailable in Polygon and Item mode (Polygon Slide isn't built).")
        .operandi().tool(),
    action("Edge Weight", Tools, "Starts Edge Weight.")
        .operandi().aliases(&["Edge Crease"]).tool(),
    action("Edge Bevel", Tools, "Starts Edge Bevel.")
        .modo("tool.set edge.bevel on").tool(),
    action("Edge Chamfer", Tools, "Starts Edge Chamfer.")
        .modo("tool.set edge.chamfer on").tool(),
    action("Edge Relax", Tools, "Starts Edge Relax.")
        .modo("tool.set edge.relax on").tool(),
    action("Create", Tools, "Starts the Create vertex tool.")
        .modo("tool.set prim.makeVertex on").tool(),
    action("Vertex Bevel", Tools, "Starts Vertex Bevel.")
        .modo("tool.set vert.bevel on").tool(),
    action("Vertex Extrude", Tools, "Starts Vertex Extrude.")
        .modo("tool.set vert.extrude on").tool(),
    action("Merge Tool", Tools, "Starts the vertex Merge tool.")
        .modo("tool.set vert.merge on").tool(),
    action("Tear", Tools, "Starts Tear.")
        .modo("@ppi_vert_tear.py").tool(),
    action("Vertex Slide", Tools, "Starts Vertex Slide.")
        .modo("tool.set vert.slide on").tool(),
    action("Retopo Edge Lock", Modeling, "Adds the selected guide curves to the RetopoEdgeLock selection set (TC-53).")
        .operandi().undo(),
    action("Retopo Edge Flow", Modeling, "Adds the selected guide curves to the RetopoEdgeFlow selection set (TC-53).")
        .operandi().undo(),
    action("Topology Pen", Tools, "Starts Topology Pen: drag an element to move it; Shift+drag an edge to pull out a new strip onto the background.")
        .modo("tool.set mesh.topology on").tool(),
    action("Tack Tool", Tools, "Turns the Tack Tool on or off: click a polygon to put the selection on it.")
        .operandi().ui(),
    action("Mesh Paint", Tools, "Turns Mesh Paint on or off: paint copies of a background mesh onto the selected mesh (missing-tools-g6).")
        .operandi().ui(),
    action("Mesh Paint Instance", Tools, "Turns Mesh Paint Instance on or off: Mesh Paint making instance items (missing-tools-g6).")
        .operandi().ui(),
    action("Guides", Tools, "Turns the Guides tool on or off: up to ten construction lines that draw in the views and that points snap to (missing-tools-g9 GD-0).")
        .operandi().ui(),
    action("Mesh Paint Instancer", Tools, "Turns Mesh Paint Instancer on or off: Mesh Paint making one Instancer per stroke (missing-tools-g8 RP-7).")
        .aliases(&["Mesh Paint Replica"]).operandi().ui(),
    action("Retopo Sketch", Tools, "Turns Retopo Sketch on or off: drag across a background mesh to draw a guide curve on it.")
        .operandi().ui(),
    action("Axis Rotate", Tools, "Starts Axis Rotate: Rotate about the work plane's normal only.")
        .operandi().tool(),
    action("Element Move", Tools, "Starts Element Move.")
        .modo("tool.set ElementMove on").aliases(&["Tweak"]).tool(),
    action("Weld", Tools, "Toggles drag-to-weld.")
        .modo("tool.set DragWeld on").tool(),
    action("Drop Weld", Tools, "Turns drag-to-weld off; unavailable while Weld is off.")
        .operandi().tool(),
    action("Add Loop", Tools, "Arms Add Loop, or disarms it: click an edge to add a loop (Ctrl: middle).")
        .operandi().tool(),
    action("Add Point", Tools, "Arms Add Point, or disarms it: click an edge to add a point (Ctrl: middle).")
        .operandi().tool(),
    action("Push", Tools, "Starts Push.")
        .modo("tool.set xfrm.push on").tool(),
    action("Smooth", Tools, "Starts Smooth.")
        .modo("tool.set xfrm.smooth on").tool(),
    action("Jitter", Tools, "Starts Jitter: moves each selected point by a seeded random offset within Range X/Y/Z.")
        .modo("tool.set xfrm.jitter on").tool(),
    action("Quantize", Tools, "Starts Quantize: snaps the selected points to a world grid of Step X/Y/Z.")
        .modo("tool.set xfrm.quantize on").tool(),
    action("Radial Align", Tools, "Starts Radial Align: puts the selected loop, points or polygon border on a circle, arc or regular polygon.")
        .modo("tool.set xfrm.radialAlign on").aliases(&["Radial"]).tool(),
    action("Linear Align", Tools, "Starts Linear Align: lines up each selected run between its two end points.")
        .modo("tool.set xfrm.linearAlign on").aliases(&["Linear"]).tool(),
    action("Soft Move", Tools, "Starts the Soft Move deform preset.")
        .modo("tool.set xfrm.magnet on").tool(),
    action("Shear", Tools, "Starts the Shear deform preset.")
        .modo("tool.set xfrm.shear on").tool(),
    action("Twist", Tools, "Starts the Twist deform preset.")
        .modo("tool.set xfrm.twist on").tool(),
    action("Swirl", Tools, "Starts the Swirl deform preset.")
        .modo("tool.set Swirl on").tool(),
    action("Taper", Tools, "Starts the Taper deform preset.")
        .modo("tool.set xfrm.taper on").tool(),
    action("Bulge", Tools, "Starts the Bulge deform preset.")
        .modo("tool.set xfrm.pole on").tool(),
    action("Flare", Tools, "Starts the Flare deform preset: Push with a Linear falloff fitted to the selection.")
        .modo("tool.set Flare on").tool(),
    action("Bend", Tools, "Starts Bend: curls the selection into an arc along its spine.")
        .modo("tool.set Bend on").tool(),
    action(
        "Symmetry Tool",
        Tools,
        "Starts the Symmetry Tool: drag to raise Threshold; points whose mirror partner is within it snap to match.",
    )
    .operandi()
    .tool(),
    action("Element Push", Tools, "Starts Element Push: Push with an Element falloff; drag the element under the pointer along its normal.")
        .modo("tool.set ElementPush on").tool(),
    action("Soft Drag", Tools, "Starts Soft Drag: Move with a Screen falloff around the pointer.")
        .modo("tool.set xfrm.softDrag on").tool(),
    action("Vortex", Tools, "Starts Vortex: Rotate with a Cylinder falloff; the middle turns most.")
        .modo("tool.set xfrm.vortex on").tool(),
    action("Soft Selection Move", Tools, "Starts Soft Selection Move: Move with a Soft Selection falloff about the selection.")
        .modo("tool.set SoftSelectionMove on").tool(),
    action("Soft Selection Rotate", Tools, "Starts Soft Selection Rotate: Rotate with a Soft Selection falloff about the selection.")
        .modo("tool.set SoftSelectionRotate on").tool(),
    action("Soft Selection Scale", Tools, "Starts Soft Selection Scale: Scale with a Soft Selection falloff about the selection.")
        .modo("tool.set SoftSelectionScale on").tool(),
    action("Soft Selection Transform", Tools, "Starts Soft Selection Transform: Transform with a Soft Selection falloff about the selection.")
        .modo("tool.set SoftSelectionTransform on").tool(),
    action("Flex", Tools, "Starts Flex: Transform with a Selection falloff about the selection's border.")
        .modo("tool.set Flex on").tool(),
    action("Loop Slice", Tools, "Starts Loop Slice.")
        .modo("tool.set poly.loopSlice on").tool(),
    action("Edge Slice", Tools, "Starts Edge Slice.")
        .modo("tool.set edge.knife on").tool(),
    action("Polygon Slice", Tools, "Starts Polygon Slice.")
        .modo("tool.set poly.knife on").tool(),
    action("Inset", Tools, "Starts Inset.")
        .modo("tool.set poly.inset on").tool(),
    action("Shift", Tools, "Starts Shift.")
        .modo("tool.set poly.shift on").tool(),
    action("Curve Slice", Tools, "Starts Curve Slice.")
        .modo("tool.set {Curve Slice} on").tool(),
    action("Pen Slice", Tools, "Starts Pen Slice.")
        .modo("tool.set PenKnife on").aliases(&["PenKnife"]).tool(),
    action("Primitive Slice", Tools, "Starts Primitive Slice.")
        .modo("tool.set {Primitive Slice} on").tool(),
    action("Contour", Tools, "Starts Contour: drag across a background mesh to lay a curve loop on it (missing-tools-g4).")
        .operandi()
        .tool(),
    action("Polygon Axis Slice", Tools, "Starts Polygon Axis Slice.")
        .modo("tool.set poly.julienne on")
        .aliases(&["Axis Slice"])
        .tool(),
    action("Mirror", Tools, "Starts Mirror.")
        .modo("tool.set *.mirror on").tool(),
    action("Radial Sweep", Tools, "Starts Radial Sweep.")
        .modo("tool.set {Radial Sweep} on").aliases(&["Lathe"]).tool(),
    action("Clone", Tools, "Starts Clone.")
        .modo("tool.set *.clone on").tool(),
    action("Array", Tools, "Starts Array.")
        .modo("tool.set *.array on").tool(),
    action("Radial Array", Tools, "Starts Radial Array.")
        .modo("tool.set {*.Radial Array} on").tool(),
    // The path tools' buttons (path_tools.rs, CX-01; ids provisional, CX-Q6).
    action("curve.extrude", Tools, "Starts the bare Curve Extrude tool (the buttons are its presets).")
        .operandi().tool(),
    action("curve.clone", Tools, "Starts the bare Curve Clone tool (the buttons are its presets).")
        .operandi().tool(),
    action("Symmetrize", Tools, "Starts Symmetrize: deletes one side of the mesh and mirrors the other side over it (DX-40).")
        .operandi().tool(),
    // The Duplicate tab's Instance buttons (DX-30, duplicate_items.rs).
    action("Instance Clone", Tools, "Starts Instance Clone: Mesh Instances of the selected items (DX-30).")
        .tool().operandi(),
    action("Instance Array", Tools, "Starts Instance Array: Mesh Instances of the selected items (DX-30).")
        .tool().operandi(),
    action("Instance Radial Array", Tools, "Starts Instance Radial Array: Mesh Instances of the selected items (DX-30).")
        .tool().operandi(),
    action("Instance Mirror", Tools, "Starts Instance Mirror: Mesh Instances of the selected items (DX-30).")
        .tool().operandi(),
    action("Instance Scatter", Tools, "Starts Instance Scatter: Mesh Instances of the selected items (DX-30).")
        .tool().operandi(),
    action("Curve Instance", Tools, "Starts Curve Instance: Mesh Instances of the selected items along a path (DX-30).")
        .tool().operandi(),
    action("Bezier Instance", Tools, "Starts Bezier Instance: Mesh Instances of the selected items along a path (DX-30).")
        .tool().operandi(),
    action("B-Spline Instance", Tools, "Starts B-Spline Instance: Mesh Instances of the selected items along a path (DX-30).")
        .tool().operandi(),
    action("Pen Instance", Tools, "Starts Pen Instance: Mesh Instances of the selected items along a path (DX-30).")
        .tool().operandi(),
    // The Duplicate tab's Replica buttons (missing-tools-g8 RP-6,
    // replicator.rs): the Instance buttons' placement, one Replicator.
    action("Instancer Clone", Tools, "Starts Instancer Clone: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["Replica Clone"]).tool().operandi(),
    action("Curve Instancer", Tools, "Starts Curve Instancer: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["Curve Replica"]).tool().operandi(),
    action("Bezier Instancer", Tools, "Starts Bezier Instancer: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["Bezier Replica"]).tool().operandi(),
    action("B-Spline Instancer", Tools, "Starts B-Spline Instancer: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["B-Spline Replica"]).tool().operandi(),
    action("Pen Instancer", Tools, "Starts Pen Instancer: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["Pen Replicate"]).tool().operandi(),
    action("Instancer Scatter", Tools, "Starts Instancer Scatter: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["Replica Scatter"]).tool().operandi(),
    action("Instancer Array", Tools, "Starts Instancer Array: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["Replica Array"]).tool().operandi(),
    action("Instancer Radial Array", Tools, "Starts Instancer Radial Array: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["Replica Radial Array"]).tool().operandi(),
    action("Instancer Mirror", Tools, "Starts Instancer Mirror: one Instancer of the selected items, a point per copy (missing-tools-g8 RP-6).")
        .aliases(&["Replica Mirror"]).tool().operandi(),
    action("Scatter Clone", Tools, "Starts Scatter Clone: copies scattered at random in a box (GO-01).")
        .modo("tool.set scatter.clone on").operandi().tool(),
    // The Duplicate tab's Transform buttons (DX-20): each moves the selected
    // items onto its placer's spots (duplicate_transform.rs).
    action("Transform Array", Tools, "Starts Transform Array: moves the selected items onto an Array grid's cells.")
        .modo("tool.set TransformArray on").tool().operandi(),
    action("Transform Radial Array", Tools, "Starts Transform Radial Array: moves the selected items round a circle.")
        .modo("tool.set TransformRadialArray on").tool().operandi(),
    action("Transform Scatter", Tools, "Starts Transform Scatter: moves the selected items onto scattered places.")
        .modo("tool.set TransformScatter on").tool().operandi(),
    action("Curve Transform", Tools, "Starts Curve Transform: moves the selected items along a smooth curve.")
        .modo("tool.set CurveTransform on").tool().operandi(),
    action("Bezier Transform", Tools, "Starts Bezier Transform: moves the selected items along a Bezier path.")
        .modo("tool.set BezierTransform on").tool().operandi(),
    action("B-Spline Transform", Tools, "Starts B-Spline Transform: moves the selected items along a B-Spline path.")
        .modo("tool.set BSplineTransform on").tool().operandi(),
    action("Transform Pen", Tools, "Starts Transform Pen: moves the selected items to the points of a straight-line path.")
        .modo("tool.set PenTransform on").tool().operandi(),
    // UV tab live tools (BEHAVIOR-UV-TOOLS §2-4, UVS-1 §1, §1.5, §3).
    action("UV Project", Tools, "Starts Project: writes the selected polygons' UVs from a Planar, Cylindrical, Spherical, Barycentric or Cubic projection (UVT-10..18).")
        .modo("tool.set operandi.uv.project on").operandi().tool(),
    action("Project from View", Tools, "Starts Project from View: a click in a 3D view writes the selected polygons' screen positions as UVs (UVT-19).")
        .modo("tool.set operandi.uv.viewProject on").operandi().tool(),
    action("Unwrap Tool", Tools, "Starts UV Unwrap as a live tool: unwraps every visible polygon along the seams; drag for Iterations.")
        .modo("tool.set uv.unwrap on").tool(),
    action("UV Relax", Tools, "Starts UV Relax: relaxes the map's islands to take out stretch; drag for Iterations.")
        .modo("tool.set uv.relax on").tool(),
    action("Unwrap & Relax", Tools, "Starts Unwrap & Relax: Unwrap, then Relax the result, in one tool.")
        .modo("tool.set uv.pelt on").aliases(&["Unwrap and Relax"]).tool(),
    action("Peeler", Tools, "Starts Peeler: lays clean quad strips out as a regular grid, cut at the selected edges (UVT-30).")
        .modo("tool.set operandi.uv.peeler on").operandi().tool(),
    action("Box Transform", Tools, "Starts Box Transform (UV view): move, turn and scale the selected UVs by a box round them (UVT-43).")
        .modo("tool.set operandi.uv.boxTransform on").operandi().tool(),
    action("Curve Extrude", Tools, "Starts Curve Extrude: extrude polygons along a smooth curve.")
        .modo("tool.set CurveExtrude on").tool().operandi(),
    action("Bezier Extrude", Tools, "Starts Bezier Extrude: extrude polygons along a Bezier path.")
        .modo("tool.set BezierExtrude on").tool().operandi(),
    action("B-Spline Extrude", Tools, "Starts B-Spline Extrude: extrude polygons along a B-Spline path.")
        .modo("tool.set BSplineExtrude on").tool().operandi(),
    action("Pen Extrude", Tools, "Starts Pen Extrude: extrude polygons along a straight-line path, one section per point.")
        .modo("tool.set PenExtrude on").tool().operandi(),
    action("Curve Clone", Tools, "Starts Curve Clone: copies along a smooth curve.")
        .modo("tool.set CurveClone on").tool().operandi(),
    action("Bezier Clone", Tools, "Starts Bezier Clone: copies along a Bezier path.")
        .modo("tool.set BezierClone on").tool().operandi(),
    action("B-Spline Clone", Tools, "Starts B-Spline Clone: copies along a B-Spline path.")
        .modo("tool.set BSplineClone on").tool().operandi(),
    action("Pen Clone", Tools, "Starts Pen Clone: copies at each point of a straight-line path.")
        .modo("tool.set PenClone on").tool().operandi(),
    action("Curve", Tools, "Starts the Curve draw tool.")
        .modo("tool.set prim.curve on").tool(),
    action("B-Spline", Tools, "Starts the B-Spline draw tool.")
        .modo("tool.set prim.bspline on").aliases(&["BSpline"]).tool(),
    action("Bezier Curve", Tools, "Starts the Bezier draw tool.")
        .modo("tool.set prim.bezier on").aliases(&["Bezier"]).tool(),
    action("Sketch", Tools, "Starts the Sketch draw tool.")
        .modo("tool.set prim.sketch on").tool(),
    action("Draw Patch", Tools, "Starts the Draw Patch tool.")
        .modo("tool.set patch.draw on").tool(),
    action("Quick Align", Tools, "Starts Quick Align.")
        .modo("tool.set QuickAlign on").tool(),
    action("Ground Align", Tools, "Starts Ground Align.")
        .modo("tool.set GroundAlign on").tool(),
    action("Ruler", Tools, "Starts the Ruler.")
        .modo("tool.set util.ruler on").tool(),
    action("Protractor", Tools, "Starts the Protractor.")
        .modo("tool.set util.protractor on").tool(),
    action("Dimensions", Tools, "Starts Dimensions.")
        .modo("tool.set util.bounds on").tool(),
    action("Paste Tool", Tools, "Toggles the Paste tool.")
        .modo("tool.set paste.tool on").tool(),
    action("Sculpt Smooth", Sculpt, "Starts the Smooth sculpt brush.")
        .modo("tool.set sculpt.smooth on").tool(),
    action("Sculpt Push", Sculpt, "Starts the Push sculpt brush.")
        .modo("tool.set sculpt.push on").tool(),
    action("Sculpt Inflate", Sculpt, "Starts the Inflate sculpt brush.")
        .modo("tool.set sculpt.inflate on").tool(),
    action("Sculpt Move", Sculpt, "Starts the Move sculpt brush.")
        .modo("tool.set sculpt.move on").tool(),
    action("Sculpt Fold", Sculpt, "Starts the Fold sculpt brush.")
        .modo("tool.set sculpt.fold on").tool(),
    action("Sculpt Clay", Sculpt, "Starts the Clay sculpt brush.")
        .modo("tool.set sculpt.clay on").tool(),
    action("Sculpt Smudge", Sculpt, "Starts the Smudge sculpt brush.")
        .modo("tool.set sculpt.smudge on").tool(),
    action("Sculpt Carve", Sculpt, "Starts the Carve sculpt brush.")
        .modo("tool.set sculpt.carve on").tool(),
    action("Sculpt Scar", Sculpt, "Starts the Scar sculpt brush.")
        .modo("tool.set sculpt.scar on").tool(),
    action("Sculpt Flatten", Sculpt, "Starts the Flatten sculpt brush.")
        .modo("tool.set sculpt.flatten on").tool(),
    action("Sculpt Tangent Pinch", Sculpt, "Starts the Tangent Pinch sculpt brush.")
        .modo("tool.set sculpt.tangentPinch on").tool(),
    action("Sculpt Center Pinch", Sculpt, "Starts the Center Pinch sculpt brush.")
        .modo("tool.set sculpt.centerPinch on").tool(),
    action("Sculpt Emboss", Sculpt, "Starts the Emboss sculpt brush.")
        .modo("tool.set sculpt.emboss on").tool(),
    action("Sculpt Attenuate", Sculpt, "Starts the Attenuate sculpt brush.")
        .modo("tool.set sculpt.shrink on").tool(),
    action("Sculpt Spin", Sculpt, "Starts the Spin sculpt brush.")
        .modo("tool.set sculpt.spin on").tool(),
    action("Vertex Map Paint", Sculpt, "Starts the Paint Vertex Map Tool.")
        .modo("tool.set sculpt.vmapPaint on").tool(),
    action("Vertex Map Smooth", Sculpt, "Starts the Smooth Vertex Map Tool.")
        .modo("tool.set sculpt.vmapSmooth on").tool(),
    action("Vertex Map Scale Down", Sculpt, "Starts the Scale Down Vertex Map Tool.")
        .modo("tool.set sculpt.vmapScale on").tool(),
    action("Vertex Map Erase", Sculpt, "Starts the Erase Vertex Map Tool.")
        .modo("tool.set sculpt.vmapErase on").tool(),
    action("Vertex Map Set Value", Sculpt, "Starts the Set Value Vertex Map Tool.")
        .modo("tool.set sculpt.vmapSetVal on").tool(),
];
