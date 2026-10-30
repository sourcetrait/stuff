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

ratatui-image takes its `crossterm` feature alone, its defaults left off for
the `image` formats and the chafa encoder they would add; it finds the
terminal's graphics protocol and draws images through it. `image` is declared
with `bmp` and `png` alone, the two forms an icon comes in, the instance
ratatui-image builds against.
