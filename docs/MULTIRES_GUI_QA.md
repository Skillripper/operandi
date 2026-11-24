# Multiresolution sculpting: GUI checks

Use the QA app named in the merge paste. Brush tool set, Sculpt tab.

1. New scene, add a Sphere (any size). In the Sculpt tab's **Multiresolution** section the read-out says *Not multiresolution*; Level Down, Level Up, Delete Higher Levels and Apply are greyed with a reason on hover; Attenuate is greyed.
2. Click **Add Level** three times. The read-out counts up (*Level 3 of 3 · … polygons*), the mesh gets denser, Level Up greys.
3. Sculpt a few small Push / Carve strokes. Click **Level Down** twice: the mesh shows level 1 without the small strokes.
4. At level 1 (or 0), sculpt a big Move stroke that pulls one side out. Click **Level Up** twice: the small strokes from step 3 sit on the pulled-out side, not where they were.
5. At level 3, pick **Attenuate** and brush over the small strokes: they melt back toward the smooth shape. At level 0 Attenuate is greyed again.
6. Ctrl+Z steps back through each stroke and level change; Ctrl+Shift+Z redoes them.
7. Save, close, reopen: same level, same shape; Level Down / Up still work.
8. At level 2, run a polygon tool that adds polygons (e.g. Bevel one polygon). The read-out adds *polygons changed*, Level Up/Down grey with the reason. **Apply Multiresolution** leaves the mesh as it is and the section goes back to *Not multiresolution*. Undo brings the levels back.
9. **Delete Higher Levels** at level 1 of 3 leaves *Level 1 of 1*.
10. Large mesh: Add Level until about a million polygons; each step should take a second or two, and brushing should feel like sculpting an ordinary mesh of that size.
