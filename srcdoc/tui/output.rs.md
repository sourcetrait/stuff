# output.rs

## struct Output

### fn push

A carriage return marks the open line for rewriting by the next byte, so a
progress bar's updates replace each other as in a terminal, while a CRLF
still ends its line intact. Bytes arrive in arbitrary pieces from the pipes,
so the state spans pushes.

### fn tail

Each line is converted from ANSI on its own; a color left open across a line
end does not carry into the next.
