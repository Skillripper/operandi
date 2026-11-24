<!-- help-ids: menu.item ; template: panel ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/guide_menus_tests.rs from the app's menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->
# Item menu

The entries of the **Item** menu, in order. A **▸** opens a submenu. Keys in brackets run the entry; the `code` is the command it runs.

- **Create Environment** · `item.create environment`
- **Freeze Instancer** · `Freeze Instancer`
- **Transform ▸**
  - **Reset Position** · `transform.reset translation`
  - **Reset Rotation** · `transform.reset rotation`
  - **Reset Scale** · `transform.reset scale`
  - **Reset All** · `transform.reset all`
  - **Freeze Position** · `transform.freeze translation`
  - **Freeze Rotation** · `transform.freeze rotation`
  - **Freeze Scale** · `transform.freeze scale`
  - **Freeze All** · `transform.freeze all`
- **Primitive ▸**
  - **Cube** · `Cube`
  - **Sphere** · `Sphere`
  - **Cylinder** · `Cylinder`
  - **Cone** · `Cone`
  - **Ellipsoid** · `Ellipsoid`
  - **Unit Ellipsoid in Current Mesh** · `Ellipsoid Unit`
  - **Unit Ellipsoid in New Mesh** · `Ellipsoid New Unit`
  - **Torus** · `Torus`
  - **Capsule** · `Capsule`
  - **Arc** · `Arc`
  - **Tube** · `Tube`
  - **Text** · `Text`
  - **Teapot** · `Teapot`
  - **Teapot in New Mesh** · `Teapot Item`
  - **Plane in New Mesh** · `Plane Item`

## Related

- [Menus](index.md)
- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)
