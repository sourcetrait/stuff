Stuff
================================================================================

Adds and removes components in your home, through a fullscreen TUI or a CLI.
A person picks components from a list; an agent names them on the command
line.

Usage
--------------------------------------------------------------------------------

```
stuff <dir>                         the TUI
stuff <dir> add                     lists the components not added
stuff <dir> update                  lists the components added that can be updated
stuff <dir> remove                  lists the components added
stuff <dir> add <components>...     adds them, with what they depend on
stuff <dir> update <components>...  updates those added
stuff <dir> remove <components>...  removes them, with what depends on them
stuff <dir> health <components>...  their health, as NUON
stuff --probe                       what the terminal can draw
```

The source is a stuff directory, one holding a `stuff.toml`.

In the TUI, `w` `a` `s` `d` move, as do helix's `k` `h` `j` `l` and the
arrows; space marks a component to add or remove, enter goes on, and escape
or `q` goes back or quits. The preview lists what will be installed, what will
be uninstalled, and what will be updated, `w` and `s` scrolling a long one. On
the list, `h` opens the highlighted component's health where it has a health
action, `[h]ealth` showing in the legend, so there helix's `h` moves nothing,
and `u` marks an added component to update where it offers that, `[u]pdate`
showing in the legend.

The TUI has two looks. The fancy look, for a terminal that scales images to
its cells (kitty's protocol or iTerm2's), draws each component with a radio
box, the stuff directory's icon in the header, and rounded borders. The simple
look, for any other, colors a component's title by its state instead, blue
added, green to add, red to remove, cyan to update, marks an added one
`[present]`, and draws bars above and below where the fancy look draws borders.
`--look fancy` or `--look simple` chooses; `stuff --probe` prints what Stuff
finds the terminal able to draw.

License (AGPL3)
--------------------------------------------------------------------------------
SourceTrait Stuff
Developed by [SourceTrait](https://sourcetrait.com), a division of **Asmov LLC**
Copyright (C) 2026 [Asmov LLC](https://asmov.software)

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a [copy](./LICENSE-AGPL-3.txt) of the
GNU Affero General Public License along with this program.
If not, see https://www.gnu.org/licenses/.
