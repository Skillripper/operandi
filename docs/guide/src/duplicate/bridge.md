<!-- help-ids: Bridge, edge.bridge ; template: tool ; since: 0.1.0 -->
# Bridge

**Bridge** joins two separate selections with new polygons: two polygons or groups, or two border edges, runs or loops.

## How to use

1. Select two polygons (or groups of them) that share no edge, or two border
   edges, edge runs or loops. Both sides need the same number of edges.
2. Click **Bridge** on the **Duplicate** tool tab.
3. Click anywhere in the 3D view, or click **Apply** in **Tool Properties**.
   The bridge appears and Bridge stays on.
4. Change **Segments**, **Twist** or any other value. The bridge you just
   made changes as you go. You can also drag left or right in the view to
   add or remove segments.
5. Press **Q** to keep the bridge and turn Bridge off.

The bridge and all your changes to it are one undo step. **Cancel** removes
the bridge. While Bridge is on, clicks in the view don't select anything, so
select both sides first. To bridge another pair, press **Q**, select the next
pair, and pick **Bridge** again.

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
