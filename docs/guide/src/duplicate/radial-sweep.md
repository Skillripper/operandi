<!-- help-ids: Radial Sweep ; template: tool ; since: 0.1.0 -->
# Radial Sweep

**Radial Sweep** sweeps the selection round an axis into one joined shape: turn a side profile into a vase, a wheel or a bowl.

## How to use

1. Select what you want to copy: polygons, or nothing for the whole mesh.
2. Click **Radial Sweep** on the **Duplicate** tool tab.
3. Click in the view to start the tool, then set the copies in **Tool
   Properties** or drag the handles.
4. Press **Q**, or click **Drop Tool**, to keep the copies. **Cancel** takes
   them back.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Count** | How many steps round the axis (segments over the whole sweep). | 8 |
| **Axis**, **Center** | The axis the sweep goes round, and where it passes. | Y, 0 m |
| **Start Angle**, **End Angle** | How far round the sweep goes. | 0°, 360° |
| **Offset** | Moves each step along the axis, for a spiral. | 0 m |
| **Invert Polygons** | Turns the new polygons' normals the other way. | Off |
| **Cap Start**, **Cap End** | Closes the ends of a sweep that doesn't go all the way round. | — |
| **Sweep UVs** | Gives the swept shape UVs, with the chosen direction (**U** or **V**) running round the axis. | — |

## Related

- [Duplicate tab](index.md): copies, instances, Instancers and Transform versions
- [Radial Array](radial-array.md)
