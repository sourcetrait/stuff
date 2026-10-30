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

ratatui-image's default features pull chafa through pkg-config, which fails
the build without the C library installed; only its `crossterm` feature is
taken. `image` is declared with the `png` feature alone, matching the
instance ratatui-image builds against.
