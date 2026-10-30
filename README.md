Stuff
================================================================================

Adds and removes components in your home, through a fullscreen TUI or a CLI.
A person picks components from a list; an agent names them on the command
line. Each component is a few nushell scripts in a directory of its own.

Usage
--------------------------------------------------------------------------------

```
stuff <dir>                         the TUI
stuff <dir> add                     lists the components not added
stuff <dir> remove                  lists the components added
stuff <dir> add <components>...     adds them, with what they depend on
stuff <dir> remove <components>...  removes them, with what depends on them
```

In the TUI, `w` `a` `s` `d` move, space marks a component to add or remove,
enter goes on, and escape goes back or quits.

The stuff directory
--------------------------------------------------------------------------------

```
<dir>/
  config.toml           title = "..."
  logo.png
  component/<component>/
    config.toml         title, description (80 characters at most), depends
    logo.png
    exists.nu           exits 0 when the component is added, 1 when not
    add.nu              adds it
    remove.nu           removes it
    <subcomponent>/     the same, to any depth
```

A component is named by its path under `component/`, `rust/nightly`. A
subcomponent depends on its parent, and `depends` names any other components
it needs. A component's `config.toml` may hold keys of its own for its
scripts to read.

Icons are 100x100 PNGs with transparency. A terminal with a graphics protocol
(kitty's, sixel, or iTerm2's) shows them as images, any other as half-blocks.

Each script is a `def main` with no arguments, run by nu from its component's
directory with your own nushell `env.nu` and `config.nu`, and with
`STUFF_DIR`, `COMPONENT_ID`, and `COMPONENT_DIR` set. After an add or a
remove, `exists.nu` runs again to confirm it, and a run stops at the first
failure.

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
