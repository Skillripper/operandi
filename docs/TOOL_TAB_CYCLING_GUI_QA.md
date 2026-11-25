# Tool tab cycling: GUI check

QA build, Model tool set, a box in the scene. "Control" is the real Control key on a Mac.

1. [ ] Create tab open. Press **Control+Tab**: Select opens while Control is still held (T1, T11). Again: Deform.
2. [ ] On Curve (the last tab), **Control+Tab** opens Create; **Control+Shift+Tab** goes back to Curve (T2, T3).
3. [ ] Hold **Control+Tab** down: the tabs step one per key repeat.
4. [ ] Start Bevel on the Polygon tab, press **Control+Tab**: Topology opens and Bevel stays active (T6).
5. [ ] Click a Tool Properties number field, press **Control+Tab**: the tab changes, the field keeps focus. Press **Tab**: the next field (T12).
6. [ ] Open Mesh Cleanup… (a dialog), press **Control+Tab**: the tool tab behind it doesn't change (T8).
7. [ ] Over the viewport, **Tab** toggles subdivision as before; no tab change (T10).
8. [ ] **Cmd+Tab** opens the system app switcher; Operandi's tab is unchanged (T9).
9. [ ] Collapse the left column: **Control+Tab** does nothing and doesn't reopen it.
10. [ ] Hover a tool tab: the tooltip lists "Next tab Control+Tab · Previous tab Control+Shift+Tab".
11. [ ] Dock the tool tabs as a horizontal row narrow enough to scroll: **Control+Tab** past the visible end scrolls the row to show the open tab.
