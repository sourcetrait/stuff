# config.rs

## struct StuffConfigToml, struct ComponentConfigToml

Unknown keys are refused: a module carries its own constants and libraries, so a
component.toml keeps nothing for it, and an old `[add]` block or a key of a
component's own says so at once rather than being quietly ignored. `super` is a
Rust keyword, so that field is `superuser`, renamed for serde.

## struct Actions

`actions` tells Stuff what a component offers and `super` where sudo is
expected, each a list in the file and a set here. A super action still runs as
the user and calls sudo itself; the mark tells Stuff the credential is needed,
so the TUI can ask for it in its dialog before the action's own sudo would ask
on the terminal under the screen.

## impl TryFrom<ComponentConfigToml> for ComponentConfig

A title is required, the card having nothing else to show. A description
past 80 characters is refused rather than cut, so an author finds the
limit when the directory loads instead of seeing a clipped card.

`actions` must be there and list `present` and `add`, the required ones being
expected in it though they are implied. `super` names only what `actions` lists,
and never `health`, which runs without sudo.
