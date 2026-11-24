<!-- help-ids: menu.window ; template: panel ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/commands/guide_menus_tests.rs from the app's menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->
# Window menu

The entries of the **Window** menu, in order. A **▸** opens a submenu. Keys in brackets run the entry; the `code` is the command it runs.

- **Layouts ▸**
  - **Model** · `layout.restore Model`
  - **UV** · `layout.restore UV`
  - **Animate** · `layout.restore Animate`
- **Panels ▸**
  - **Items** (**5**) · `operandi.panel.show kind:{Items}`
  - **Shader Tree** · `operandi.panel.show kind:{Shader Tree}`
  - **Groups** · `operandi.panel.show kind:{Groups}`
  - **Properties** · `operandi.panel.show kind:{Properties}`
  - **Mesh Stack** · `operandi.panel.show kind:{Mesh Stack}`
  - **UV Maps** · `operandi.panel.show kind:{UV Maps}`
  - **Preset Browser** · `operandi.panel.show kind:{Preset Browser}`
  - **Command History** · `operandi.panel.show kind:{Command History}`
  - **Vertex Normal Tools** · `operandi.panel.show kind:{Vertex Normal Tools}`
  - **Statistics** · `operandi.panel.show kind:{Statistics}`
  - **Node View** (**Ctrl+Shift+N**) · `operandi.window.nodeView`
- **Diagnostics…** · `operandi.window.diagnostics`
- **Frame Time** · `operandi.window.frameTime`
- **MCP Bridge** · `operandi.mcp.bridge`
- **Compact** · `operandi.window.size size:compact`
- **Standard** · `operandi.window.size size:standard`

## Related

- [Menus](index.md)
- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)
