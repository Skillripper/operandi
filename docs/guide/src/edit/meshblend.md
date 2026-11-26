<!-- help-ids: meshblend.new, meshblend.strip, meshblend.addSource, meshblend.role, meshblend.trim, meshblend.untrim, meshblend.clearRole, meshblend.source, meshblend.convert, meshblend.item ; template: tool ; since: 0.1.0 -->
# MeshBlend

**MeshBlend** builds a live boolean of ordinary meshes. The meshes stay
editable, and the result rebuilds whenever you move or change one of them.

Each mesh is used the way the viewport shows it. A mesh with subdivision
on (Tab) blends as its smooth surface, so a box with a few segments blends
as a rounded block; a plain mesh blends as it is, sharp corners and all.
The result lives on its own item, the
**MeshBlend Item**: a mesh whose **Mesh Operations** list starts with a
**MeshBlend** operation. The source meshes are never changed.

Where two meshes meet, MeshBlend lays a **strip**: a band of quads that
replaces the sharp crease with a round or a chamfer.

## How to use

1. Select the meshes to blend in the **Items list**.
2. Choose **Geometry > New MeshBlend**. A new item named **MeshBlend**
   appears, selected, showing the union of the meshes.
3. Hide the source meshes in the viewport so you can see the result. Hiding
   a source doesn't take it out of the result.
4. Select the **MeshBlend** row in the item's **Mesh Operations** list to
   see its properties.
5. To cut with a mesh, set its **Role** to **Subtractive Trim**. To keep only
   the overlap, set it to **Intersect Trim**.
6. Move or edit any source mesh; the result follows.

## Roles

| Role | What it does |
|---|---|
| **Primary** | Adds its volume. All Primaries are joined into one. |
| **Subtractive Trim** | Cuts its volume out of the Primaries it acts on. |
| **Intersect Trim** | Keeps only the part of the Primaries it acts on that lies inside it. If it doesn't reach them, they stay whole and its section says so. |

A new Trim acts on every Primary. Untick **Acts on** for a Primary to leave
that one alone. A Trim never changes a Primary it doesn't act on, even where
they overlap.

## Strips

The strip settings are at the top of the **MeshBlend** properties.

| Name | What it does | Default |
|---|---|---|
| **Width Units** | **Distance** uses **Strip Width**; **Percent of Polygon Size** uses **Strip Width %**, a share of the meshes' average edge length, so the strips follow the meshes' density. The other width is greyed. | Distance |
| **Strip Width** | How far each edge of the strip lies from the old crease, measured along the surface. Set it to 0 for sharp joins. | 2 % of the meshes' size when the MeshBlend is made |
| **Strip Width %** | The width as a percent of the meshes' average edge length. | 20 % |
| **Strip Profile** | The strip's cross-section: 0 % is a flat chamfer, 100 % a round that meets both surfaces smoothly, up to 200 % for a slight groove. | 100 % |
| **Strip Rows** | Rows of polygons across the strip: each row adds two quads across. | 3 |
| **Design Mode** | Narrow flat strips that build quickly while you work out the shape; your settings are kept. | Off |
| **Output** | **Draft**: no strips, quickest. **Closed**: strips, the cut faces as triangles. **Closed, clean**: the same with triangle pairs turned into quads. **Closed with parts per patch** / **per source**: also puts each area between strips (or each source) and all the strips in their own parts, ready for materials. **Strips only**: just the strips, as an open band. | Closed, clean |

Where three strips meet, a corner patch of quads joins them. Faces the
strips cut are re-meshed into even triangles, then paired into quads
where they can be.

## One strip on its own

Every strip is listed under **MeshBlend Strips**, named after the two
meshes it joins. Tick **Override Defaults** to give that strip its own
**Width** and **Profile**; untick it to go back to the item's settings
(your values are kept). A strip keeps its name when you move the meshes,
so its settings stay with it. Scripts use `meshblend.strip` with the id
in the strip's tooltip.

If a strip is too wide to fit on the meshes, MeshBlend narrows it and the
**MeshBlend** row's tooltip says the width it used. If no strip fits at all,
the joins are shown sharp, with a note. The shape never collapses.

## Properties

Each source has its own section.

| Name | What it does | Default |
|---|---|---|
| **Role** | Primary, Subtractive Trim or Intersect Trim. | Primary |
| **Acts on** | For a Trim: which Primaries it changes. | All Primaries |
| **In Result** | Off leaves the source out of the result without changing its role. | On |
| **Subdivision** | For a mesh with subdivision on: how finely its smooth surface becomes polygons, 0 to 3. At 0 the mesh is used as it is. Each step makes about four times as many polygons. | 2 |
| **Keep Cage Points** | Uses the mesh's own points instead of its smooth surface, even with subdivision on. | Off |
| **Take Out of MeshBlend** | Removes the source. It's an ordinary mesh again. | |

Below the sources:

| Name | What it does |
|---|---|
| **Add Primary**, **Add Subtractive Trim**, **Add Intersect Trim** | Pick a mesh to add with that role. |
| **Convert to Mesh** | Freezes the result into this item as an ordinary mesh. |
| **Duplicate and Convert** | Makes a new ordinary mesh from the result and keeps the MeshBlend live. |

Every change is one undo step.

## When a source can't be used

A source has to be a closed solid. If one is open, has edges shared by more
than two polygons, faces inside out or passes through itself, MeshBlend
leaves it out, says why in red in its section, and builds the rest. The
**MeshBlend** row's tooltip lists the same lines. Fix the mesh and the
result updates.

## Tips

- The result is the same whatever order the sources were made or added in,
  and the same on every computer.
- To keep working on the result with other tools, add more operations above
  **MeshBlend** in the **Mesh Operations** list, or use **Convert to Mesh**.
- Scripts and the test bridge use the `meshblend.*` commands, listed in the
  commands reference.

## Related

- [Booleans and drills](booleans.md): one-off booleans that change the mesh.
- [Freeze](freeze.md)
