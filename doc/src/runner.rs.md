# runner.rs

## enum RunEventKind

The kind mirror the estate's fieldful enums carry; nothing matches on it
yet, so it is allowed as dead code.

## struct Runner

### fn program

The module is taken by `overlay use --prefix <dir> as component`, so an action
is `component add`. The prefix keeps an exported `update` from shadowing nu's
own for the rest of the program: measured on nu 0.116.1, a later `update` fails
at parse without it, and its builtin works beside `component update` with it.

stdin is null: an action cannot read input, so a run never waits on input the
TUI cannot give it. sudo reads a password from the terminal device, not stdin,
so a super action's `sudo -v` still asks on the command line; in the TUI the
credential is cached first. It runs from its component's directory, so a
relative path in it is the component's own.

### fn present

An action's answer comes over stdout, its last line, what the action printed
coming before it: `true` or `false`, and anything else is an error. An exit
other than 0 is an error carrying its stderr, so a broken module or a broken
user config, where nu exits 1 with nothing run, is never taken as absent.

### fn contract

A module's exports, read by `scope commands` after its overlay, which names them
alone, private helpers left out, must be exactly `actions`. nu's type checks
stop at a literal, so this and the answers' own checks are what hold a module to
its contract at runtime. The check needs the user's config for a module that
reaches their libraries.

### fn present_all

Each plain present runs on a thread of its own: nu's start, its config
included, dominates the time, and the checks are independent. The super ones
run after them, one at a time, so on the command line only the first one's
`sudo -v` asks and the rest find the credential cached.

### fn health

The action is called after its overlay, returning its record as any nu command
does, and its record's outer shape is checked in the same nu: a record of
`component`, a string, and `present`, a bool or a record whose fields the
component's health.nutype gives and Stuff takes untyped. The record comes out as
NUON for the command line, or as nu's expanded table, its colors forced on, for
the TUI's page.

### fn spawn_health

The check runs off the loop, the page showing `checking` until it ends.

### fn spawn_step

One step a thread, the TUI choosing the next once one is done, so it can
refresh the credential between steps.

### fn step_piped

stdout and stderr are read on their own threads and sent as they arrive, so
the TUI shows output as it is written; both pipes reach the TUI as one
stream, the way a terminal shows them.

### fn settle

After a step exits 0, present runs again: an action that claims success without
the state to show for it fails the run. An add or an update leaves its component
added, a remove leaves it absent.

## fn config_flags

nushell reads no `env.nu` or `config.nu` for a program, not even with
`--login`, measured with 0.116.1; given `--env-config` and `--config` it
reads both, a relative `source` in `config.nu` included. The paths come from
nu itself, `$nu.env-path` and `$nu.config-path`, so they follow the user's
XDG settings, and a file that does not exist is left out; with neither,
actions run with `-n`.
