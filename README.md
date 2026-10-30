Stuff
================================================================================

Adds and removes components in your home, through a fullscreen TUI or a CLI.
A person picks components from a list; an agent names them on the command
line. Each component is a few nushell scripts in a directory of its own.

Usage
--------------------------------------------------------------------------------

```
stuff in <dir>                         the TUI
stuff in <dir> add                     lists the components not added
stuff in <dir> remove                  lists the components added
stuff in <dir> add <components>...     adds them, with what they depend on
stuff in <dir> remove <components>...  removes them, with what depends on them
stuff probe                            what the terminal can draw
```

`in` names the source: the components in a stuff directory.

In the TUI, `w` `a` `s` `d` move, space marks a component to add or remove,
enter goes on, and escape goes back or quits.

The TUI draws in the tier of what the terminal does well. The image tier, for
a terminal with a graphics protocol (kitty's, sixel, or iTerm2's), draws
rounded lines and images. The lines tier, for a terminal known to draw its own
line characters, draws rounded lines and colors. The plain tier, for any
other, draws colors alone, tinting where it would draw a border. `--tier
image`, `--tier lines`, or `--tier plain` chooses; `stuff probe` prints what
Stuff finds the terminal able to draw.

The stuff directory
--------------------------------------------------------------------------------

```
<dir>/
  config.toml           title = "..."
  logo.png
  logo.bmp
  component/<component>/
    config.toml         title, description (80 characters at most), depends
    logo.png
    logo.bmp
    exists.nu           exits 0 when the component is added, 1 when not
    add.nu              adds it
    remove.nu           removes it
    <subcomponent>/     the same, to any depth
```

A component is named by its path under `component/`, `rust/nightly`. A
subcomponent depends on its parent, and `depends` names any other components
it needs. A component's `config.toml` may hold keys of its own for its
scripts to read.

An icon comes as `logo.png`, an image with transparency, and as `logo.bmp`, a
bitmap four pixels square, its black and transparent pixels clear. The image
tier shows the png, else the bmp, else the default png, a component's icon in
a rounded frame. The other tiers draw the bmp, else the default bmp, in
half-block pixels, two to a cell, one over the other.

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
