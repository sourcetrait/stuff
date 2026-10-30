# tier.rs

## fn detect

Terminals differ in what they draw well, so the TUI draws in tiers. A graphics
protocol, which the startup query finds, gives the image tier. No query finds
whether a terminal draws its line characters itself, cell-exact, or from its
font, which can leave gaps between the rows of a vertical line; a terminal
known to draw its own, by a variable it sets or the name in `TERM` or
`TERM_PROGRAM`, gives the lines tier, any other the plain tier. `--tier`
overrides.

## fn probe

Prints what the detection found, with the cell size from the terminal's answer
and the window size from the kernel, so the two can be held against each
other when an image comes out the wrong size.
