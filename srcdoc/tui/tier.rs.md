# tier.rs

## fn detect

Terminals differ in what they draw well, so the TUI draws in tiers, chosen by
the graphics protocol the startup query finds, `--tier` overriding. kitty's
protocol and iTerm2's place an image by cells, the terminal scaling it, so the
image tier draws its radio boxes and icon there. A sixel is sized in pixels
from the cell size the terminal reports, which has come out wrong, so a sixel
terminal draws in the lines tier, without images. A terminal with no protocol
draws in the plain tier, whose borders are horizontal bars alone, since some
terminals break a vertical line between rows.

## fn probe

Prints what the detection found, with the cell size from the terminal's answer
and the window size from the kernel, so the two can be held against each
other when an image comes out the wrong size.
