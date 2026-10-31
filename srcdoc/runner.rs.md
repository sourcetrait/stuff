# runner.rs

## enum RunEventKind

The kind mirror the estate's fieldful enums carry; nothing matches on it
yet, so it is allowed as dead code.

## struct Runner

### fn command

stdin is null: a script cannot read input, so a run never waits on input the
TUI cannot give it. sudo reads a password from the terminal device, not stdin,
so a super script's `sudo -v` still asks on the command line; in the TUI the
credential is cached first. The script runs from its component's directory, so
a relative path in it is the component's own.

### fn present

Exit 0 is added and 1 is not; any other exit is an error carrying the
script's stderr, since a state that cannot be read must not be taken as
either.

### fn present_all

Each plain present script runs on a thread of its own: nu's start, its config
included, dominates the time, and the checks are independent. The super ones
run after them, one at a time, so on the command line only the first one's
`sudo -v` asks and the rest find the credential cached.

### fn health

health.nu is sourced from its component's directory and its main called, so a
health script returns its record as any nu command does, and its record's
outer shape is checked in the same nu: a record of `component`, a string, and
`present`, a bool or a record whose fields the component's health.nutype
gives and Stuff takes untyped. The record comes out as NUON for the command
line, or as nu's expanded table, its colors forced on, for the TUI's page.

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

After a step exits 0, the present script runs again: a script that claims success
without the state to show for it fails the run.

## fn config_flags

nushell reads no `env.nu` or `config.nu` for a script, not even with
`--login`, measured with 0.116.1; given `--env-config` and `--config` it
reads both, a relative `source` in `config.nu` included. The paths come from
nu itself, `$nu.env-path` and `$nu.config-path`, so they follow the user's
XDG settings, and a file that does not exist is left out; with neither,
scripts run with `-n`.
