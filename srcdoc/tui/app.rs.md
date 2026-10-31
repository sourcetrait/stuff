# app.rs

## enum Look

Terminals draw images and lines with different success, so the TUI has two
looks, fancy and simple, the same row structure in each (look.rs).

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

A key is made its arrow first: `w` `a` `s` `d`, helix's `k` `h` `j` `l`, and
the arrows move alike, and `q` is escape on every screen, so each screen
matches arrows, enter, escape, space, and tab alone. The preview scrolls until
its last line sits at the body's bottom, the body's rows taken from the last
frame drawn.

Escape on the dialog is cancel, back to the preview; enter on a focused
cancel does the same. The run screen holds its keys until the run ends, a
script being in progress.

While the sudo dialog shows, every key but enter and escape is the password's,
`q` and w a s d included, a password holding any of them; escape leaves it as
it leaves every screen. A refresh's dialog stays hidden until sudo writes, and
the keys wait meanwhile.

### fn leave_sudo

Before a run, leaving returns to the preview, nothing having started; between
steps it ends the run as failed, the next step needing the credential; before
the first check it leaves the TUI, the list having nothing to show; before a
later check it stays on the finished run.

### fn event, fn next

The TUI drives the run a step at a time, so where the plan needs sudo the
credential is refreshed before each step, whether that step's scripts use it
or not, and a lapsed one asks in the dialog. The first step's refresh is the
dialog shown before anything starts.

### fn refresh

After a run, every present script runs again rather than trusting the plan, and the
cursor stays on the component it was on.

## struct SudoDialog

A refresh with the credential cached writes nothing, so it never shows; one
whose credential lapsed shows once sudo asks.

## fn run

The terminal is entered before the picker queries it, and the look is settled
before the icons load, since it decides whether they are drawn. ratatui's
`init` installs the panic hook that restores the terminal. Where a present
script is super, the first check waits on the dialog, its validation the
loop's first request.

## fn serve

The password moves from the dialog to sudo's stdin by a take, so the typed
copy is cleared once sent.
