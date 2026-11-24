<!-- help-ids: ; template: concept ; since: 0.1.0 -->
<!-- Written by crates/operandi-app/src/guide_python_tests.rs from crates/operandi-python/src/operandi.py and the command registry. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_python -->
# Python reference

This page lists everything the `operandi` module offers, the commands a script can ask questions with, and the commands that run scripts. It is made from Operandi's own code, so it matches the version it came with. For how to turn scripting on and run a script, see [Python scripting](python.md).

## The `operandi` module

`import operandi` in a script file. In the **Script Console** it is already imported.

### `CommandError` (exception)

A command refused or failed; the message is the command's own.

### `operandi.run(command, check=True, **args)`

Runs a command line, e.g. run("operandi.time.set frame:12") or run("operandi.time.set", frame=12).

Returns True, or the command's answer text for a line that answers. A command that refuses or fails raises CommandError with its own message; with check=False it returns False instead.

### `operandi.query(command, **args)`

Runs a query (it changes nothing) and returns its answer: a Python value for a JSON answer, else the text. Raises CommandError when the line isn't a query or the query fails.

### `with operandi.undo_group(name="Script"):`

Every undo step the block makes becomes one, named name. Groups nest: only the outermost one makes the step.

### `operandi.mesh_get(item=None, flat=False)`

A mesh item's points and polygons (item space, metres) as {"item", "points", "polygons"}. points is a list of (x, y, z), or with flat=True one memoryview of doubles x0, y0, z0, x1, ...

### `operandi.mesh_set(points, polygons=None, item=None)`

Sets a mesh item's points, and its polygons when given, as one undo step. points: a list of (x, y, z), or any buffer of doubles (array("d"), a memoryview) holding x0, y0, z0, x1, ... polygons: lists of point indices. Points alone must keep the point count.

### `operandi.undo(steps=1)`

Undoes the last step (or `steps` steps), as Edit > Undo does. Returns True; raises CommandError when there is nothing to undo.

### `operandi.redo(steps=1)`

Redoes the last undone step (or `steps` steps), as Edit > Redo. Returns True; raises CommandError when there is nothing to redo.

### `operandi.set_time(frame)`

Goes to a frame (not an undo step).

### `operandi.set_channel(item, name, value)`

Sets an item channel such as pos.Y, rot.X or scl.Z (one undo step; Auto Key keys it when on).

### `operandi.set_key(item, *names)`

Keys the named channels of item at the current time.

## Queries

These commands answer a question and change nothing. Ask them with `operandi.query(...)`; a JSON answer comes back as Python lists and dictionaries. `item.channel` and `transform.channel` are queries only when `value` is `"?"`.

| Command | What it answers | Example |
|---|---|---|
| `item.channel` | Sets (or with ? reads) one channel of an item or shader layer; one undo step. Channel names are Operandi's own for now. | `item.channel intensity ?` |
| `operandi.acs.query` | Reads the scene's character rigs as JSON: context, options, modules, guides, joints and bound meshes. | `operandi.acs.query` |
| `operandi.anim.query` | Reads the scene's time settings, playback, pending edits, markers and animated channels with their keys and values, as JSON. | `operandi.anim.query frames:0,12,24` |
| `operandi.eval.graph` | Reads the scene's evaluation graph (who reads whom: time, channels, fields, transforms, shown meshes) as JSON nodes and edges (the Schematic's data). | `operandi.eval.graph` |
| `operandi.ik.jointQuery` | Reads a joint's IK settings and limits as JSON (angles in degrees). | `operandi.ik.jointQuery joint:4` |
| `operandi.ik.query` | Reads IK solvers as JSON: chain joints' solved positions and rotations at a time, goal, pole, and whether the goal is met. | `operandi.ik.query` |
| `operandi.multires.query` | Reads the active mesh's multiresolution levels as JSON. | `operandi.multires.query` |
| `operandi.rig.query` | Reads the rig pieces (all, or one item's) with their channels, inputs, outputs and state, and the item's evaluated world, driven channels and pivot, as JSON. | `operandi.rig.query` |
| `operandi.rig.schematic` | Reads the Schematic's nodes (items and rig pieces), their ports and wires as JSON. | `operandi.rig.schematic` |
| `operandi.skel.query` | Reads the skeleton's joints as JSON: parent, children, first child, world position, bone length and axes. | `operandi.skel.query` |
| `operandi.skin.query` | Reads a bound mesh's skin as JSON: channels and joints; with vertex, its weights and deformed position at a time. | `operandi.skin.query vertex:0` |
| `operandi.weight.query` | Reads weight maps as JSON: the mesh's maps, one map's values, or every map's value at a point. | `operandi.weight.query vertex:0` |
| `transform.channel` | Sets (or with ? reads) one transform channel: pos.X/Y/Z, rot.X/Y/Z, scl.X/Y/Z; one undo step. | `transform.channel name:scl.X value:2` |

## Script commands

The commands behind the **Script Console** and **System > Run Script…**. You can put them on a button or a menu like any other command. A script can't run these itself: a script can't start another script.

| Command | What it does |
|---|---|
| `operandi.python.console [open:toggle\|true\|false]` | Shows or hides the Script Console (Python scripting preview). |
| `operandi.python.run [code:text] [file:path]` | Runs Python: code:{...} or file:&lt;path.py>. The script is one undo step; its output goes to the Script Console. |
| `operandi.python.runScript` | Run Script…: chooses a .py file (starting in the Scripts folder) and runs it. |
| `operandi.python.stop` | Stops the running script (KeyboardInterrupt); Ctrl+C in the Script Console does the same. |

The bulk mesh write that `operandi.mesh_set` makes is also a command of its own, for the test bridge and typed lines:

| Command | What it does |
|---|---|
| `operandi.mesh.setData [item:integer] points:array [polygons:array]` | Set Mesh Data: replaces a mesh item's points, and its polygons when given, as one undo step (scripts, K9). |
