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
stuff <dir> health <components>...  their health, as NUON
stuff --probe                       what the terminal can draw
```

The source is a stuff directory, one holding a `stuff.toml`.

In the TUI, `w` `a` `s` `d` move, as do helix's `k` `h` `j` `l` and the
arrows; space marks a component to add or remove, enter goes on, and escape
or `q` goes back or quits. The preview lists what will be installed and what
will be uninstalled, `w` and `s` scrolling a long one. On the list, `h` opens
the highlighted component's health where it has a `health.nu`, `[h]ealth`
showing in the legend, so there helix's `h` moves nothing.

The TUI has two looks. The fancy look, for a terminal that scales images to
its cells (kitty's protocol or iTerm2's), draws each component with a radio
box, the stuff directory's icon in the header, and rounded borders. The simple
look, for any other, colors a component's title by its state instead, blue
added, green to add, red to remove, marks an added one `[present]`, and draws
bars above and below where the fancy look draws borders. `--look fancy` or
`--look simple` chooses; `stuff --probe` prints what Stuff finds the terminal
able to draw.

The stuff directory
--------------------------------------------------------------------------------

```
<dir>/
  stuff.toml            title = "..."
  logo.png              the icon, an image with transparency
  component/<component>/
    component.toml      title, description (80 characters at most), depends,
                        and [present], [add], [remove], each with super
    present.nu          exits 0 when the component is added, 1 when not
    add.nu              adds it
    remove.nu           removes it, where it can be removed
    health.nu           returns its health record, where it has one
    health.nutype       the record's type, beside health.nu
    <subcomponent>/     the same, to any depth
```

A component is named by its path under `component/`, `rust/nightly`. A
subcomponent depends on its parent, and `depends` names any other components
it needs. A component's `component.toml` may hold keys of its own for its
scripts to read.

The stuff directory's `logo.png`, else the sourcetrait logo, heads the TUI in
the fancy look.

Each script is a `def main` with no arguments, run by nu from its component's
directory with your own nushell `env.nu` and `config.nu`, and with
`STUFF_DIR`, `COMPONENT_ID`, and `COMPONENT_DIR` set. After an add or a
remove, `present.nu` runs again to confirm it, and a run stops at the first
failure.

A script that needs root says so in its block, `[add]` with `super = true`.
It still runs as you, in your environment: shortly before its one command
that needs root it calls `sudo -v`, then puts sudo in front of that command,
`sudo npm -g install ...`. On the command line that `sudo -v` asks on the
terminal. In the TUI, a run that needs sudo asks first, in a yellow `[sudo]`
dialog holding sudo's own exchange, and Stuff's `sudo -v` refreshes the
credential before each step; with a super `present.nu` the dialog comes
before the list, every present script running as the TUI opens.

A component without `remove.nu` stays: a removal that takes it removes the
rest, then warns `[stuff] remove not supported: <component>` on stderr, or in
the TUI's output.

`health.nu`, which needs `health.nutype` beside it, reports on its component
for user-level administration: versions, sizes, what to clear. It runs as
you, without sudo, and returns `record<component: path, present: oneof<bool,
record>>`, its `health.nutype` giving the present record's own fields; it
runs `present.nu` first and returns `{component: <component>, present:
false}` when the component isn't added. A component without `health.nu`
reports `present.nu`'s answer alone. `stuff <dir> health` prints one
component's record as NUON, or a record of records keyed by name for several;
in the TUI the health page draws it as a table.

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
