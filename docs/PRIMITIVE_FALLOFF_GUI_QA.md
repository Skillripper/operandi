# Primitive MeshOps and falloffs: GUI checks (GitHub #14, #15)

For Jason or Codex, on the QA build. Claude can't click in Operandi.

1. **Live falloff.** Create > Cylinder, drag one out and give it height. Set
   Segments to 12. Pick Falloff > Linear. The Cylinder tool stays on. Open
   Tool Properties > Falloff (it starts folded) and click Fit Falloff to
   Shape, then set Scale to 0%. The cylinder comes to a point at the top.
2. **Move the falloff mid-tool.** Drag the Linear falloff's End arrow. The
   cone shape follows while you drag. Then drag the cylinder's own side
   square. The radius changes and the taper stays. Set Twist to 90°.
3. **Switch and turn off.** Pick Falloff > Noise, then set Sample At to
   Points. The rings dent. Pick Falloff > (none). The cylinder is plain
   again and the tool is still on. Turn Linear back on and press Q. One
   Undo removes the whole shape.
4. **Greyed rows.** With Falloff > Selection, the Falloff rows are grey
   and hovering one says why.
5. **Cube and Sphere.** Repeat step 1 on a Cube (set Segments Y to 8) and a
   Sphere. On the Sphere, Spacing is greyed with a reason.
6. **MeshOp.** With a Linear falloff on, use Mesh Stack > Add Operator >
   Create > Primitives > Cylinder. Its Falloff section names "Linear". Set
   Falloff Scale to 0%: it tapers. Clear Falloff makes it plain again, and
   Use Active Falloff brings it back.
7. **New primitive ops (#14).** Add Cone, Torus, Capsule, Ellipsoid and
   Teapot ops. Each shows its tool's rows, and changing one updates the
   view. Save, reopen and check that they're unchanged.

## Mesh Stack edge and point selections (GitHub #19)

8. **Edge picks.** On a cube, switch to Edges mode, select two edges, then
   add Mesh Stack > Edge Bevel and set Value to 0.1. Only those two edges
   bevel.
9. **Random edges.** Add Selection > Select Randomly on the Edge Bevel:
   random edges bevel, not whole polygons' edges.
10. **Rule op and reorder.** On a Polygon Bevel, add Select by Polygon Sides
    (Quads), then Select by Facing set to Subtract. Every side but the top
    bevels. Click Move Down on Facing, and the top is included again.
11. **Shifted numbers.** Capture a Select by Index, then put a Triangulate
    below it. The op's row shows the "numbers may point at other
    polygons" hint.
