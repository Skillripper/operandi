# Materials as MaterialX graphs

The maintainer's direction (2026-11-19, master plan A31): every material
becomes a MaterialX graph. The Shader Tree is a layer-stack view that
builds the graph, a node editor (A3) will be a second view, Cycles renders
the graph by translating it, and USD export writes it as UsdShade.
`.opscene` stays the native file; materials inside it are stored as
MaterialX text.

Built only from public documentation: the MaterialX specification and
standard library, the OpenPBR specification, OpenUSD's UsdShade and
UsdPreviewSurface docs, and Cycles' own node definitions. Nothing from the
reference application or the research side's tree-to-USD work.

## Pieces

| Piece | Where | Does |
|---|---|---|
| Reader | `operandi-core/src/materialx/doc.rs` | `.mtlx` text to plain data |
| Writer | `materialx/write.rs` | one material and what it reads back to text, prefixes folded |
| Translator | `materialx/cycles.rs` | a material to Cycles nodes (`node_graph::FlatGraph`), constants folded |
| Graph layer | `materialx/graph.rs` | a material the tree can't show as a Node Material layer with a MaterialX graph |
| Storage | `node_graph::StoredGraph::materialx` | the material's MaterialX text, its picture folder; native version 54 |
| Render | `operandi-cycles` `compile_graph` | unchanged: a stored graph flattens and goes through the general node list (NM-30) |
| Tree form | `materialx/tree.rs` | a tree surface (`shader_eval::Principled`, the layer stack composed) to `operandi_*` MaterialX and back, exactly |
| Node library | `materialx/tree_library.rs` | `ND_operandi_*` nodedefs; standard-node nodegraphs for most nodes (see stage 3), checked against the tree at shading points |
| Tree render | `operandi-cycles` `RenderShader::compile_through_graph` | every tree surface is written as its MaterialX form and rendered from the form read back |
| Standard form | `materialx/tree_standard.rs` | a tree surface as standard MaterialX onto `open_pbr_surface`, for USD and other readers |
| Tree to USD | `operandi-usd` `materialx_out::tree_network` | the standard form as a flat UsdShade network, custom nodes expanded (`materialx_graph_out`) |
| Saved graphs | `materialx/saved.rs` | every surface's graph and its bindings in `.opscene` (native 65); pins a saved graph where the layers now differ |

## Stages

1. **Graphs stored and rendered** (built 2026-11-20). Complex `.mtlx`
   materials import as graph layers instead of being refused; Cycles
   renders them; scenes save them as MaterialX.
2. **Graphs through USD** (export and round trip built 2026-11-20; multi-output
   and custom nodes, displacement and project paths still to do). USD export writes graph layers as UsdShade
   MaterialX networks (nodegraphs included); USD import keeps any network
   as a graph. Needed by the Hydra helper (A35) and online libraries (A34).
3. **The tree builds the graph.** Every tree layer gets a MaterialX form
   (`operandi_*` nodes with standard-node implementations where they
   exist). Cycles renders tree materials through the graph, checked against
   today's route, then the old route retires.
   *Built 2026-11-20 (slice 3):* the form is written from the evaluated
   surface, not layer by layer, so blends, opacity, masks and groups are
   the expression the renderer already uses. Cycles renders from the
   form; debug builds and tests check it reads back to the same surface.
   The `operandi_*` nodes reach Cycles through the existing op list (the
   C bridge builds stock Cycles nodes for each), not through
   `materialx::cycles`.
   *Standard versions (same day):* nodes with a mode are one node per
   mode (`operandi_mix_<mode>`, `operandi_planar_<axis>`,
   `operandi_falloff_<shape>`, `operandi_tile_<h>_<v>`, `_alpha`
   variants), since a MaterialX graph can't branch on a string. All 13
   blends, images, UVs, tiling, clipping, locators, normals, planar,
   falloff, facing, gamma, bias/gain, HSV, HSV variation, distance and
   locator incidence have standard-node graphs. Tests check each graph
   uses MaterialX 1.39.4 nodes as defined, and evaluate it at shading
   points against the tree's own sums (`Expr::eval_at`). Images leave
   their colour space to the reader (the tree picks sRGB or raw by the
   input it drives). Cylindrical, spherical and probe projections and
   box, cubic and triplanar face weights followed (one node per axis or
   face). Still without: patterns and cell randoms (Operandi's own
   procedurals), gradient ramps (a sampled curve has far more stops than
   MaterialX's 10-stop `ramp`), geometry inputs (incidence, slope, camera
   distance, screen position and back-facing have no exact standard
   form) and occlusion.
   *USD export (same day):* `materialx::tree_standard` writes a tree
   surface as standard MaterialX (the form, the definitions and
   implementations it uses, and an `open_pbr_surface` by the inverse of
   the OPM-03 table; bump, displacement, rounded edges, inside volumes,
   Node Material layers, material values and Shader settings are listed as
   left out). USD export keeps the simpler Material-and-Image-Map network
   (MX-05) when it leaves nothing out, since it reads back as the same
   layers; otherwise it writes the standard form, flattened with each
   `operandi_*` node expanded through its implementation, and imports
   back as a graph layer. A surface using a node with no standard version
   falls back to MX-05 with a note. File ▸ Export Material as MaterialX
   makes the same choice (the group's surface is read on the first item
   carrying its tag). Pictures that drive a colour input are marked
   `srgb_texture` (EXR/HDR stay linear), the rest are read as numbers;
   a picture used both ways is read as a colour, with a note. The direct
   route retired with stage 4.
