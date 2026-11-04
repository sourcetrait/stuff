# look.rs

## fn detect

Terminals differ in what they draw well, so the TUI has two looks, chosen by
the graphics protocol the startup query finds, `--look` overriding. kitty's
protocol and iTerm2's place an image by cells, the terminal scaling it, so a
terminal speaking either gets the fancy look, its radio boxes and icon images.
A sixel is sized in pixels from the cell size the terminal reports, which has
come out wrong, so a sixel terminal gets the simple look with every terminal
that speaks no protocol.

## fn probe

Prints what the detection found, with the cell size from the terminal's answer
and the window size from the kernel, so the two can be held against each
other when an image comes out the wrong size.
