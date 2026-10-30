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

Drawn by hand, pixel by pixel, with ImageMagick, four pixels square: the three
dots, and the logo's slant.

## struct Radios

Each mark's radio box is made once, at load, and shared by every card.

## fn Icons::load

Images in the image tier alone, so a tier given on the command line draws the
same everywhere. A sixel, whose transparency is all or nothing, is flattened
onto the terminal's background where the terminal reports one; an image for
the kitty protocol keeps its alpha, since a terminal's reported background is
not always the color it shows, under a translucent window for one.

## fn picker, fn background

The startup query also asks the terminal for its background color (OSC 11).
ratatui-image reports the answer as a capability and uses a picker's own
background only to pad an image that doesn't fill its cells.

## fn icon

With images: the png, else the bmp, else the default png, so a directory that
ships only a bmp still shows its own icon. Without: the bmp in half-block
pixels, else the default bmp.
