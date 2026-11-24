<!-- help-ids: Cut Selection, Copy Selection, Paste Selection, cut, copy, paste, Paste Tool ; template: tool ; since: 0.1.0 -->
# Cut, Copy and Paste

**Cut**, **Copy** and **Paste** move or repeat the selected geometry through Operandi's clipboard. **Paste Tool** puts a copy wherever you click.

## How to use

1. Select the points, edges or polygons you want.
2. Press **Ctrl+C** (**Copy**) to copy them, or **Ctrl+X** (**Cut**) to
   take them out of the mesh.
3. Press **Ctrl+V** (**Paste**). The copy appears where the original was,
   selected, ready to move.

The same commands are the **Cut**, **Copy** and **Paste** buttons in the
**Edit** section of the **Create** tool tab.

## Paste Tool

1. Copy something first.
2. Click **Paste Tool** in the **Edit** section.
3. Click in the **3D view**: each click adds a copy there.
4. Press **Q**, or click **Drop**, to close the tool.

## Paste Tool properties

| Name | What it does | Default |
|---|---|---|
| **Center** | Where the next copy goes (X, Y, Z). | where you click |
| **Size** | Scales the copy along X, Y and Z, about **Center**. | 100% |
| **Bias** | Moves the copy by this much after it is placed. | 0 m |
| **Apply** | Adds one copy at **Center**. |  |

## Related

- [Interface](../interface/window.md): the **Toolbox** and **Tool Properties**
- [Merge and Unmerge](merge-unmerge.md)
