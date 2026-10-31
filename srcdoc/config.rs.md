# config.rs

## struct StuffConfigToml, struct ComponentConfigToml

Unknown keys are allowed, against the estate's usual denial: a component.toml
is shared with its scripts, which may keep any keys of their own there and
read them through `COMPONENT_DIR`, as a stuff.toml through `STUFF_DIR`.

## struct InterfaceToml

A script's block, `[present]`, `[add]`, or `[remove]`, takes `super`, false
unless set. `super` is a Rust keyword, so the field is `superuser`, renamed
for serde.

## struct Supers

A super script still runs as the user and calls sudo itself; the mark tells
Stuff the credential is needed, so the TUI can ask for it in its dialog
before the script's own sudo would ask on the terminal under the screen.

## impl TryFrom<ComponentConfigToml> for ComponentConfig

A title is required, the card having nothing else to show. A description
past 80 characters is refused rather than cut, so an author finds the
limit when the directory loads instead of seeing a clipped card.
