<!-- help-ids: ; template: concept ; since: 0.1.0 -->
# Duplicate tab

The **Duplicate** tool tab makes copies: in a line, a grid, round a circle,
along a path, scattered, or mirrored. Most tools come in up to four
versions:

| Version | Makes |
|---|---|
| Plain (**Clone**, **Array** ...) | Copies of the selected geometry inside the same mesh. |
| **Instance** (Alt-click, or its own button) | Mesh Instances: new items that share the source item's mesh, so editing one edits them all. |
| **Instancer** | One **Instancer** item with a point per copy, drawn as boxes until shaded. Light on memory for thousands of copies. |
| **Transform** | No copies: moves the selected *items* onto the places the tool works out, one item per place. |

Each tool has a *generator* (where the copies go) and an *effector* (what
happens to them), shown as sections in **Tool Properties**. Most wait for
a first click in the view before they start.

| Tool | Copies |
|---|---|
| [Clone](clone.md) | in a line |
| [Array](array.md) | in a 3D grid |
| [Radial Array](radial-array.md) | round an axis |
| [Scatter Clone](scatter-clone.md) | at random in a box |
| [Mirror](mirror.md) | across a plane |
| [Path tools](path-tools.md) | along a curve or a path you click out |
| [Radial Sweep](radial-sweep.md) | swept round an axis into one shape |
| [Symmetrize](symmetrize.md) | one side over the other |

Also on the tab: [Bridge](bridge.md), [Tack Tool](tack-tool.md),
[Duplicate and Duplicate Hierarchy](duplicate-item.md),
[Mesh Paint](../modeling/mesh-paint.md), [Patch](../modeling/patch.md)
(as **Patch Curves**) and the [Paste Tool](../modeling/cut-copy-paste.md).
