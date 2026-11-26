<!-- help-ids: operandi.uv.fit, uv.fitUVs ; template: tool ; since: 0.1.0 -->
# Fit UVs

**Fit UVs** scales and moves the selected UVs so they fill a UDIM tile.

## How to use

1. Select the UVs you want to fit, or nothing to fit every visible UV.
2. Click **Fit UVs**. It is under **UV** on the **Edit** tool tab, and on the **UV** tool tab.
   - **Shift**-click fits each UV island on its own.
   - **Alt**-click fits each polygon on its own.
   - **Ctrl**-click opens the options first, then fits when you click **OK**.

A plain click uses the options you set last time. Each fit is one undo step.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Fit** | What is fitted: **Entire UVs** (the selection as one block), **Per UV Polygon** or **Per UV Island**. | Entire UVs |
| **Keep Proportion** | Scales evenly, so the UVs aren't stretched, until the longer side fills the tile. Off stretches both sides to fill it. | On |
| **Gaps** | A margin left inside the tile, as a percentage of the tile (0 to 49.9 %). | 0 % |
| **Gaps by Pixel** | Gives the margin in pixels instead. | Off |
| **Pixel Size** | The margin in pixels of a 1024-pixel texture. Available with **Gaps by Pixel** checked. | 0 px |
| **UDIM** | The tile to fit into, from 1001 to 1999. 1001 is the usual 0 to 1 square. | 1001 |

## Tips

- With **Keep Proportion** on, the UVs line up with the tile's lower-left corner (inside the margin), so a long, thin piece leaves space at the top or right.
- **Gaps** and **Pixel Size** are the same margin given two ways; on a 1024-pixel texture, 10 % is about 102 pixels.

## Related

- [Flip U and Flip V](flip-uvs.md)
- [Selection modes](../selection/selection-modes.md)
