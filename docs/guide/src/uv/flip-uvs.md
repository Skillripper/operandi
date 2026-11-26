<!-- help-ids: operandi.uv.flip, uv.flipU, uv.flipV ; template: tool ; since: 0.1.0 -->
# Flip U and Flip V

**Flip U** mirrors the selected UVs left to right, and **Flip V** mirrors them top to bottom, about the center of their bounding box.

## How to use

1. Select the UVs you want to flip, or nothing to flip every visible UV.
2. Click **Flip U** or **Flip V**. They are under **UV** on the **Edit** tool tab, and on the **UV** tool tab.
   - **Alt**-click flips each polygon about its own center.
   - **Ctrl**-click opens the options first, then flips when you click **OK**.

The flip happens at once, as one undo step. There's no tool to drop.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Axis** | **U** flips left to right; **V** flips top to bottom. | The button's own axis |
| **Per Polygon** | Flips each polygon about its own center instead of the whole selection about one center. | Off |

## Tips

- The selection stays where it was: it flips in place about its own center. To mirror across a fixed line instead, use **Mirror U** or **Mirror V**.
- A plain click remembers whether you last chose **Per Polygon**.

## Related

- [Fit UVs](fit-uvs.md)
- [Selection modes](../selection/selection-modes.md)
