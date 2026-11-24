<!-- help-ids: menu.view ; template: panel ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/guide_menus_tests.rs from the app's menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->
# View menu

The entries of the **View** menu, in order. A **▸** opens a submenu. Keys in brackets run the entry; the `code` is the command it runs.

- **Frame All** (**A**) · `Frame All`
- **Frame Selection** (**Shift+A**) · `Frame Selection`
- **UV View** · `UV View`
- **Navigate ▸**
  - **Zoom In** (**Period**) · `Zoom In`
  - **Zoom Out** (**Comma**) · `Zoom Out`
  - **Re-centre** · `Re-centre`
  - **Turn** (**Slash**) · `Turn`
  - **Turn Upright** (**Shift+Slash**) · `Turn Upright`
  - **Align Selected** (**Ctrl+A**) · `Align Selected`
  - **Fit and Align Selected** (**Shift+Ctrl+A**) · `Fit and Align Selected`
- **Hide Selected** (**H**) · `Hide Selected`
- **Hide Unselected** (**Shift+H**) · `Hide Unselected`
- **Hide Invert** (**Control+H**) · `Hide Invert`
- **Unhide** (**U**) · `Unhide`
- **Show Locked Mesh Items** · `operandi.view.showLocked`
- **Show Falloffs** · `operandi.view.showFalloffs`
- **Tool Handles ▸**
  - **Draw Style ▸**
    - **Invisible** · `tool.handleStyle invisible`
    - **Basic** · `tool.handleStyle basic`
    - **Advanced** · `tool.handleStyle advanced`
    - **Default** · `tool.handleStyle default`
- **Split View** · `viewport.split`
- **Four Views** · `viewport.maximize`
- **Viewport Properties…** (**O**) · `Viewport Properties`
- **Tear Off View** · `viewport.detach`
- **Close View** · `operandi.viewport.close`
- **Navigation Help** · `operandi.help.navigation`

## Related

- [Menus](index.md)
- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)
