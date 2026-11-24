<!-- help-ids: menu.texture ; template: panel ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/guide_menus_tests.rs from the app's menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->
# Texture menu

The entries of the **Texture** menu, in order. A **▸** opens a submenu. Keys in brackets run the entry; the `code` is the command it runs.

- **Open UV Editor** · `Open UV Editor`
- **Atlas Unwrap...** · `operandi.uv.atlas`
- **Export UVs to SVG...** · `Export UVs to SVG...`
- **Export UVs to EPS...** · `Export UVs to EPS...`
- **Convert UVs to Mesh** · `operandi.uv.toMesh`
- **Transfer UVs** · `operandi.uv.transfer`
- **Bake Curvature...** · `operandi.bake.curvature`
- **Bake Thickness...** · `operandi.bake.thickness`
- **Pad Texture to UVs...** · `operandi.bake.padImage`
- **Paint Tools ▸**
  - **Paint Tab** · `operandi.paint.tab`
  - **Show Texture Locators while Painting** · `operandi.paint.showLocators`
  - **Airbrush** · `operandi.paint.airbrush`
  - **Paintbrush** · `operandi.paint.paintbrush`
  - **Eraser** · `operandi.paint.eraser`
  - **Clone** · `operandi.paint.clone`
  - **Line** · `operandi.paint.line`
  - **Fill** · `operandi.paint.fill`
  - **Linear Gradient** · `operandi.paint.linearGradient`
  - **Radial Gradient** · `operandi.paint.radialGradient`
  - **Smudge** · `operandi.paint.smudge`
  - **Blur** · `operandi.paint.blur`
  - **Sharpen** · `operandi.paint.sharpen`
  - **Lasso** · `operandi.paint.lasso`
  - **Smooth Tip** · `operandi.paint.tip kind:smooth`
  - **Hard Tip** · `operandi.paint.tip kind:hard`
  - **Procedural Tip** · `operandi.paint.tip kind:procedural`
  - **Sphere Tip** · `operandi.paint.tip kind:sphere`
  - **Texture Tip** · `operandi.paint.tip kind:image`
  - **Stamp Tip** · `operandi.paint.tip kind:stamp`
  - **Text Tip** · `operandi.paint.tip kind:text`
  - **Tablet Nozzle** · `operandi.paint.nozzle`
  - **Save Brush Preset** · `operandi.paint.savePreset`
  - **Brush Presets ▸**
    - **No Brush Presets** · `operandi.paint.applyPreset`
- **Add Color Texture…** · `operandi.paint.addTexture kind:color`
- **Add Mask Texture…** · `operandi.paint.addTexture kind:mask`
- **Add Displacement Texture…** · `operandi.paint.addTexture kind:displacement`
- **Add Fur Textures…** · `operandi.paint.addFurTexture`
- **Images** · `operandi.paint.images`
- **Image Processing ▸**
  - **Invert Image** · `operandi.image.invert`
  - **Clear Image** · `operandi.image.clear`
  - **Expand UV Seams** · `operandi.image.expandSeams`
  - **Convert to Vector Displacement** · `operandi.image.toVectorDisplacement`
  - **Set Resolution…** · `operandi.image.setResolution`
  - **Flatten Layers** · `operandi.image.flatten`

## Related

- [Menus](index.md)
- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)
