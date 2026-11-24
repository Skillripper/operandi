<!-- help-ids: Twist Bevel ; template: tool ; since: 0.1.0 -->
# Twist Bevel

**Twist Bevel** bevels polygons out into a tube that can taper, twist and bend along its length.

![The top of a 1 m cube beveled out 0.8 m in 6 segments, turned 90 degrees](../images/twist-bevel.png)

## How to use

1. Press **3** for Polygons and select the polygons.
2. Click **Twist Bevel** on the **Polygon** tool tab.
3. Drag anywhere for **Shift**, or use the handles: the **Shift** arrow, the
   **Inset** and twist ring at the cap, the bend arrow and bend direction ring
   at the base. **Shift**-click starts a new bevel on the caps.
4. Press **Q**, or click **Drop Tool**, to keep the result. **Cancel** takes it back.

With nothing selected, every polygon is bevelled.

## Properties

| Name | What it does |
|---|---|
| **Shift** | How far the tube reaches out. |
| **Inset** | How much the end of the tube shrinks. |
| **Segments** | How many rows along the tube. |
| **Group Polygons** | Bevels touching polygons together as one. |
| **Rotation**, **Rotation Axis** | How far the tube twists along its length, and about which axis (**Normal** by default). |
| **Profile** | Shapes the tube's outline along its length. |
| **Segment Spacing**, **Center Bias** | Bunches the rows toward one end or the middle. |
| **Bend**, **Bend Direction** | Curves the tube, and which way. |

## Related

- [Polygon tab](index.md)
- [Selection modes](../selection/selection-modes.md)
- [Bevel](../modeling/polygon-bevel.md)
