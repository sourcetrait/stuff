# frame.rs

## fn radio_image

Each shape is a signed distance to a rounded square, a pixel's coverage taken
from the distance at its center, so the edges come out antialiased. The radio
box is a border, a moat, and a fill, a radio button's ring and dot made
square.

## const SIDE

ratatui-image fits an image into its cells by scaling it down, never up, so
the image starts larger than any cell size makes its cells.
