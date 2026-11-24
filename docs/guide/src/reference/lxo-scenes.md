<!-- help-ids: scene.open format:lxo ; template: concept ; since: 0.1.0 -->
# Opening LXO scenes

**File > Open…** reads `.lxo` scene files: their meshes, and now their animation, cameras, lights and rigs too. The scene opens ready to play, with the same frame rate and time range it was saved with.

## What comes in

- **Meshes, locators and groups** keep their names, places and parents. Keyed items open with their keys, so a mesh that moves still moves.
- **Keys and curves.** Every keyed channel Operandi has becomes a key curve on that channel, with the same shape between keys, the same slopes and handle weights, stepped and linear segments, and the same behaviour before the first key and after the last (Repeat, Oscillate, Offset Repeat, Linear and the rest). Where Operandi would work a handle out differently, the handle comes in as **Manual** so the curve keeps its exact shape. A key whose in and out values differ comes in as two keys one tick apart.
- **Time.** The frame rate, the scene range, the playback range, the time display and the current time.
- **Cameras** with their lens, film back, clipping, depth of field and resolution settings, and **lights** of every type with their intensity, falloff, shadows, size, cone and colour. Keyed focal length, intensity, colour and cone angle play back.
- **Joints and skins.** Joints come in as joints, and each skinned mesh gets a skin whose weight maps are named after their joints. The joints' setup values become the setup pose the skin binds to.
- **Constraints.** Position, rotation, scale and direction constraints, and dynamic parents, with their offsets and target weights.
- **Channel links and modifiers.** A channel driving another comes in as a link (or, between a rotation and a distance, as an Expression that converts degrees); Math and Expression modifiers come in as Operandi modifiers.
- **Two-bone IK** chains, with their goal.
- **Morph maps** come in as morph shapes on the mesh, and weight maps as weight maps.

## What is left out

The import report lists everything it didn't bring in, by kind. The common ones:

- Animated morph amounts (the shapes come in, still).
- Deformers other than skins, most rig modifiers, physics, particles and baked point caches.
- Extra animation layers and render passes; only the main animation comes in.
- Changes made at the current time without keying them, on keyed channels. The report says how many.
- Keys on pivot transforms.

## Tips

- An imported two-bone IK chain bends to the side its setup pose bends. If the knee points the wrong way, set a pole target on the IK.
- Open the **Graph Editor** to see the imported curves; handles that came in as **Manual** show it in their slope type.
