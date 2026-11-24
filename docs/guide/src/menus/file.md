<!-- help-ids: menu.file ; template: panel ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/guide_menus_tests.rs from the app's menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->
# File menu

The entries of the **File** menu, in order. A **▸** opens a submenu. Keys in brackets run the entry; the `code` is the command it runs.

- **New** · `scene.new`
- **Open…** · `scene.open`
- **Save** (**Ctrl+S**) · `scene.save`
- **Save As…** · `scene.saveAs`
- **Current ▸**
  - the open scenes or recent projects, one entry each
- **New Image…** · `operandi.image.new`
- **New Image Sequence…** · `operandi.image.newSequence`
- **Save Image** · `operandi.image.save`
- **Save Image As…** · `operandi.image.saveAs`
- **Save All Images** · `operandi.image.saveAll`
- **Scene Save Also Saves Changed Images** · `operandi.paint.alsoSaveImages`
- **Import…** · `scene.importReference`
- **Import MaterialX…** · `operandi.materialx.import`
- **Export ▸**
  - **Wavefront OBJ…** · `scene.saveAs format:obj export:true`
  - **STL (Binary)…** · `scene.saveAs format:stl export:true`
  - **STL (ASCII)…** · `scene.saveAs format:stlAscii export:true`
  - **glTF (.gltf)…** · `scene.saveAs format:gltf export:true`
  - **glTF Binary (.glb)…** · `scene.saveAs format:glb export:true`
  - **VideoScape GEO…** · `scene.saveAs format:geo export:true`
  - **SVG Drawing (Front View)…** · `scene.saveAs format:svg export:true`
  - **LXO Scene…** · `scene.saveAs format:lxo export:true`
  - **PLY (Binary)…** · `scene.saveAs format:ply export:true`
  - **PLY (ASCII)…** · `scene.saveAs format:plyAscii export:true`
  - **Alembic (.abc)…** · `scene.saveAs format:abc export:true`
  - **USD (.usd)…** · `scene.saveAs format:usd export:true`
  - **USD ASCII (.usda)…** · `scene.saveAs format:usda export:true`
  - **USD Binary (.usdc)…** · `scene.saveAs format:usdc export:true`
  - **USDZ (.usdz)…** · `scene.saveAs format:usdz export:true`
  - **FBX (.fbx)…** · `scene.saveAs format:fbx export:true`
  - **3D Manufacturing (.3mf)…** · `scene.saveAs format:3mf export:true`
- **Export Material as MaterialX…** · `operandi.materialx.export`
- **Load Preset…** · `preset.do`
- **Save Item Preset…** · `preset.saveAs`
- **Save Assembly Preset…** · `preset.saveAs kind:assembly`
- **Save Environment Preset…** · `preset.saveAs kind:environment`
- **New Project…** · `operandi.project.new`
- **Set Project…** · `operandi.project.set`
- **Recent Projects ▸**
  - the open scenes or recent projects, one entry each
- **Project Presets ▸**
  - **Load Project Layout** · `operandi.project.loadPreset preset:layout`
  - **Load Project Viewport Display** · `operandi.project.loadPreset preset:viewportDisplay`
  - **Load Project Input Map** · `operandi.project.loadPreset preset:inputMap`
  - **Save Layout to Project** · `operandi.project.savePreset preset:layout`
  - **Save Viewport Display to Project** · `operandi.project.savePreset preset:viewportDisplay`
  - **Save Input Map to Project** · `operandi.project.savePreset preset:inputMap`
- **Close Scene** (**Ctrl+W**) · `scene.close`
- **Close All…** (**Shift+Ctrl+W**) · `scene.closeAll`

## Related

- [Menus](index.md)
- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)
