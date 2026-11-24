<!-- help-ids: Symmetrize ; template: tool ; since: 0.1.0 -->
# Symmetrize

**Symmetrize** makes a mesh symmetric: it deletes one side and mirrors the other side over it.

## How to use

1. Click **Symmetrize** on the **Duplicate** tool tab. It works on the whole
   active mesh; a selection is ignored.
2. Choose the **Axis** and the **Direction**.
3. Press **Q**, or click **Drop Tool**, to keep it.

## Properties

| Name | What it does | Default |
|---|---|---|
| **Axis** | The axis across which the mesh is mirrored. | X |
| **Direction** | Which side is kept: **Positive to Negative** copies the + side over the − side. | Positive to Negative |
| **Merge** | Welds the two halves along the middle. | On |
| **Offset** | Moves the mirror plane off the middle. | 0 m |
| **Topology** | Mirrors by the mesh's connections rather than by position. | Off |

## Related

- [Duplicate tab](index.md): copies, instances, Instancers and Transform versions
- [Mirror](mirror.md)
