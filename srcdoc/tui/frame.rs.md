# frame.rs

## fn radio_image, fn framed_image

Each shape is a signed distance to a rounded square, a pixel's coverage taken
from the distance at its center, so the edges come out antialiased at any
size the terminal scales the image to. The radio box is a border, a moat, and
a fill, a radio button's ring and dot made square; an icon's frame is the same
border with the icon fitted to its inside, touching it, its corners clipped to
the border's curve.

## fn radio_dots, fn framed_dots

The same shapes in braille, eight dots by eight over an icon's four cells by
two: the ring drops its four corner dots to round them. A braille cell has
one color, so a cell holding ring and fill dots takes their average.
