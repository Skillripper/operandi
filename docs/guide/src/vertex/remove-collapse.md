<!-- help-ids: Vertex Collapse, vert.collapse, Vertex Remove, vert.remove, Vertex Remove Edge, vert.removeEdge ; template: tool ; since: 0.1.0 -->
# Remove and Collapse

Three ways to get rid of points.

## The commands

| Button | Key | What it does |
|---|---|---|
| **Collapse** | **Control+Backspace** | Pulls each selected point and the points joined to it by an edge into one point at their average. The polygons around the selected point disappear; the polygons beyond them keep the merged point. |
| **Remove** | **Shift+Backspace** | Takes the selected points out of their polygons, which keep their other corners (a quad becomes a triangle). A polygon left with fewer than three corners goes. |
| **Remove Edge** |  | Removes the selected points and the edges through them. |

## Collapse on a corner

Collapse works on the point's neighborhood, not just the point. On a
cube corner, the corner and its three neighbors become one point, the
three polygons that met at the corner disappear, and the other three
lean in to meet at the new point, so the shape stays closed.

## Related

- [Vertex tab](index.md)
- [Selection modes](../selection/selection-modes.md)
- [Merge, Join and Weld](merge-join.md)
