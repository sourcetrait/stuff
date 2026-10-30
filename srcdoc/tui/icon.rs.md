# icon.rs

## const ICON_SIZE

Four columns by two rows, about square in a typical font whose cells are
twice as tall as wide, and as tall as a card's two lines. The view lays a row
out by its width, so the encoding and the layout agree on the icon's cells.

## const DEFAULT_ICON, const DEFAULT_LOGO

100x100 PNGs rendered with resvg from the SVGs beside them in `assets/icon/`;
the logo's SVG is sourcetrait's, from its brand repository. They are compiled
in, so the binary needs no files of its own at run time.

## fn picker, fn background

The startup query also asks the terminal for its background color (OSC 11).
ratatui-image reports the answer as a capability and uses a picker's own
background only to pad an image that doesn't fill its cells, while chafa's
encoding drops alpha, so a transparent pixel comes out as whatever color it
carries, usually black. A terminal that doesn't answer keeps that black.

## fn protocol

Each icon is encoded for the terminal once, at load, fitted to its cells; a
missing or unreadable PNG takes the default rather than leaving a gap. Where
the terminal's background is known, the icon is flattened onto it first and
padded with it, so its empty pixels take the terminal's own color.
