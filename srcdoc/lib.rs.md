# lib.rs

The crate is a library behind a thin binary so its tests reach every item
through the hub; the binary's whole surface is `run`.

## mod r

ratatui's and ratatui-image's paths run long, so their items come through
this hub. crossterm is reached as `ratatui::crossterm`, never a direct
dependency, so the backend and the event types are the one instance ratatui
was built with.

## #[cfg(test)] use

The test-only names, `ComponentSpec`, `CatalogErrorKind`, and `Request`, are
used by the unit tests alone; in the main hub they warn as unused imports.

## Dependencies

ratatui-image takes its `crossterm` and `chafa-dyn` features, its defaults left
off for the `image` formats they would add. Where the terminal speaks no image
protocol, its halfblocks protocol draws an icon through chafa, which picks
among blocks, braille, and other symbols for a richer picture than half blocks
alone. `chafa-dyn` finds chafa 1.8.0 or newer through pkg-config and links it
dynamically: a build needs chafa's development files, Fedora's `chafa-devel`,
and a run its library, `chafa-libs`. `image` is declared with the `png`
feature alone, matching the instance ratatui-image builds against.
