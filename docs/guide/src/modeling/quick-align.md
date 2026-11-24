<!-- help-ids: Quick Align ; template: tool ; since: 0.1.0 -->
# Quick Align

**Quick Align** lines the selection up with another element: it moves (or turns) the selection so it sits against a point, edge or polygon you pick.

## How to use

1. Make sure the mesh you want to work on is selected in the **Items list**.
2. Click **Quick Align** in the **Align** section of the **Create** tool tab.
   Its key is **Shift+R**.
3. Select what you want to move.
4. Press on the target element in the **3D view**, move to one of the
   arrows that appear, and release.
5. Press **Q**, or click **Drop**, to keep it and close the tool.

Hold **Shift** to align the connected geometry too, or **Control** to
flatten it.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Mode** | **Position** moves the selection; **Orientation** turns it. | Position |
| **Selection Axis** | What counts as one piece: **Selection Island**, **Mesh Island** or **Total Selection**. | Selection Island |
| **Connected** | Moves the geometry connected to the selection too. | Off |
| **Flatten** | Flattens the selection onto the target. | Off |

## Related

- [Interface](../interface/window.md): the **Toolbox** and **Tool Properties**
- [Ground Align](ground-align.md)
