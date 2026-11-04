# run.rs

## fn line

The estate's convention for a user-facing line: `[stuff]` in its tone's color,
blue info, green done, red an error, yellow attention, then the message. The
lines an action writes, and the CLI's lists and health records, which agents
read, stay as they are.

## fn fresh_login

What a run that succeeded says just before its success, on the command line and
in the TUI's output. A component's environment changes reach new logins alone,
Stuff being a child of the shell that called it.

## fn dispatch

`--probe` reports and returns. The source reaches its stuff directory by its form
(`source.rs`). Every module is held
to its component.toml first, a broken one refused before anything runs. Every
component's present runs next, the TUI's first screen and the CLI's lists and
plans alike needing the state; the TUI runs its own check, which waits on the
sudo dialog where a present is super. A failing present stops the command: a
state it cannot read would make every plan a guess.

## fn list

One component a line, its name first and padded, so an agent can take the first
word of each line; the title and the summary follow for a person. An action with
no names lists what it would accept: `add` those not added, `remove` those
added, `update` those added that offer it.

## fn health

Health needs no check of every component first, each health action asking its
own present. A record of records quotes each name, a component's name
holding a `/` where it is a subcomponent; a name given twice is checked once.

## fn unsupported

The one warning both front ends give a step they pass over, its `[stuff]`
yellow.

## fn execute

The plan prints before any step runs, then each step under a `==` line with
its output passed straight through, so an agent sees what will happen and
what did. The first failure ends the command with a nonzero exit and the
error on stderr, the steps after it left unrun. A step whose component doesn't
offer its action is passed over, the rest of the plan run, and warned of on
stderr once the run ends: a warning, the exit still 0.
