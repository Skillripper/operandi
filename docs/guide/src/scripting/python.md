<!-- help-ids: operandi.python.console, operandi.python.run, operandi.python.runScript, operandi.python.stop ; template: concept ; since: 0.1.0 -->
# Python scripting

Python scripts drive Operandi through the same commands its buttons,
menus and keys use, so anything a script does, you could also do by hand.
Scripting is a preview: it is off until you turn it on, and the
[limits](#limits) section lists what it can't do yet.

A script can:

- run any command, the same line a button or menu runs;
- ask questions (queries) and get the answers back as Python values;
- read and write a mesh's points and polygons in one call;
- set channels, go to a frame and set keys;
- bundle everything it does into one undo step.

Every function is listed on the [Python reference](python-reference.md)
page.

## Turn scripting on

1. Choose **Edit > Preferences…**.
2. Go to **Defaults > Application > Experimental**.
3. Tick **Python scripting (preview)**.

The **System** menu now has **Script Console** and **Run Script…**. Turn
the switch off again and both disappear, and every script command is
refused with a message saying how to turn it back on.

You can also start Operandi with the environment variable
`OPERANDI_PYTHON=1` to turn scripting on, or `OPERANDI_PYTHON=0` to force
it off. The variable wins over the preference, and the preference shows
why it can't be changed.

A build made without Python says *Python isn't in this build* when you
try to run a script.

## The Script Console

**System > Script Console** opens a floating window for trying out lines
of Python.

- Type a line and press **Enter**, or click **Run**. A line that is one
  expression prints its value, as in an interactive Python prompt.
- `operandi` is already imported. Names you set stay there for the next
  line, so `cube = ...` on one line can be used on the next.
- **Up Arrow** and **Down Arrow** bring back earlier lines.
- Errors and anything a script prints to `sys.stderr` show in red.
- **Stop** (or **Ctrl+C** in the input line) stops a running script.
- **Clear** empties the output.

The console takes one line at a time. Join short statements with `;`, or
put a longer script in a file.

When the console is empty it shows a few lines to try, for example:

```python,norun
operandi.run("mesh.new")
cube = operandi.mesh_get()["item"]
operandi.set_channel(cube, "pos.Y", 2)
help(operandi)
```

## Run a script file

1. Save your script as a `.py` file.
2. Choose **System > Run Script…**.
3. Pick the file. The panel starts in the **Scripts** folder in Operandi's
   data folder, a handy place to keep them.

What the script prints goes to the **Script Console**, and the status line
says when it finished or why it failed.

To run a script from a button or a menu, give it the command line
`operandi.python.run file:{/path/to/script.py}`. Short code works too:
`operandi.python.run code:{print(1 + 1)}`.

## Your first script

This script makes a mesh item, builds a cube from eight points and six
polygons, and lifts it 2 m:

```python
import operandi

operandi.run("mesh.new")
points = [(-0.5, -0.5, -0.5), (0.5, -0.5, -0.5), (0.5, 0.5, -0.5), (-0.5, 0.5, -0.5),
          (-0.5, -0.5, 0.5), (0.5, -0.5, 0.5), (0.5, 0.5, 0.5), (-0.5, 0.5, 0.5)]
polygons = [[0, 3, 2, 1], [4, 5, 6, 7], [0, 1, 5, 4],
            [2, 3, 7, 6], [1, 2, 6, 5], [0, 4, 7, 3]]
operandi.mesh_set(points, polygons)

cube = operandi.mesh_get()["item"]
operandi.set_channel(cube, "pos.Y", 2)
print("Made item", cube)
```

- `mesh.new` adds an empty mesh item and makes it the active item.
- `mesh_set` with points and polygons replaces the active mesh's geometry.
- `mesh_get()["item"]` is the item's id, a number. Scripts name items by
  id, because two items can share a name.

## Run commands

`operandi.run` takes a command line, exactly as a button runs it:

```python
import operandi

operandi.run("mesh.new")
operandi.run("tool.set prim.cube on")
operandi.run("tool.apply")
operandi.run("tool.drop")
operandi.run("item.name {Box}")
```

You can also give the arguments as Python keywords. Values with spaces get
their braces for you, and `True`/`False` become `true`/`false`:

```python
import operandi

operandi.run("operandi.time.set", frame=12)
```

A tool stays active until you drop it, as it does when you click it.
Drop each tool (`tool.drop`) before you run a command that needs no tool,
such as a selection command.

To find the line for something you do by hand, do it once and look in the
**Command History** panel: its **History** tab lists the command line of
each step (see [Find the command for something](README.md#find-the-command-for-something)).

## When a command fails

A command that refuses or fails raises `operandi.CommandError`, with the
same message the status line would show. Catch it to carry on:

```python
import operandi

try:
    operandi.run("no.such.command")
except operandi.CommandError as error:
    print("Didn't work:", error)
```

Or pass `check=False` to get `False` back instead of an error:

```python
import operandi

if not operandi.run("no.such.command", check=False):
    print("Skipped")
```

An error you don't catch ends the script. Its traceback shows in the
**Script Console** and its last line in the status line. Operandi itself
carries on.

## Ask questions

`operandi.query` runs a query, a command that answers and changes nothing.
A JSON answer comes back as Python lists and dictionaries:

```python
import operandi

time = operandi.query("operandi.anim.query")
print(time["fps"], "frames per second, from", time["start"], "to", time["end"])
```

Some commands answer when you give `?` as the value:

```python
import operandi

operandi.run("mesh.new")
cube = operandi.mesh_get()["item"]
print(operandi.query("transform.channel", name="pos.Y", value="?", item=cube))
```

`query` refuses a line that would change something, so a script can't
change the scene by accident while asking. The
[Python reference](python-reference.md#queries) lists every query.

## Work with mesh data

`mesh_get` reads an item's points and polygons; `mesh_set` writes them
back as one undo step. Points are in the item's own space, in metres.

This script makes a grid of points and lifts each one by a wave:

```python
import math
import operandi

size = 20
points = [(x / size - 0.5, 0.0, z / size - 0.5)
          for z in range(size + 1) for x in range(size + 1)]
polygons = [[z * (size + 1) + x, z * (size + 1) + x + 1,
             (z + 1) * (size + 1) + x + 1, (z + 1) * (size + 1) + x]
            for z in range(size) for x in range(size)]
operandi.run("mesh.new")
operandi.mesh_set(points, polygons)

wavy = [(x, 0.05 * math.sin(x * 12) * math.cos(z * 12), z) for x, y, z in points]
operandi.mesh_set(wavy)
```

- Points alone (no polygons) must keep the same count. Polygons, UV maps
  and the selection stay as they were.
- Points with polygons rebuild the mesh. UV maps, vertex maps and the
  component selection are dropped, because they belonged to the old
  polygons.

For big meshes, `mesh_get(flat=True)` gives the points as one flat
`memoryview` of numbers (x0, y0, z0, x1, ...), and `mesh_set` takes the
same back, for example as an `array("d", ...)`:

```python
import array
import operandi

operandi.run("mesh.new")
operandi.mesh_set([(0, 0, 0), (1, 0, 0), (0, 1, 0)], [[0, 1, 2]])

flat = array.array("d", operandi.mesh_get(flat=True)["points"])
for i in range(1, len(flat), 3):
    flat[i] += 1.0   # every y up by 1 m
operandi.mesh_set(flat)
```

## Animate

`set_time`, `set_channel` and `set_key` go to a frame, set a channel and
key it. Channel names are short, such as `pos.Y`, `rot.X` and `scl.Z`.
This script makes a cube bounce once over a second:

```python
import operandi

operandi.run("mesh.new")
operandi.run("tool.set prim.cube on")
operandi.run("tool.apply")
operandi.run("tool.drop")
ball = operandi.mesh_get()["item"]

for frame, height in [(0, 0), (12, 2), (24, 0)]:
    operandi.set_time(frame)
    operandi.set_channel(ball, "pos.Y", height)
    operandi.set_key(ball, "pos.Y")

operandi.set_time(6)
print(operandi.query("operandi.anim.query", item=ball, frames="6")["channels"])
```

Going to a frame is not an undo step; setting a channel and setting a key
are.

## Undo

A whole script is one undo step, so one **Ctrl+Z** takes back everything
it did. The step is named *Script* and the file's name, or *Console:* and
the line. A script that made just one step keeps that step's own name.

Inside a script, `undo_group` bundles part of the work into one step with a
name of your own:

```python
import operandi

operandi.run("mesh.new")
item = operandi.mesh_get()["item"]
with operandi.undo_group("Nudge"):
    operandi.set_channel(item, "pos.X", 1)
    operandi.set_channel(item, "pos.Z", 1)
operandi.undo()   # takes back both moves
```

Groups can be nested; only the outermost one makes a step.
`operandi.undo()` and `operandi.redo()` work like **Edit > Undo** and
**Edit > Redo**, and raise `CommandError` when there is nothing to undo or
redo.

## How scripts run

- One script runs at a time. Running another while one runs is refused.
- Operandi answers each call between frames, so the window stays live
  while a script works and you can stop it.
- **Stop** raises `KeyboardInterrupt` in the script. A script waiting in a
  long call, such as `time.sleep`, stops when that call returns.
- A script can't start another script: the `operandi.python.*` commands
  are refused from inside one.
- Python starts on its own, away from your system: it ignores
  `PYTHONPATH`, `PYTHONHOME` and your own `site-packages`. The standard
  library is there (`math`, `json`, `random`, `array` and so on).
- Operandi carries its own copy of Python, so you don't need to install
  one.

## Limits

These are the edges of the preview today:

- **No item list or selection reads.** A script can't ask for every item
  in the scene, read item names, or read which points, edges or polygons
  are selected. `mesh_get()` with no item reads the active item.
- **Few channels come back as numbers.** `transform.channel` with `?`
  answers with a line of text (*Mesh (2) (item 2): pos.Y = 0*), not a
  number; parse the number after `=`.
- **Tool settings can't be read back.** `tool.attr ... ?` shows its answer
  in the status line, not to the script.
- **No command list from a script.** `command.list` and `help` show their
  answers in the status line, not to the script.
- **No modules of your own yet.** `import` finds the standard library,
  but not other `.py` files next to your script or in the Scripts folder.
- **One line at a time in the console.** Use a file for anything with
  blocks (`for`, `if`, `def`) longer than one line.
- **No UV, weight map, material or curve data** in the bulk calls; only
  points and polygons.
- **No keys for scripts.** The **Input Map** binds Operandi's own actions,
  not command lines; put a script on a menu or button instead.
- **No panels or events.** Scripts run when you start them; they can't add
  a panel or react when the scene changes.
- **macOS first.** The Windows and Linux builds don't carry Python yet.

## Related

- [Python reference](python-reference.md)
- [Kits and extensions](kits.md)
