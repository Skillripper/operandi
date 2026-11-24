<!-- help-ids: operandi.multires.subdivide, operandi.multires.level, operandi.multires.deleteHigher, operandi.multires.apply, operandi.multires.query ; template: concept ; since: 0.1.0 -->
# Multiresolution

A multiresolution mesh keeps several subdivision levels of the same
surface, and you can sculpt at any of them. Big forms go at a low level,
where a stroke moves a few hundred points; fine detail goes at a high
level. When you step back down and reshape the mesh, the fine detail you
sculpted above follows the new shape.

The controls are in the **Multiresolution** section of the **Sculpt** tab
(in the Brush tool set). The line under the heading shows where you are,
for example *Level 2 of 3 · 98,304 polygons*.

## Add levels

1. Select the mesh you want to sculpt.
2. Click **Add Level**. The mesh is subdivided once and you are now at
   level 1. Its shape before the click becomes the base (level 0).
3. Click **Add Level** again for each level you need. Each level has four
   times the polygons of the one below it.

Add Level always adds a level above the highest one and takes you there,
whichever level you are at.

## Step between levels

- **Level Down** goes to the next lower level; **Level Up** to the next
  higher one.
- Sculpt with any brush at any level. Each stroke and each step is one
  undo step.
- At level 0 you are editing the base cage itself, so you can also use the
  ordinary modelling tools that move points (Move, Rotate, Scale, falloffs)
  there. The levels above follow.

Tools that add or remove polygons don't mix with multiresolution: once
the polygons change, the levels can't follow any more. The read-out then
says *polygons changed*, stepping is greyed, and **Apply Multiresolution**
keeps the mesh as it is. Undo brings the levels back.

## Attenuate

The **Attenuate** brush works on a multiresolution mesh above level 0. It
fades the detail of the level you are at back toward the smoothed level
below it, as if melting that detail away. At level 0, and on meshes
without levels, it is greyed.

## Remove levels

| Button | What it does |
|---|---|
| **Delete Higher Levels** | Removes the levels above the one you are at. At level 0 the mesh becomes a plain mesh again. |
| **Apply Multiresolution** | Keeps the mesh as it is at this level, as an ordinary mesh, and drops the base and the other levels. |

## How it affects tools

- Rendering, export and every viewport show the level you are at.
- The base keeps its UV maps, weight maps and other vertex maps. The
  levels above it carry its UV maps (subdivided along with the polygons)
  and its materials and parts, but not its weight maps or morphs.
- Locked and hidden elements are cleared when you change level.
- Multiresolution saves with the scene. A scene with a multiresolution
  mesh needs this version of Operandi or newer to open.

## Scripting

Every button is a command, so scripts and the test bridge can drive it:
`operandi.multires.subdivide`, `operandi.multires.level level:2` (or
`step:up` / `step:down`), `operandi.multires.deleteHigher`,
`operandi.multires.apply`, and `operandi.multires.query`, which answers
with the levels as JSON. Each takes `item:<id>` to act on a mesh other
than the active one.

## Related

- [Getting started](../getting-started.md)
