<!-- help-ids: edge.bridge.item, poly.loopSlice.item, poly.thicken.item, poly.setType.meshop.item, edge.slide.item, setvmap.subd.item, hardedge.meshop.item, hardedgeconvert.meshop.item, chamfer.edit.item, pmodel.partTag.item, Set Polygon Type, Hard Edge, Hard Edge Convert, Part Tag ; template: concept ; since: 0.1.2 -->
# Tool steps in the Mesh Stack

Many modeling tools can also be a step in a mesh's **Mesh Stack**, so you
can change their values later without rebuilding the mesh. A step runs
the same code as its tool, so the tool and the step give the same result
from the same values. Add one from the stack's **Add Operator** list. Its
values show in **Properties**, and its **Selection** section says which
part of the mesh it works on (see
[Mesh Stack selections](mesh-stack-selections.md)).

With nothing in its **Selection**, a step works on the whole mesh: every
polygon, or every edge for the edge steps.

## The steps

| Step | Folder | Works on | What it does |
|---|---|---|---|
| **Bridge** | Edit | Edges | Joins two edge runs or loops with new polygons, as [Bridge](../duplicate/bridge.md). With nothing selected it bridges the mesh's open borders, so two loose pieces with one open border each join up. |
| **Loop Slice** | Edit | Edges | Cuts loops across the ring of each chosen edge, as [Loop Slice](../edit/slice-tools.md). With a **Count** above 1 the loops are spaced evenly and **Position** is set aside. |
| **Hard Edge** | Edit | Edges | Marks the chosen edges hard (or, with **Hardness** set to **Soft**, soft) and rebuilds the vertex normals. **Mode: By Angle** takes only the chosen edges sharper than **Angle**. **Clear Other Hard Edges** softens the rest. |
| **Hard Edge Convert** | Edit | Whole mesh | Turns every edge where the shading splits into a hard edge. It can also remove the normal map and the smoothing groups. |
| **Edge Slide** | Edge | Edges | Slides the chosen edges along their neighboring edges, as [Slide](../edit/slide.md), by **Percent** or by **Distance**. |
| **Chamfer Edit** | Edge | Polygons | Reworks a chamfer or bevel made lower in the stack: pick its strip polygons, then change **Offset**, **Segments** and **Depth**. |
| **Thicken** | Polygon | Polygons | [Smooth Shift](../polygon/smooth-shift.md) with **Thicken** on: a sheet gets a back side and walls. **Offset** sets the thickness. |
| **Set Polygon Type** | Polygon | Polygons | Makes the chosen polygons plain faces, **Subdivision** polygons or **Catmull-Clark** polygons. It sets the type rather than switching it, so it gives the same result every time. |
| **Part Tag** | Polygon | Polygons | Gives the chosen polygons a part name, as [Set Part](../polygon/part-material.md). |
| **Set Edge Weight** | Vertex Map | Edges | Gives the chosen edges a subdivision weight, as [Edge Weight](../edge/edge-weight.md). **0%** removes their weights. |

A step that can't run (for example **Bridge** on a closed mesh with no
open border) turns its row red and passes the mesh below through
unchanged, so the rest of the stack still works.

## Related

- [Mesh Stack selections](mesh-stack-selections.md)
- [Properties panel](properties.md)
