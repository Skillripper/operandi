<!-- help-ids: selop.create, selop.edit, selop.selection, Select Edges by Angle, Select Border, Select by Polygon Sides, Select by Facing ; template: concept ; since: 0.1.2 -->
# Mesh Stack selections

A step in a mesh's **Mesh Stack** can work on part of the mesh instead of
all of it. Its **Selection** section says which part. Each step picks the
kind of element it works on: the **Edge** steps (such as **Edge Bevel**)
pick edges, the **Vertex** steps pick points, and every other step picks
polygons. **Select Randomly** on an **Edge Bevel**, for example, bevels
random edges.

## Picking by number

- With the mesh's elements selected in the matching mode (edges in
  **Edges** mode for an Edge step), adding the step captures them as its
  **Select by Index**. **Set**, **Add** and **Remove** change it later.
- **Select by Index**, **Select by Range**, **Select by Pattern** and
  **Select Randomly** count the step's own kind.

Numbers name elements of the mesh as it was. If a step lower in the
stack adds or removes elements, the numbers may land on other ones. The
step's row then says so. It still runs.

## Picking by rule

These steps work their selection out again every time the stack runs, so
they keep up when the mesh below changes:

| Name | What it picks | Default |
|---|---|---|
| **Select Edges by Angle** | Edges whose two polygons meet between **Min Angle** and **Max Angle** (0° is flat). | 30° to 180° |
| **Select Border** | Edges with a polygon on one side only. | |
| **Select by Polygon Sides** | Triangles, quads or n-gons. | Quads |
| **Select by Facing** | Polygons facing within **Angle** of a **Direction**. | Up (Y), 45° |

A rule picks in its own kind and hands it over in the step's kind. An
edge is picked when one of its polygons is, a polygon when one of its
edges is, and a point when it lies on a picked polygon or edge.
**Select by Previous Operation** and **Select by Selection Set** work the
same way. A Selection Set of the step's own kind (an edge set for an
Edge step) is used first.

## Order

Selection steps apply from the bottom up, each by its **Blend Mode**. For
**Subtract** and **Multiply** the order changes the result. Use
**Move Up** and **Move Down** to reorder them. Scripts use
`selop.edit index:<n> position:<m>`.

## Related

- [Properties panel](properties.md)
- [Edge Bevel](../edge/edge-bevel.md)
