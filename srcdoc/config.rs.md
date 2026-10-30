# config.rs

## struct StuffConfigToml, struct ComponentConfigToml

Unknown keys are allowed, against the estate's usual denial: a config.toml
is shared with its scripts, which may keep any keys of their own there and
read them through `COMPONENT_DIR` or `STUFF_DIR`.

## impl TryFrom<ComponentConfigToml> for ComponentConfig

A title is required, the card having nothing else to show. A description
past 80 characters is refused rather than cut, so an author finds the
limit when the directory loads instead of seeing a clipped card.
