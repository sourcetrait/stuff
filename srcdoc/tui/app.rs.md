# app.rs

## enum Look

Two looks, named for what they are. Some terminals draw images and lines
well and some don't, and each look does its best for its kind: pretty with
rounded lines and images, ugly with braille dots.

## struct App

The explicit marks, `adds` and `removes`, are what the user toggled; the
marks shown are recomputed from them through the closures after every
change, so what a mark carries with it is always current.

### fn toggle

Turning off a mark another mark carries turns off the marks that carry it:
unmarking rust's addition unmarks nightly's. Marking an addition drops any
removal that would take something it needs, and marking a removal drops any
addition that needs what it takes, so the two sets never conflict.

### fn key

Escape on the dialog is cancel, back to the preview; enter on a focused
cancel does the same. The run screen holds its keys until the run ends, a
script being in progress.

### fn refresh

After a run, every exists runs again rather than trusting the plan, and the
cursor stays on the component it was on.

## fn run

The terminal is entered before the picker queries it, the query needing raw
mode. A terminal that answers with a graphics protocol gets the pretty look
and any other the ugly one, unless the command names a look. ratatui's `init`
installs the panic hook that restores the terminal.
