# icon.rs

## const ICON_SIZE

Four columns by two rows, about square in a typical font whose cells are
twice as tall as wide, and as tall as a card's two lines. The radio box takes
the same cells.

## const DEFAULT_ICON_PNG, const DEFAULT_LOGO_PNG

100x100 PNGs rendered with resvg from the SVGs beside them in `assets/icon/`;
the logo's SVG is sourcetrait's, from its brand repository. They and their
bmps are compiled in, so the binary needs no files of its own at run time.

## const DEFAULT_ICON_BMP, const DEFAULT_LOGO_BMP

Drawn by hand, pixel by pixel, with ImageMagick: the three dots six pixels
square, for inside a frame, and the logo's two slanted halves eight square.

## struct Radios

Each mark's radio box is drawn once, at load, and shared by every card.

## fn picker, fn background

The startup query also asks the terminal for its background color (OSC 11).
ratatui-image reports the answer as a capability and uses a picker's own
background only to pad an image that doesn't fill its cells.

## fn icon

The png where the terminal speaks a graphics protocol, else the bmp in
braille, else the default the same way, so a directory that ships only a bmp
looks the same in every terminal. An image is flattened onto the terminal's
background where it is known, and padded with it, so its empty pixels take
that color whatever the protocol does with alpha.
