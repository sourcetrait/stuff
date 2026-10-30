# cli.rs

## struct Cli

The directory is a positional argument, the first, so a wrapper can bake it
in and pass the rest through: `stuff <dir> add rust`.

## enum CliAction

`add` and `remove` with no names list rather than act, so an agent can see
what each would accept before naming anything.
