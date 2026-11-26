<!-- help-ids: operandi.channelSet.create, operandi.channelSet.add, operandi.channelSet.remove, operandi.channelSet.move, operandi.channelSet.rename, operandi.channelSet.delete, operandi.channelSet.current ; template: panel ; since: 0.1.0 -->
# Channel panel

The channel panel keeps the channels you animate most in a small panel over the 3D view, so you can drag and key them without hunting through Properties. It shows one **channel set**: a named list of channels you pick, from as many items as you like, such as a rig's sliders and the rotation of a door.

## Make a channel set

1. Click the labels of the rows you want in Properties (Shift-click adds more). A picked label is highlighted.
2. On the timeline, open **Channel Set**, type a name in **Name**, and choose **Create Channel Set**.

The panel appears over the 3D view with one row per channel. The set is saved with the scene, and so is which set is showing, so the panel comes back when you reopen the file.

## Use the panel

- Each row is **Item : Channel**, its status light, and its value.
- **Drag the value** left or right to change it live: the scene follows as you drag. On a Mac the pointer stays put while you drag, so you never run into the edge of the screen. Hold Shift for fine steps and Ctrl for big ones. The whole drag is one undo step, and with Auto Key on it sets one key.
- **Click the value** to type a number. It applies when you press Enter or Tab; Esc cancels.
- **Click the status light** to set or remove a key at the current time, as on any Properties row. A yellow light means the value was changed but not keyed.
- Driven channels (by a link, constraint or modifier) show their value but can't be changed here.
- Drag the panel's header to move it; it stays where you put it in that view. The **×** closes it (the set is kept).

## Change a set

- **Channel Set > Add Selected Channels** or **Remove Selected Channels** works on the set that's showing.
- A row's **⋯** button moves it up or down, or removes it.
- **Rename to Name** and **Delete Set** are in the same menu. Deleting a set never touches the channels themselves.
- Choose **(none)** to close the panel. With **Show Animated** ticked and no set picked, the panel lists the selected items' animated channels instead.

## Tips

- An imported rig's sliders are on the **User Channels** tab of the item that holds them. Pick their labels there to put them in a set.
- Every action is a command (`operandi.channelSet.create`, `.add`, `.remove`, `.move`, `.rename`, `.delete`, `.current`), so scripts can build panels too.
