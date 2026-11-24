<!-- help-ids: ; template: concept ; since: 0.1.0 -->
# Scripting and the test bridge

Everything you do in Operandi is a command with a name and a line of
arguments, such as `tool.set poly.bevel on`. Buttons, menus and keys run
those lines, and so can you, from a script or another program.

There are three ways in:

| Way in | What it is | Where |
|---|---|---|
| **Python scripts** (preview) | Python code that runs commands, asks questions and edits mesh data | [Python scripting](python.md) |
| **Forms** | Buttons and menu entries that run any command line, your scripts included | [Kits and extensions](kits.md) |
| **The test bridge** | Lets another program drive Operandi over a local connection, using the Model Context Protocol | **Window > MCP Bridge** |

Because every way in runs the same commands, a script, a button and a
click do exactly the same thing: the same checks, the same undo steps,
the same refusals.

## Find the command for something

Do it once by hand, then look in the **Command History** panel (at the
bottom of the right column in the Model layout; drag the line above its
command entry up to open it). Each step on its **History** tab shows the
command line it ran.

## In this section

- [Python scripting](python.md): turn it on, the Script Console, your
  first script, mesh data, animation, undo, and the limits of the preview.
- [Python reference](python-reference.md): every function of the
  `operandi` module, every query, and the script commands. Made from
  Operandi's own code.
- [Kits and extensions](kits.md): putting scripts on menus and buttons,
  scenes made with kits, and what isn't there yet.
