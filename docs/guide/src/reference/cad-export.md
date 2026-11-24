<!-- help-ids: scene.saveAs format:step, scene.saveAs format:iges ; template: concept ; since: 0.1.0 -->
# CAD export (STEP and IGES)

**File > Export > STEP (.step)…** and **IGES (.iges)…** write your visible meshes as surfaces a CAD program can open, for 3D printing services, product design and engineering hand-off.

A Subdivision surface (the smooth surface you get with **Tab**) is made of small curved patches, and each one is written into the file as an exact NURBS surface. Nothing is approximated: the CAD program gets the same surface you see in the viewport, at any zoom.

## What goes in the file

- **Subdivision (Tab) polygons** become curved B-spline faces, about four per cage polygon (one per corner). Neighbouring faces share their edges exactly, so there are no gaps. Edge weights and creases come along, because they are already part of the patches.
- **Face polygons** become flat faces. A polygon whose corners don't lie in one plane is split into flat triangles first; the export report says how many.
- **Catmull-Clark (Shift+Tab) polygons** have no exact NURBS form, so they come out as flat facets of their render-level surface. Use **Tab** for anything you want exact in CAD.
- Lines and curves, hidden items, cameras, lights and backdrops are left out.

## Solids and surfaces

In **STEP**, faces joined by shared edges form one piece. A piece with no open edges is written as a closed solid, which CAD programs can measure, cut and fillet. A piece with open edges (a single sheet, a cup with an open top) is written as a surface. Each piece is named after its item.

**IGES** is the older, simpler format: it holds the same faces, but as separate surfaces with no solids. Prefer STEP when the program you're sending to reads it.

## Units and axes

Files are written in millimetres with Z up, which is what CAD programs expect. Your scene's metres are converted, and its up direction (Y) becomes Z. The File I/O unit and up-axis preferences don't apply to these two formats.

## From a script

`scene.saveAs filename:"/path/part.step" format:step` and `format:iges` do the same as the menu entries, from the command line, Python or the MCP bridge. A file name ending in **.step**, **.stp**, **.iges** or **.igs** picks the format by itself.

## Tips

- Patch faces meet with matching position everywhere, but around a point where three, five or more edges meet the surface can be only nearly smooth. That's fine for printing and rendering; if you plan to offset or fillet across those spots in CAD, check them there.
- A dense cage makes a lot of faces. Model with as few polygons as the shape needs before you export.

## Related

- [File menu](../menus/file.md)
- [Edge Weight](../edge/edge-weight.md)
- [Freeze](../edit/freeze.md)
