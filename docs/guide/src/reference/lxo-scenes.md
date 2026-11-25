<!-- help-ids: scene.open format:lxo ; template: concept ; since: 0.1.0 -->
# Opening LXO scenes

**File > Open…** reads `.lxo` scene files: their meshes, and now their animation, cameras, lights and rigs too. The scene opens ready to play, with the same frame rate and time range it was saved with.

## What comes in

- **Meshes, locators and groups** keep their names, places and parents. Keyed items open with their keys, so a mesh that moves still moves.
- **Keys and curves.** Every keyed channel Operandi has becomes a key curve on that channel, with the same shape between keys, the same slopes and handle weights, stepped and linear segments, and the same behaviour before the first key and after the last (Repeat, Oscillate, Offset Repeat, Linear and the rest). Where Operandi would work a handle out differently, the handle comes in as **Manual** so the curve keeps its exact shape. A key whose in and out values differ comes in as two keys one tick apart.
- **Time.** The frame rate, the scene range, the playback range, the time display and the current time.
- **Cameras** with their lens, film back, clipping, depth of field and resolution settings, and **lights** of every type with their intensity, falloff, shadows, size, cone and colour. Keyed focal length, intensity, colour and cone angle play back.
- **Joints and skins.** Joints come in as joints, and each skinned mesh gets a skin whose weight maps are named after their joints. The joints' setup values become the setup pose the skin binds to.
- **Custom channels** (user channels) with their type, label, limits, value and keys, so the sliders a rig is driven by are there before anything that reads them.
- **Constraints.** Position, rotation, scale, direction and path constraints, and dynamic parents, with their offsets and target weights. Open curves come in too, so a path constraint has its path.
- **Channel links and modifiers.** A channel driving another comes in as a link (or, between a rotation and a distance, as an Expression that converts degrees). A Channel Relationship comes in as a link in **Relationship** mode with its curve. Math (add, subtract, multiply, divide, min, max) and Expression modifiers come in as Operandi modifiers.
- **IK.** Two-bone IK and two-bar planar IK come in as two-bone IK; three-bar planar IK comes in as chain IK.
- **Morph maps** come in as morph shapes on the mesh, and weight maps as weight maps.

## What is left out

The import report lists everything it didn't bring in, by kind. The common ones:

- Animated morph amounts (the shapes come in, still).
- Deformers other than skins, most rig modifiers (including matrix blends, and Math's remainder and average), physics, particles and baked point caches.
- Keys on materials and shader layers: materials come in as polygon tags only.
- Extra animation layers and render passes; only the main animation comes in.
- Changes made at the current time without keying them, on keyed channels. The report says how many.
- Keys on pivot transforms.

## Tips

- **Older scenes.** LXO files saved by older versions (2xx to 10) open too: their meshes, UV maps, materials and parts. Data that plug-ins (renderers, importers) saved into a scene is left out, with one note per kind in the import report. The very first LXO files (version 202 and earlier) can't be read yet; re-save those in a newer version. If an older scene still won't open, the message says what went wrong and where. Please report it with the file attached (**Help > Report Bug or Idea…**) so support for older scenes can improve. As a last resort, re-saving it from version 16 or 17 lets it open.
- A rig's custom channels (the sliders that drive it, such as an arm angle or landing gear) are on the **User Channels** tab of the item that holds them, usually the rig's top locator. Drag a slider to pose the rig; with Auto Key on, the change is keyed.
- An imported two-bone IK chain bends to the side its setup pose bends. If the knee points the wrong way, set a pole target on the IK. A three-bar chain shares its bend the way Operandi's chain IK does, which can differ from the original.
- **File > Open** and **File > Import** both bring the animation in. Import adds the file's items to your scene; if your scene has no keys yet, it also takes the file's frame rate and time range.
- Open the **Graph Editor** to see the imported curves; handles that came in as **Manual** show it in their slope type.
