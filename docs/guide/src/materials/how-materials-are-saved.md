<!-- help-ids: ; template: concept ; since: 0.1.0 -->
# How materials are saved

Every material in a scene is saved as a **MaterialX** graph, the open
standard for describing materials. The graph is the master copy: it is
exactly what the renderer draws. The **Shader Tree** is how you edit it.
Its layers are saved too, so the tree opens just as you left it, with
its order, names, masks and hidden layers.

You don't need to do anything for this. Save as usual.

## Older scenes

Scenes saved by earlier test builds have only their layers. They open
the same as before, and the next save adds their graphs.

A scene saved with graphs can't be opened by builds older than the one
that saved it. Keep a copy if you still need an older build.

## When the layers and the graph disagree

This is rare. It can happen if a scene was saved by a build whose layers
worked out slightly differently, or if another program changed the
graph inside the file. Operandi then trusts the graph, so renders and
exports look the way the scene did when it was saved.

As soon as you edit the layers that shade those polygons, the layers
are the truth again and the graph is made from them. The viewport always
draws the layers.

## Related

- [Shader Tree](../panels/shader-tree.md)
- [File menu](../menus/file.md) (**Import MaterialX…**, **Export Material as MaterialX…**)