4. **The graph is the saved truth.**
   *Built 2026-11-24 (slice 1, `materialx/saved.rs`, native version 65):*
   a scene file keeps every distinct surface its polygons render with as
   its MaterialX form, plus a binding table (item, material tag, part
   tag, selection sets → graph), as MaterialX and UsdShade bind materials
   to geometry. The Shader Tree is saved beside them as the view's layout
   (order, names, masks, hidden layers). On open the layers are evaluated
   again for each binding; where they give a different surface (a build
   with different layer sums, or an outside edit), the saved graph is
   pinned to those polygons and renders and exports use it, until the
   layers that shade them change. Older scenes open from their layers and
   save their graphs from then on; a new, untouched scene saves none.
   Cycles, USD export and File ▸ Export Material as MaterialX all ask
   `saved::surface`, and the direct route from the layers to Cycles is
   gone (`RenderShader::compile` is private; a graph that doesn't read
   back renders clay, which the round-trip tests guard).
   Tests (`tests/material_graphs_saved.rs`): a scene with every texture
   source, procedural kind, gradient input and mask kind, and the icon
   studio's 250 version-44 scenes, open → save → open → save to the same
   file with no pins.
   Placement by reference (MXG-Q13): the form holds texture locator
   matrices as one item sees them, so a graph is written with each
   matrix entry as a reference number and marked `placed`; reading it for
   an item puts that item's own numbers back. Saving checks, once per
   graph, that this gives exactly what the layers give; a surface whose
   sums run through a placement number is saved for its item alone.
   1,000 textured cubes (about 100 layers, every mask kind) save 5 graphs
   and 4,008 bindings: 4.1 MB against 3.5 MB with no materials, in about
   0.5 s (release; `material_graphs_saved::probe`).
   *Not yet:* a pinned graph isn't drawn by the viewport or marked in the
   tree (MXG-Q12); graph-to-layers recognition, so an outside graph shows
   as layers, waits for node editing (MXG-Q11); the saved form is
   Operandi's own (`operandi_*` nodes, placement by reference), so outside
   tools read materials through USD export or Export Material as
   MaterialX, not straight from `.opscene`.
5. **Viewport from the graph.** The Advanced style draws MaterialX graphs.
   *Built 2026-11-20:* `graph_view` reads a graph layer in its flattened
   Cycles form (so MaterialX graphs and Blender Node Materials alike) and
   turns the closure feeding the output (Principled, Emission,
   Transparent, Glass, Mix Shader) into the tree's channels, laid over
   the layers by the layer's coverage. Node outputs become tree
   expressions, plus `Expr::Func` (`shader_func`) for Cycles' Math,
   Vector Math, Map Range, Mapping, the extra colour blends,
   Hue/Saturation/Value, HSV/HSL, Fresnel, Layer Weight and the Noise,
   White Noise, Checker, Gradient and Wave textures, ported from Cycles'
   kernel (`cycles_noise`, checked against values from Cycles' own code)
   to Rust and WGSL. The existing per-pixel programs draw them, in every
   style; tangent-space normal maps and Bump nodes feed the Advanced
   style's normal. All 17 graph layers among the example materials draw
   from their nodes. Not drawn (each input falls back to its own value,
   MXG-Q07): Camera/Window/Reflection coordinates, Incoming,
   non-tangent normal maps, Displacement. A surface closure outside the
   five keeps NM-40's single colour.
   *Voronoi (2026-11-21):* the Voronoi texture draws too, every feature
   (F1, F2, Smooth F1, Distance to Edge, N-Sphere Radius), metric and
   dimension count (`cycles_voronoi`, checked against Cycles' own numbers).

## Node editor (A3)

*Slice 1 (2026-11-20), read-only:* the node view (NM-15) draws
MaterialX graph layers (`materialx::view`: each node a box named by its
MaterialX category, nodegraph nodes as `<nodegraph>/<node>`), and a
Material Group's right-click > Show as Nodes (or
`operandi.nodeMaterial.showGraph layer:<group>`) shows that material's own
graph, the tree form Cycles renders, made again after each edit
(UIC-244: one source of truth). Since 2026-11-21 it is the Node View
panel, docked like any panel and saved with the layout.
Editing waits for a spec (MXG-Q11).

*Slices 2-3 (2026-11-20, docs/NODE_EDITING_PLAN.md):* a Material Group keeps
free MaterialX nodes (`ShaderTree::node_views`, native 55). A wire into a
material channel makes it `Expr::Nodes`; the tree form writes those nodes
as the standard nodes they are, so Cycles still renders from the one
combined graph. Cycles gets them through `materialx::cycles` plus
`OC_EXPR_NODE` (ABI 28); the viewport through `graph_view::output_expr`.
Export writes every free node with `xpos`/`ypos`; a one-material file's
nodes that feed nothing now import and show too.

## Libraries (groundwork for A34)

MaterialX's own 47 example files (62 materials, `fixtures/materialx-examples`)
are imported by a regular test: all but the Disney principled (5) and
hair (1) shaders import, as layers or graph layers. Nodes MaterialX
implements as nodegraphs (`stdlib_ng.mtlx`, and the shader libraries'
helpers such as `gltf_colorimage`) translate through those graphs;
`rgbtohsv`/`hsvtorgb` are Cycles' Separate/Combine Color in HSV mode.
`materialx::library_probe` (opt-in, `OPERANDI_MX_LIBRARY=<folder>`)
reports the same for any downloaded library.

Open questions for slice 1: MXG-Q01..Q06 in SPECIFICATION_QUESTIONS.md.
