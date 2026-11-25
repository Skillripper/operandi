<!-- help-ids: Polygon Slice, Curve Slice, Edge Slice, Pen Slice, Loop Slice, Primitive Slice, Polygon Axis Slice ; template: tool ; since: 0.1.0 -->
# Slice tools

The slice tools cut new edges into a mesh: along a line, a curve, a loop round the mesh, or the outline of a shape.

![One loop added round a 2 m column with Loop Slice](../images/loop-slice.png)

## The tools

| Tool | Key | What it does |
|---|---|---|
| **Polygon Slice** | **Shift+C** | Cuts the selected polygons along a line you drag across them. |
| **Polygon Axis Slice** |  | Cuts the polygons along a plane square to an axis. |
| **Curve Slice** |  | Cuts along a curve you click out. |
| **Edge Slice** | **G** | Cuts a path through the edges, points and polygons you click. |
| **Pen Slice** |  | Cuts along a straight-line path you click out. |
| **Loop Slice** | **Alt+C** | Adds loops all the way round a ring of polygons. |
| **Primitive Slice** |  | Cuts along the outline of a shape (a rectangle, a circle) you draw. |

## Loop Slice

1. Select polygons (or edges) in the ring you want to cut across.
2. Press **Alt+C**, or click **Loop Slice**.
3. Set **Count** for how many loops, and drag in the view (or set
   **Position**) to place them.
4. Press **Q** to keep them.

### Loop Slice properties

| Name | What it does | Default |
|---|---|---|
| **Edit** | What a drag does to the loops: **Move** them, or other edits. | Move |
| **Mode** | How the loops are placed: **Free**, or evenly. | Free |
| **Count** | How many loops. | 1 |
| **Current** | Which loop the **Position** field edits. | 0 |
| **Uniform** | Spaces the loops evenly. |  |
| **Position** | Where the current loop crosses its polygons. | 50% |
| **Slice Selected** | Cuts only the selected polygons, not the whole ring. | Off |
| **Slice N-gon** | Also cuts through polygons with more than four sides. | Off |
| **Preserve Curvature** | Bends new points out to follow the surface's curve. | Off |

## Edge Slice

1. Press **G**, or click **Edge Slice**.
2. Click where the cut starts: on an edge, on a point, or inside a polygon.
3. Click the next place along the cut. It must be on a polygon next to the
   last one: one of its edges or corner points, or a spot inside it.
4. To close the loop, click the first point again. It turns the closing
   colour when the pointer is on it.
5. Press **Q** to keep the cut.

A click near a point uses that point, so the cut runs right through it.
Drag a point on an edge to slide it along the edge.

### Edge Slice properties

| Name | What it does | Default |
|---|---|---|
| **Split at Middle** | Puts every edge point at the middle of its edge. Hold **Ctrl** for one click. | Off |
| **Split Polygons** | Joins the points with new edges. Off, it only adds the points. | On |
| **Add Support Edges** | Splits any polygon the cut leaves with more than four sides into quads and triangles. It adds edges between existing corners, preferring ones that start at the cut's new points, and never adds points. | Off |
| **Show** | The readout while you place a point: **None**, **Position** or **Distance**. | Position |
| **Snap Value** | Rounds edge points to steps along the edge. | 0.5% |
| **Inside Polygons** | Lets you click inside a polygon to add a point there. | On |
| **Always Raycast** | Picks only what faces you, also in wireframe. | On |
| **Multi Slice** | Draws a dashed line from the last point to the pointer. | Off |

### Tips

- A cut never leaves a loose end inside a polygon. If you stop with an end
  inside one, that end runs on to the polygon's nearest corner.
- Turn on **Add Support Edges** to keep a mesh in quads for subdivision.
  The cut's neighbours that pick up a point on their edge are tidied too.

## Related

- [Edit tab](index.md)
- [Selection: loops and rings](../selection/loops.md)
