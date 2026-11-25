<!-- help-ids: ; template: concept ; since: 0.1.0 -->
# Deform tab

The **Deform** tool tab holds tools that bend and reshape a mesh without
adding or removing polygons. Most of them are one of the three transform
tools (**Move**, **Rotate**, **Scale**, or **Transform**, which does all
three) set up with a *falloff*: a weight that fades the tool's effect
from full strength to nothing, so part of the mesh moves and the rest
blends smoothly into place.

- A Deform tool starts with its own falloff and puts your own falloff back
  when you drop it.
- The falloff's settings appear under the tool's own in **Tool
  Properties**, and its handles in the 3D view.
- Type a falloff value and press **Enter**, or drag the arrows at the
  end of its field, and the tool's result updates at once (a **Noise**
  falloff's **Scale** sets the size of its lumps).
- **Esc** in the view clears the falloff while no edit is in progress.
- Each drag is one undo step. With nothing selected, a tool works on the
  whole mesh.

The **Move Tools** section:

| Tool | What it does |
|---|---|
| [Element Move](element-move.md) | Drag one point, edge or polygon. |
| [Element Push](element-push.md) | Push the element under the pointer along its normal. |
| [Flex](flex.md) | Bend the selection at its border, like a limb at a joint. |
| [Soft Drag](soft-drag.md) | Drag what is inside a circle round the pointer. |
| [Soft Move](soft-move.md) | Move with a round falloff placed where you click. |
| [Shear](shear.md) | Move with a falloff from one end to the other. |
| [Smooth](smooth.md) | Even out bumps. |
| [Jitter](jitter.md) | Move each point by a random amount. |
| [Quantize](quantize.md) | Snap points to a grid. |
| [Soft Selection tools](soft-selection.md) | Move, rotate or scale the selection, fading out beyond it. |

The other sections:

| Section | Tools |
|---|---|
| **Rotate Tools** | [Twist](twist.md), [Bend](bend.md), [Vortex](vortex.md), [Swirl](swirl.md) |
| **Scale Tools** | [Push](push.md), [Taper](taper.md), [Bulge](bulge.md), [Flare](flare.md) |
| **Align Tools** | [Radial and Linear Align](align.md), [Quick Align](../modeling/quick-align.md), [Ground Align](../modeling/ground-align.md) |
| **Commands** | [Relax Mesh](relax-mesh.md) |

The **Sculpt** button in **Scale Tools** opens the **Brush** tool set's
**Sculpt** tab, where the sculpting brushes are.

## Related

- [Selection modes](../selection/selection-modes.md)
