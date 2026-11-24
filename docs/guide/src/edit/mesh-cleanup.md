<!-- help-ids: mesh.cleanup ; template: tool ; since: 0.1.0 -->
# Mesh Cleanup

**Mesh Cleanup** repairs common faults in a mesh, such as stray points, broken polygons and duplicates, in one step.

## How to use

1. Select the mesh. To clean only part of it, select those polygons in Polygon mode; with nothing selected, the whole mesh is cleaned.
2. Open **Geometry > Mesh Cleanup...**.
3. Check the repairs you want. All of them are on until you change them.
4. Click **OK**. **Cancel** closes the dialog and changes nothing.

The repairs run as one undo step, called **Mesh Cleanup**. If the mesh is already clean, nothing changes and no undo step is added. The dialog remembers your choices for next time.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Remove Floating Vertices** | Removes points that no polygon uses. Only when the whole mesh is cleaned. | On |
| **Remove One Point Polygons** | Removes polygons with a single point. | On |
| **Remove Two Points Polygons** | Removes polygons with two points. | On |
| **Fix Duplicate Points in Polygon** | Takes out a point that a polygon lists twice. | On |
| **Remove Colinear Vertices** | Removes a point that sits on a straight line between its neighbors, when only that polygon uses it. A polygon keeps at least three points. | On |
| **Fix Face Normal Vectors** | When a polygon's first corner lies on a straight line, starts its corner list at another corner so its normal comes out right. | On |
| **Merge Vertices** | Merges points that sit at exactly the same position. | On |
| **Merge Disco Values** | Not active yet: it changes nothing. | On |
| **Unify Polygons** | Removes a polygon that uses the same points as an earlier one, in any order or direction, when both have the same material. | On |
| **Force Unify** | Removes those duplicates even when their materials differ. Available only with **Unify Polygons** checked. | On |
| **Remove Disco Weight Values** | Not active yet: it changes nothing. | On |
| **Fix Gaps** | Where a point sits on a polygon's edge and its neighbors use it but that polygon doesn't, adds the point to the polygon. This closes the hairline gap. | On |

## Tips

- A selection in Vertex or Edge mode makes Mesh Cleanup do nothing. Switch to Polygon mode, or clear the selection, first.
- With every repair unchecked, Mesh Cleanup doesn't run and the status line says that no operations are selected.
- Finish or cancel the current tool, Pen drawing or Extrude stage before opening the dialog. A hidden mesh has to be shown first.
- Removed polygons leave the selection; the rest stay selected.

## Related

- [Geometry menu](../menus/geometry.md)
- [Clean-up commands](cleanup.md): Fix Black Dots, Edge Zip and Diagonal Re-Direct
- [Selection modes](../selection/selection-modes.md)
