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
the summary, `GAP` apart. A subcomponent's parts are indented its depth times
four columns inside the row, the border still spanning the whole width. The
tests find the radio box from the title by those widths.

## fn radio

The box is the card's two lines tall and four columns wide, about square in a
typical font. An absent component's is an outline of one-eighth blocks, its
corners from Symbols for Legacy Computing: they sit on the cells' outer edges,
so the outline is as large as a filled box, where box-drawing lines run through
the cells' middles and would draw it half the size. A marked one is filled in
its mark's color.

## fn confirm

The dialog draws over the preview after a clear, so the preview stays as its
backdrop.
