<!-- help-ids: menu.select ; template: panel ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/guide_menus_tests.rs from the app's menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->
# Select menu

The entries of the **Select** menu, in order. A **▸** opens a submenu. Keys in brackets run the entry; the `code` is the command it runs.

- **Vertices** (**1**) · `Vertices`
- **Edges** (**2**) · `Edges`
- **Polygons** (**3**) · `Polygons`
- **Items** (**5**) · `Items`
- **Materials** (**4**) · `Materials`
- **Polygon Tag Type ▸**
  - **Material** · `operandi.select.polygonTagType type:material`
  - **Part** · `operandi.select.polygonTagType type:part`
  - **Smoothing Group (no smoothing group tags yet)** · `operandi.select.polygonTagType type:smoothingGroup`
  - **Selection Set** · `operandi.select.polygonTagType type:selectionSet`
- **Lasso Style ▸**
  - **Freeform** · `operandi.lasso.style style:freeform`
  - **Rectangle** · `operandi.lasso.style style:rectangle`
  - **Circle** · `operandi.lasso.style style:circle`
  - **Ellipse** · `operandi.lasso.style style:ellipse`
  - **Partial** · `operandi.lasso.partial`
- **Close Loop** (**Shift+]**) · `Close Loop`
- **Grow** (**Shift+Up Arrow**) · `Grow`
- **Shrink** (**Shift+Down Arrow**) · `Shrink`
- **Loop** (**L**) · `Loop`
- **Ring** (**Alt+L**) · `Ring`
- **Next Loop** (**Right Arrow**) · `Next Loop`
- **Previous Loop** (**Left Arrow**) · `Previous Loop`
- **Select All** (**Alt+Shift+A**) · `Select All`
- **Invert** (**[**) · `Invert`
- **Connected** (**]**) · `Connected`
- **Invert Connected** · `Invert Connected`
- **Convert to Vertices** (**Alt+1**) · `Convert to Vertices`
- **Convert to Edges** (**Alt+2**) · `Convert to Edges`
- **Convert to Polygons** (**Alt+3**) · `Convert to Polygons`
- **Boundary** · `Boundary`
- **Select More** (**Up Arrow**) · `Select More`
- **Select Less** (**Down Arrow**) · `Select Less`
- **Select Between** (**Shift+G**) · `Select Between`

## Related

- [Menus](index.md)
- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)
