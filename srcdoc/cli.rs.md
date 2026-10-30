# cli.rs

## struct Cli

`--look` is global, so it may come anywhere on the line, and it overrides the
look the terminal's answers choose.

## enum CliSource

The first word names where the components come from, so the directory and the
action read as a sentence: `stuff in <dir> add rust`. `in` is a stuff
directory; another source is another variant.

## enum CliAction

`add` and `remove` with no names list rather than act, so an agent can see
what each would accept before naming anything.
