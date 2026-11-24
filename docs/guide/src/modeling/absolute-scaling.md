<!-- help-ids: scale.absolute, Absolute Scaling ; template: tool ; since: 0.1.0 -->
# Absolute Scaling

**Absolute Scaling** resizes the selection to a size you type, such as exactly 2 m wide, instead of by a factor.

## How to use

1. Select what you want to resize, or nothing to work on the whole active mesh.
2. Open **Snaps and Precision** on the **Create** tool tab, then click **Absolute Scaling** under **Commands**. The **Absolute** window opens; click the button again to close it.
3. Click **Grab Size** to fill the fields with the selection's current size.
4. Type the sizes you want, then click **Uniform Scale**, **Explicit Scale** or **Reference Scale**.

Each scale is one undo step and only moves points. Afterward the fields show the new size, and the status line reports it. The fields keep their values until you quit.

## The buttons

| Button | What it does |
|---|---|
| **Grab Size** | Reads the selection's size into **X**, **Y**, **Z** and **Uniform**. It changes nothing. |
| **Uniform Scale** | Scales every axis by the same factor, so the side picked in **Uniform Axis** becomes **Uniform** long. |
| **Explicit Scale** | Scales each axis on its own, so the selection's box becomes **X** × **Y** × **Z**. |
| **Reference Scale** | Works out the factors that **Explicit Scale** would use on the selection, then scales the whole mesh by them. |

## Properties

| Name | What it does | Default |
|---|---|---|
| **Uniform** | The length the **Uniform Axis** side is scaled to. | 0 m |
| **Uniform Axis** | Which side of the selection's box **Uniform** measures: **Longest Axis**, **X Axis**, **Y Axis** or **Z Axis**. | Longest Axis |
| **Orient To Selection** | Measures the box along the selection's own sides instead of the world axes. Provisional. | Off |
| **X**, **Y**, **Z** | The size the box gets on that axis. 0 leaves the axis alone; a negative size mirrors it. | 0 m |
| **X Center**, **Y Center**, **Z Center** | What stays put on that axis: the box's **Center**, its **High** or **Low** side, or the item's **Origin**. | Center |

## Tips

- **Reference Scale** is handy when you know the real size of one part. Select a door that should be 2 m tall, type 2 m in **Y** (leave **X** and **Z** at 0), and the whole mesh is scaled on Y by the same factor, about the whole mesh's box. Type the matching widths in **X** and **Z** too if you want it to grow evenly.
- Sizes are measured in each mesh's own space, so an item's own scale doesn't count. With several meshes selected, they share one box.
- An axis that is flat (size zero) is left alone by every button. If the **Uniform Axis** side is flat, **Uniform Scale** does nothing and says so.

## Related

- [Snaps and Precision](snaps-and-precision.md)
- [Dimensions](dimensions.md)
- [Ruler](ruler.md)
