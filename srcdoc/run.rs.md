# run.rs

## fn dispatch

`--probe` reports and returns. The source is a stuff directory when it holds
a config.toml; any other is `what stuff?`, the source named. Every
component's present script runs before anything else, the TUI's first screen
and the CLI's lists and plans alike needing the state. A failing present
script stops the command: a state it cannot read would make every plan a
guess.

## fn list

One component a line, its name first and padded, so an agent can take the
first word of each line; the title and the summary follow for a person.

## fn execute

The plan prints before any step runs, then each step under a `==` line with
its output passed straight through, so an agent sees what will happen and
what did. The first failure ends the command with a nonzero exit and the
error on stderr, the steps after it left unrun.
