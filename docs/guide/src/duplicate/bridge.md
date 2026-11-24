<!-- help-ids: Bridge, edge.bridge ; template: tool ; since: 0.1.0 -->
# Bridge

**Bridge** joins two separate selections with new polygons: two polygons or groups, or two border edges, runs or loops.

## How to use

1. Select two polygons (or groups of them) that share no edge, or two border
   edges, edge runs or loops. Both sides need the same number of edges.
2. Click **Bridge** on the **Duplicate** tool tab.
3. Set the bridge in **Tool Properties**, then click **Apply** or **Drop**.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Segments** | How many rows of polygons across the bridge. | 0 |
| **Twist** | Turns the bridge along its length. | 0 |
| **Mode** | The bridge's path between the two sides. | Curve |
| **Remove Polygons** | Deletes the two selected polygons, so the bridge opens into them. | On |
| **Flip Polygons** | Turns the new polygons to face the other way. | Off |
| **Reverse Direction** | Joins the two sides the other way round. | Off |
| **Tension** | How far the curve bulges between the sides. | 100% |

**Reset Attributes** puts every value back to its starting value.

## Related

- [Duplicate tab](index.md): copies, instances, Instancers and Transform versions
- [Selection modes](../selection/selection-modes.md)
