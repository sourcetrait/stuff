# view.rs

## const PURPLE

The one highlight color: the cursor's card's border, the dialog's border, and
its focused button.

## const CARD_ROWS

A card is a row with a border of its own, its two lines inside, so it takes
four rows whether its border shows or not: moving the cursor redraws a border
and moves nothing.

## fn select

In the fancy look each card keeps its four rows. In the simple look
neighbouring cards share a border row, three rows a card, since only the
cursor's card draws its bars there.

## const INSET

The header and the preview sit in from the edge as far as a card's content,
so the header's icon lines up over the radio boxes.

## fn card

In the fancy look the row divides into the radio box and a column of the
title over the summary, `GAP` apart. A subcomponent's parts are indented a
radio box and its gap a level, so its radio box lines up under its parent's
title, the border still spanning the whole width.

## fn shade

In the simple look a card has no box: its title takes its mark's color,
blue added, green to add, red to remove, cyan to update, and an added component
carries
`[present]` after its title, whatever its mark, so what is in the home reads
plainly without a box to fill.

## fn image

The header's icon and a radio box draw the same way, an image through the
graphics protocol.

## fn border

A terminal that draws box-drawing characters from its font, rather than
filling their cells itself, can leave a gap between the rows of a vertical
line. The fancy look draws rounded borders; the simple look draws the top and
bottom alone, bars above and below the cursor's card, the dialog, and the
output.

## fn regions

The header, the body, and the legend, split once for the frame and once for
the loop, which bounds the preview's scroll by the body's rows.

## fn preview_lines

What will be installed, what will be uninstalled, then what will be updated,
each a heading, a sentence, a rule in the color of its action, and its
components in the order they run, a section with none left out. The app counts
the same lines to bound its scroll, the rule's width mattering only to the
drawing.

## fn confirm

The dialog draws over the preview after a clear, so the preview stays as its
backdrop.

## const YELLOW

The sudo dialog's highlight, as purple is every other's.

## const SUDO_TERMINAL

70 by 16 cells, sudo's longest normal exchange measured in a sysbox container:
its first-use lecture's ten lines, the widest 66 columns, then a prompt a try
and "Sorry, try again." after each wrong one, and "sudo: 3 incorrect password
attempts" when all three fail.

## fn health_page

`Health: <Title>` over nu's own expanded table of the record, its colors
kept, the table drawn to the body's width inside its insets, so a record reads
as nu shows a record.

## fn sudo_dialog

The newest lines show, the oldest scrolling away, and the dialog shrinks with
a smaller screen. A reversed cell after sudo's last line stands for the
terminal's cursor while sudo waits; the password never shows.
