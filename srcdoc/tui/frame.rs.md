# frame.rs

## fn radio_image, fn framed_image

Each shape is a signed distance to a rounded square, a pixel's coverage taken
from the distance at its center, so the edges come out antialiased. The radio
box is a border, a moat, and a fill, a radio button's ring and dot made
square; an icon's frame is the same border with the icon fitted to its inside,
touching it, its corners clipped to the border's curve.

## const SIDE, fn enlarged

ratatui-image fits an image into its cells by scaling it down, never up, so
every image starts larger than any cell size makes its cells, and a bitmap is
enlarged pixel for pixel to that size first.
