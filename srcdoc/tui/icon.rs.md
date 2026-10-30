# icon.rs

## const ICON_SIZE

Four columns by two rows, about square in a typical font whose cells are
twice as tall as wide, and as tall as a card's two lines.

## const DEFAULT_ICON, const DEFAULT_LOGO

100x100 PNGs rendered with resvg from the SVGs beside them in `assets/icon/`;
the logo's SVG is sourcetrait's, from its brand repository. They are compiled
in, so the binary needs no files of its own at run time.

## fn protocol

Each icon is encoded for the terminal once, at load, fitted to its cells; a
missing or unreadable PNG takes the default rather than leaving a gap.
