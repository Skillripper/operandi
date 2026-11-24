<!-- help-ids: ; template: concept ; since: 0.1.0 -->
# Kits and extensions

A kit is an add-on: a set of commands, buttons, mesh operations or item
kinds that extends Operandi. This version can't install kits yet. This
page says what you can extend today, and what isn't there.

## Put a script on a menu or button

Every menu and toolbox tab is a form, and a form's buttons run command
lines. Add a button whose line runs your script, and it works like a
built-in command.

This line adds a button to the **System** menu that runs a script file:

```python
import operandi

operandi.run("operandi.form.add",
             line="operandi.python.run file:{/Users/me/Scripts/wave.py}",
             form="menu.system")
```

You can type the same thing as a command line:
`operandi.form.add {operandi.python.run file:{/Users/me/Scripts/wave.py}} form:menu.system`.

- Open **System > Form Editor** to give the button a label, an icon, or a
  different place. You can also make a form of your own there, for a
  toolbox tab full of your scripts.
- Your changes are saved as your own copy of the form in the **Forms**
  folder, inside **Layouts** in Operandi's data folder. The shipped form
  stays as it was.
- `operandi.form.revert menu.system` puts the menu back the way it
  shipped and drops your copy.
- A script can't have a key of its own yet: the **Input Map** binds
  Operandi's own actions, not command lines.

## Scenes made with kits

A scene can hold items of a kind that comes from a kit, such as a point
cloud. This version has no kits installed, so it treats such items with
care instead of refusing the file:

- the scene opens, and the item shows in the **Items** list marked as
  needing its kit;
- the viewport draws it as a dashed box where it sits;
- it is saved back exactly as it came, so a later build with the kit gets
  it back whole;
- a script can read its values with
  `item.channel name:<value> value:? item:<id>`, but not change them.

Mesh operations of an unknown kind are kept the same way.

## For developers building Operandi

If you build Operandi from source, two entry points already take additions
from outside Operandi's own tables. Both are Rust, compiled into your own
build; nothing is loaded while Operandi runs.

| Entry point | Adds | Rules it checks |
|---|---|---|
| `Registry::register` (`crates/operandi-core/src/command_runtime.rs`) | Commands under a namespace of your own, such as `mykit.cable.make` | Every name starts with the namespace; no namespace or name of Operandi's own; something to run it |
| `meshop_types::register` (`crates/operandi-core/src/meshop_types.rs`) | A mesh operation type, with its form and default values | A dotted id such as `mykit.cable.item`; every value has a form field; the defaults read back |

Read [CONTRIBUTING.md](https://github.com/operandi3d/operandi/blob/main/CONTRIBUTING.md)
before you start; it covers licences and how changes get in.

## What isn't there yet

This version has none of the following:

- a Kits folder, a kit manifest, a **Kits** menu, or a way to turn a kit
  on and off;
- commands, panels or mesh operations written in Python;
- compiled modules loaded while Operandi runs;
- helper programs that talk to Operandi from their own process (other
  than the test bridge);
- drawing or editing kit items, or file formats added by a kit.

Until then, Python scripts and forms are the way to extend Operandi
without rebuilding it.

## Related

- [Python scripting](python.md)
- [Python reference](python-reference.md)
