<!-- help-ids: menu.edit ; template: panel ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/commands/guide_menus_tests.rs from the app's menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->
# Edit menu

The entries of the **Edit** menu, in order. A **▸** opens a submenu. Keys in brackets run the entry; the `code` is the command it runs.

- **Undo** (**Ctrl+Z**) · `app.undo`
- **Redo** (**Shift+Ctrl+Z**) · `app.redo`
- **Reset Tool Attributes** (**Ctrl+D**) · `Reset Tool Attributes`
- **Snapping ▸**
  - **Snaps and Precision** · `Snaps and Precision`
  - **Snapping Options…** (**F11**) · `Snapping Options`
  - **Snapping** (**X**) · `Snapping Toggle`
  - **Mesh Constraints…** · `Mesh Constraints`
  - **Guides** · `Guides`
- **Center Selected ▸**
  - **All** · `vert.center all`
  - **X** · `vert.center x`
  - **Y** · `vert.center y`
  - **Z** · `vert.center z`
  - **XY** · `vert.center xy`
  - **YZ** · `vert.center yz`
  - **ZX** · `vert.center zx`
- **Lock Selected** (**J**) · `Lock Selected`
- **Lock Unselected** (**Shift+J**) · `Lock Unselected`
- **Lock Invert** (**Ctrl+J**) · `Lock Invert`
- **Unlock** (**I**) · `Unlock`
- **Input Map…** · `operandi.inputMap`
- **Preferences…** (**Ctrl+,**) · `Preferences`
- **Reset Preferences…** · `Reset Preferences`

## Related

- [Menus](index.md)
- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)
