<!-- help-ids: selop.create, selop.edit, selop.selection, Select Edges by Angle, Select Border, Select Triangles, Select Quads, Select N-gons, Select Polygons by Vertex Count, Select Polygons by Area, Select Edges by Length, Select Edges by UV Seam, Select Vertices by Edge Count, Select by Normal, Select by Plane, Select by Distance, Select N Randomly, Select Random Parts, Select Symmetric ; template: concept ; since: 0.1.2 -->
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
they keep up when the mesh below changes. Where a step has a **Minimum**
and a **Maximum**, a Maximum below the Minimum means "no limit".
Positions and sizes are in the mesh's own space.

| Name | What it picks | Default |
|---|---|---|
| **Select Triangles**, **Select Quads**, **Select N-gons** | Polygons with 3, 4, or 5 or more corners. | |
| **Select Polygons by Vertex Count** | Polygons with **Minimum** to **Maximum** corners. | 3 to 4 |
| **Select Polygons by Area** | Polygons whose area is that of a square with sides **Minimum** to **Maximum**, so the values read as lengths. | 0 m to 1 m |
| **Select Edges by Angle** | Edges whose two polygons meet between **Minimum** and **Maximum** (0° is flat). **Convex** and **Concave** choose outward or inward bends. | 30° to 180°, both |
| **Select Edges by Length** | Edges **Minimum** to **Maximum** long. | 0 m to 1 m |
| **Select Border** | Edges with a polygon on one side only. | |
| **Select Edges by UV Seam** | Edges where the two polygons' UVs in **UV Map** don't meet. | Texture |
| **Select Vertices by Edge Count** | Points with **Minimum Edges** to **Maximum Edges**. | 5 or more |
| **Select by Normal** | Elements facing **Minimum** to **Maximum** degrees from a **Direction**. An edge faces the way its points do on average, and a point the way its polygons do. | Up, 0° to 45° |
| **Select by Plane** | Elements whose centre is on one side of the plane through **Position** facing **Normal**. **Negative Side** picks the other side, and **Include Zero** counts elements on the plane. | Above Y 0 |
| **Select by Distance** | Elements whose centre is **Minimum** to **Maximum** from **From**. | 0 m to 1 m |
| **Select N Randomly** | **Number** elements at random, or **Percentage** of them when Number is 0. **Seed** gives another pick. | 10 |
| **Select Random Parts** | Whole connected parts, each with **Chance**. | 50% |
| **Select Symmetric** | The mirror image of the selection below it, across the checked axes together. Set **Blend Mode** to **Add** to keep both halves. | Reflect X |

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
