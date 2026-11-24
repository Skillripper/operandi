<!-- help-ids: operandi.mesh.fixBlackDots, operandi.edge.zip, operandi.poly.diagonalRedirect ; template: tool ; since: 0.1.0 -->
# Clean-up commands

Three one-click repairs for common problems in a mesh's flow.

## The commands

| Button | What it does | Needs selected |
|---|---|---|
| **Fix Black Dots** | Merges polygon pairs that share a run of edges into one polygon, on the whole active mesh. These pinches shade as dark dots. | nothing |
| **Edge Zip** | Welds the edge rows either side of the selected edges onto them, removing the quads between. | edges with a quad on each side |
| **Diagonal Re-Direct** | Turns the edge flow through two diagonal quads by 45°. | two quads touching at one corner |

## Related

- [Edit tab](index.md)
- [Mesh Cleanup](../interface/window.md)
