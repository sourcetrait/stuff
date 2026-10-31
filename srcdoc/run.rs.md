# run.rs

## fn dispatch

`--probe` reports and returns. The source is a stuff directory when it holds
a stuff.toml; any other is `what stuff?`, the source named. Every
component's present script runs before anything else, the TUI's first screen
and the CLI's lists and plans alike needing the state; the TUI runs its own
check, which waits on the sudo dialog where a present script is super. A
failing present script stops the command: a state it cannot read would make
every plan a guess.

## fn list

One component a line, its name first and padded, so an agent can take the
first word of each line; the title and the summary follow for a person.

## fn health

Health needs no check of every component first, each health script running
its own present.nu. A record of records quotes each name, a component's name
holding a `/` where it is a subcomponent; a name given twice is checked once.

## fn unsupported_remove

The one warning both front ends give a removal they pass over, its `[stuff]`
yellow.

## fn execute

The plan prints before any step runs, then each step under a `==` line with
its output passed straight through, so an agent sees what will happen and
what did. The first failure ends the command with a nonzero exit and the
error on stderr, the steps after it left unrun. A removal whose component has
no remove.nu is passed over, the rest of the plan run, and warned of on
stderr once the run ends: a warning, the exit still 0.
