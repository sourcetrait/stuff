# view.rs

## const PURPLE

The one highlight color: the cursor's card's border, the dialog's border, and
its focused button.

## const CARD_ROWS

A card is a row with a border of its own, its two lines inside, so it takes
four rows whether its border shows or not: moving the cursor redraws a border
and moves nothing.

## const INSET

The header and the preview sit in from the edge as far as a card's content,
so the header's icon lines up over the radio boxes.

## fn card

The row divides into the radio box, the icon, and a column of the title over
the summary, `GAP` apart. A subcomponent's parts are indented a radio box and
its gap a level, so its radio box lines up under its parent's icon, the border
still spanning the whole width.

## fn radio

## fn icon

An icon and a radio box draw the same way: an image through the graphics
protocol, half-block pixels, or a box filled with a color.

## fn border

A terminal that draws box-drawing characters from its font, rather than
filling their cells itself, can leave a gap between the rows of a vertical
line. The image and lines tiers draw rounded lines; the plain tier draws no
line character, tinting the cursor's row and the dialog instead, a cell's
background filling the cell in every terminal.

## fn confirm

The dialog draws over the preview after a clear, so the preview stays as its
backdrop.
