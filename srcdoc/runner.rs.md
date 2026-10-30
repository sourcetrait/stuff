# runner.rs

## enum RunEventKind

The kind mirror the estate's fieldful enums carry; nothing matches on it
yet, so it is allowed as dead code.

## struct Runner

### fn command

stdin is null: a script cannot prompt, so a run never waits on input the TUI
cannot give it. The script runs from its component's directory, so a
relative path in it is the component's own.

### fn present

Exit 0 is added and 1 is not; any other exit is an error carrying the
script's stderr, since a state that cannot be read must not be taken as
either.

### fn present_all

Each present script runs on a thread of its own: nu's start, its config included,
dominates the time, and the checks are independent.

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
