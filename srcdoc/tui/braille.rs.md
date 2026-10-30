# braille.rs

## fn Braille::new

The bitmap is fitted to its cells' dots by nearest neighbour, so a bitmap of
exactly that size maps pixel for pixel. A pixel counts as a dot unless it is
transparent or near black, the terminal's own dark showing through where a
bmp, which has no alpha, is black.
