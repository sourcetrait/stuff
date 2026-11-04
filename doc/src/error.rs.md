# error.rs

## enum Error

The crate's error wraps each module's transparently, so its message is the
module error's own: a command's stderr reads `stuff: <the failure>` with no
layer of prefixes.

## enum SourceError

### UnknownStuff

`what stuff?` and the source as given, a source Stuff can't take as stuff, with
what it tried: no alias by that name, not a stuff directory, or what a URI
fetched holding no stuff.toml.

### Fetch

A URI that fails to fetch fails as that, never as `what stuff?`: the source was
understood, and the failure is git's, its own message already on stderr.

## enum RunnerError

### Script

Names the component and the script as `<component> <script>`, `base/extra
add.nu`, the pair a person needs to find the failing file.
