<!-- help-ids: menu.animate ; template: panel ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/guide_menus_tests.rs from the app's menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->
# Animate menu

The entries of the **Animate** menu, in order. A **▸** opens a submenu. Keys in brackets run the entry; the `code` is the command it runs.

- **Play** · `Play/Stop`
- **Play Backwards** · `Play Backwards`
- **Auto Key ▸**
  - **Off** · `operandi.key.autoKey mode:off`
  - **Animated** · `operandi.key.autoKey mode:animated`
  - **All** · `operandi.key.autoKey mode:all`
- **Key ▸**
  - **Selected Channels** (**S**) · `Key Selected Channels`
  - **Selected Items** (**Shift+S**) · `Key Selected Items`
  - **Position** (**Shift+W**) · `Key Position`
  - **Rotation** (**Shift+E**) · `Key Rotation`
  - **Scale** · `Key Scale`
  - **Transforms** · `Key Transforms`
  - **Apply Edits** · `operandi.key.apply`
  - **Discard Edits** · `operandi.key.discard`
- **Setup** · `operandi.anim.setup`
- **Constrain ▸**
  - **Position** · `operandi.rig.constrain kind:position`
  - **Rotation** · `operandi.rig.constrain kind:rotation`
  - **Scale** (**R**) · `operandi.rig.constrain kind:scale`
  - **Transform** (**Y**) · `operandi.rig.constrain kind:transform`
  - **Direction** · `operandi.rig.constrain kind:direction`
  - **Distance** · `operandi.rig.constrain kind:distance`
  - **Path** · `operandi.rig.constrain kind:path`
  - **Path Position** · `operandi.rig.constrain kind:pathPosition`
  - **Path Normal** · `operandi.rig.constrain kind:pathNormal`
  - **Position (multiple)** · `operandi.rig.constrain kind:position multiple:true`
  - **Rotation (multiple)** · `operandi.rig.constrain kind:rotation multiple:true`
  - **Direction (multiple)** · `operandi.rig.constrain kind:direction multiple:true`
  - **Remove ▸**
    - **Remove Position** · `operandi.rig.unconstrain kind:position`
    - **Remove Rotation** · `operandi.rig.unconstrain kind:rotation`
    - **Remove Scale** · `operandi.rig.unconstrain kind:scale`
    - **Remove Transform** · `operandi.rig.unconstrain kind:transform`
    - **Remove Direction** · `operandi.rig.unconstrain kind:direction`
    - **Remove Distance** · `operandi.rig.unconstrain kind:distance`
    - **Remove Path** · `operandi.rig.unconstrain kind:path`
    - **Remove Path Position** · `operandi.rig.unconstrain kind:pathPosition`
    - **Remove Path Normal** · `operandi.rig.unconstrain kind:pathNormal`
- **Hierarchy ▸**
  - **Parent** · `operandi.rig.parent`
  - **Unparent** · `operandi.rig.unparent`
  - **Insert** · `operandi.rig.insert`
  - **Extract** · `operandi.rig.extract`
  - **Add Parent** · `operandi.rig.dynamicParentAdd`
  - **Remove Dynamic Parent** · `operandi.rig.dynamicParentRemove`
  - **Compensation** · `operandi.rig.compensation`
- **Channel Links** · `operandi.rig.links`
- **Schematic** · `operandi.rig.schematicView`
- **Timeline** · `operandi.anim.timeline`
- **Graph Editor** · `operandi.graph.open`
- **Animation Editors** · `operandi.anim.editors`
- **Time Ranges…** · `operandi.anim.dialog kind:ranges`
- **Settings…** · `operandi.anim.dialog kind:settings`
- **Render Frame Range…** · `operandi.anim.dialog kind:render`
- **Playblast…** · `operandi.anim.dialog kind:playblast`
- **Export USD Animation…** · `scene.saveAs format:usda export:true`

## Related

- [Menus](index.md)
- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)
