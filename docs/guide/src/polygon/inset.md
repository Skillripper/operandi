<!-- help-ids: Inset, Shift ; template: tool ; since: 0.1.0 -->
# Inset and Shift

**Inset** gives each selected polygon a smaller copy inside it, joined to
its outline by a ring of new side polygons. **Shift** does the same with
the copy pushed out along the polygon's normal instead.

Both are polygon tools on the **Polygon** tool tab. Each polygon is worked
on its own; for polygons that move as one piece, use
[Bevel](../modeling/polygon-bevel.md) with **Group Polygons** on.

## How to use

1. Press **3** to switch to Polygon mode.
2. Click the polygons you want. With nothing selected, the tool works on
   every polygon of the mesh.
3. Click **Inset** (or **Shift**) on the **Polygon** tool tab.
4. Drag the handle, or type a value in **Tool Properties**.
5. Press **Q** to drop the tool. The whole session is one undo step.

## Handles

| Handle | Drag it to |
|---|---|
| Inset's arrow, across the first polygon | Change **Inset**. Dragging out, away from the middle, makes the copy bigger. |
| Shift's arrow, on the moved copy | Change **Shift**. The arrow rides on the copy as it moves. |

## Properties

| Name | What it does | Default |
|---|---|---|
| **Inset** | How far each edge moves in across its polygon. A negative value grows the copy outward. | 0 m |
| **Stop Before Fold** | Stops each polygon's inset just before its copy would turn over (its edges cross or run backwards). Off, an **Inset** past the middle crosses over. | Off |
| **Keep Cap UVs** | The inner copies keep their polygons' UVs exactly, seams included; only the side polygons get new UVs. Off, the copies' UVs shrink with them. | Off |
| **Shift** | (Shift only) How far each copy moves along its polygon's normal. A negative value pushes it in. | 0 m |

Distances are measured in the scene, so they stay the same on a scaled
item.

## Tips

- New points keep the weights, colours and morphs of the corners they came
  from, and the side polygons are the same type as their polygon (SubD
  stays SubD).
- **Stop Before Fold** is handy for a quick "as far as it goes" inset:
  type a big number and each polygon stops at its own limit.

## Related

- [Bevel (polygons)](../modeling/polygon-bevel.md): Shift and Inset together,
  with grouping, segments and profiles.
