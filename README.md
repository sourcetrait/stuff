Stuff
================================================================================

Adds and removes components in your home, through a fullscreen TUI or a CLI.
A person picks components from a list; an agent names them on the command
line.

Usage
--------------------------------------------------------------------------------

```
stuff <source>                         the TUI
stuff <source> add                     lists the components not added
stuff <source> update                  lists the components added that can be updated
stuff <source> remove                  lists the components added
stuff <source> add <components>...     adds them, with what they depend on
stuff <source> update <components>...  updates those added
stuff <source> remove <components>...  removes them, with what depends on them
stuff <source> health <components>...  their health, as NUON
stuff --probe                          what the terminal can draw
```

The source is one of three, and anything else is `what stuff?`:

- A URI, a git remote holding a `stuff.toml` at its root: `http://`, `https://`,
  `git://`, `ssh://`, or `user@host:path`. Stuff fetches its head each run and
  keeps it under `$XDG_CACHE_HOME/stuff/source/`.
- A stuff directory, one holding a `stuff.toml`, named by a path: anything holding
  a `/` or starting with `.` or `~`.
- An alias, a plain word, which `$XDG_CONFIG_HOME/stuff/alias.toml` names as a URI
  or a path. A plain word is always an alias, so `stuff rust` means the same
  wherever it runs, and `./rust` names the directory.

```toml
rust = "~/stuff/rust"
team = "git@host:team/stuff.git"
```

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
