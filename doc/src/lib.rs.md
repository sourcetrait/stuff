# lib.rs

The crate is a library behind a thin binary so its tests reach every item
through the hub; the binary's whole surface is `run`.

## mod r

ratatui's and ratatui-image's paths run long, so their items come through
this hub. crossterm is reached as `ratatui::crossterm`, never a direct
dependency, so the backend and the event types are the one instance ratatui
was built with.

## #[cfg(test)] use

The test-only names, `ComponentSpec`, `CatalogErrorKind`, and `Request`, are
used by the unit tests alone; in the main hub they warn as unused imports.

## Dependencies

zeroize clears the sudo password typed into the dialog once it is dropped.

ratatui-image takes its `crossterm` feature alone, its defaults left off for
the `image` formats and the chafa encoder they would add; it finds the
terminal's graphics protocol and draws images through it. `image` is declared
with `bmp` and `png` alone, the two forms an icon comes in, the instance
ratatui-image builds against.

## The stuff directory

```
<dir>/
  stuff.toml            title = "..."
  logo.png              the icon, an image with transparency
  component/<component>/
    component.toml      title, description (80 characters at most), depends,
                        actions, and super
    mod.nu              the module, exporting the actions
    health.nutype       the health record's type, with a health action
    <subcomponent>/     the same, to any depth
```

A component is named by its path under `component/`, `rust/nightly`. A
subcomponent depends on its parent, and `depends` names any other components
it needs. The stuff directory's `logo.png`, else the sourcetrait logo, heads
the TUI in the fancy look.

`mod.nu` is the only nushell a component requires. Its exports are its
interface, each export an action, and what it needs beyond them it keeps
private: constants, helpers, and libraries of its own beside it.

```nu
const NPM_PACKAGE: string = '@openai/codex'

export def present []: nothing -> bool {
    (^npm ls --global --depth=0 $NPM_PACKAGE | complete).exit_code == 0
}

export def add []: nothing -> nothing {
    ^sudo -v
    ^sudo npm install --global $NPM_PACKAGE
}
```

`actions` lists every action the component offers, the required ones among them,
and `super` the ones where sudo is expected:

```toml
title = "ChatGPT Codex"
description = "OpenAI ChatGPT terminal interface"
actions = ["present", "add", "update", "remove", "health"]
super = ["add", "remove"]
```

`present` and `add` are required, and `update`, `remove`, and `health` optional.
Stuff holds `actions` to the module's exports when it loads the directory, and
refuses either without the other.

Each action runs in nu of its own, from its component's directory, with your own
nushell `env.nu` and `config.nu`: `overlay use --prefix <dir> as component`,
then `component add`. Its stdout and stderr flow through to you. After an add,
an update, or a remove, `present` runs again to confirm it, and a run stops at
the first failure. An error is an error: a `present` that fails, or answers
anything but a bool, stops the command rather than reading as not added.

An action that needs root is named in `super`. It still runs as you, in your
environment: shortly before its one command that needs root it calls `sudo -v`,
then puts sudo in front of that command, `sudo npm -g install ...`. On the
command line that `sudo -v` asks on the terminal. In the TUI, a run that needs
sudo asks first, in a yellow `[sudo]` dialog holding sudo's own exchange, and
Stuff's `sudo -v` refreshes the credential before each step; with a super
`present` the dialog comes before the list, every present running as the TUI
opens.

A component without `remove` stays: a removal that takes it removes the rest,
then warns `[stuff] remove not supported: <component>` on stderr, or in the
TUI's output. An update of one without `update` warns the same way.

`health`, which needs `health.nutype` beside the module, reports on its
component for user-level administration: versions, sizes, what to clear. It runs
as you, without sudo, and returns `record<component: path, present: oneof<bool,
record>>`, its `health.nutype` giving the present record's own fields; it asks
`present` first and returns `{component: <component>, present: false}` when the
component isn't added. A component without `health` reports `present`'s answer
alone. `stuff <dir> health` prints one component's record as NUON, or a record
of records keyed by name for several; in the TUI the health page draws it as a
table.
