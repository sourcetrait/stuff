# cli.rs

## struct Cli

The source comes first and the action after, so a line reads as a sentence:
`stuff <dir> add rust`. `--look` is global, so it may come anywhere on the
line, and it overrides the look the terminal gets; `--probe` stands alone, a
report on the terminal rather than work on components.

## enum CliAction

`add` and `remove` with no names list rather than act, so an agent can see
what each would accept before naming anything.
