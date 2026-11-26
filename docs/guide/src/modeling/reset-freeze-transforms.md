<!-- help-ids: transform.reset, transform.freeze ; template: tool ; since: 0.1.0 -->
# Reset and Freeze transforms

**Reset** puts an item's position, rotation or scale back to zero (scale back to 100%), and **Freeze** bakes them into the mesh so the item keeps its look but its transform reads zero.

## How to use

1. Select the items you want to change in the **Items** list or the view.
2. Open **Item > Transform**.
3. Choose a **Reset** entry to move the items back to identity, or a **Freeze** entry to bake the transform into their geometry.

Each choice is one undo step for the whole selection. If nothing would change, no undo step is added.

## The commands

| Menu entry | What it does |
|---|---|
| **Reset Position** | Sets position to 0 in the parent's space. The item moves; its geometry doesn't change. |
| **Reset Rotation** | Sets rotation to 0. |
| **Reset Scale** | Sets scale to 100%. |
| **Reset All** | Resets position, rotation and scale together. |
| **Freeze Position** | Moves the mesh's points by the item's position, then sets position to 0. Nothing moves on screen. |
| **Freeze Rotation** | Bakes the rotation into the points, then sets rotation to 0. |
| **Freeze Scale** | Bakes the scale into the points, then sets scale to 100%. |
| **Freeze All** | Bakes the whole transform into the points. |

## Tips

- Reset leaves children's own values alone, so they move along with their parent. Freeze adjusts each direct child so it stays where it was.
- Freeze works on mesh items only. Other item types in the selection are skipped. With no mesh item selected, Operandi asks you to select one.
- Locked items are skipped, and the status line says how many. When every selected item is locked, nothing happens and Operandi tells you so.
- Freeze skips an item with a scale of zero, and an item whose child would end up sheared by a non-uniform scale. The status line names the item; **Freeze All** on both the parent and the child usually works.
- Freezing a negative (mirrored) scale reverses the polygons so they still face outward. UVs, weights and materials are kept as they are.
- Drop the active tool before you use Freeze.

## Related

- [Item menu](../menus/item.md)
- [Properties panel](../panels/properties.md)
- [Freeze (subdivision)](../edit/freeze.md): a different Freeze, which turns a subdivision surface into polygons
