# icon.rs

## const ICON_SIZE

Four columns by two rows, about square in a typical font whose cells are
twice as tall as wide, and as tall as a card's two lines. The radio box takes
the same cells.

## const DEFAULT_ICON, const DEFAULT_LOGO

100x100 PNGs rendered with resvg from the SVGs beside them in `assets/icon/`;
the logo's SVG is sourcetrait's, from its brand repository. They are compiled
in, so the binary needs no files of its own at run time.

## struct Radios

Each mark's radio box is made once, at load, and shared by every card.

## fn Icons::load

Images in the image tier alone, so a tier given on the command line draws the
same everywhere; below it the TUI draws no icon or box. A sixel, whose
transparency is all or nothing, is flattened onto the terminal's background
where the terminal reports one; an image for the kitty protocol keeps its
alpha, since a terminal's reported background is not always the color it
shows, under a translucent window for one.

## fn picker, fn background

The startup query also asks the terminal for its background color (OSC 11).
ratatui-image reports the answer as a capability and uses a picker's own
background only to pad an image that doesn't fill its cells.

## fn icon

The png, else the bmp, else the default, so a directory that ships only a bmp
still shows its own icon; a bmp is enlarged pixel for pixel.
