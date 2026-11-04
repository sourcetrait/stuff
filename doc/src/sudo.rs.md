# sudo.rs

No script runs under sudo: a script runs as the user, in the environment it
relies on, and puts sudo in front of its one command needing root, calling
`sudo -v` shortly before it. Stuff's part is the credential alone, so a
script's sudo finds it cached: sudo keeps its timestamp per terminal, and
every script runs on Stuff's terminal, measured carrying from a `sudo -v` to
a bash child's and a nu child's `sudo -n` in a sysbox container.

## struct Secret

The password is zeroed when dropped and never printed by Debug, the TUI's
state deriving Debug.

## struct Validation

`sudo -S` reads the password from stdin and writes its prompt and messages to
stderr, so the dialog's mini terminal shows sudo's own exchange: its first-use
lecture, its prompt, "Sorry, try again.", and its last word. sudo echoes no
newline after a password it reads from a pipe; the dialog adds one on enter.
With the credential cached, `sudo -S -v` exits 0 at once and writes nothing,
which is how a refresh before a step stays unseen.

### fn start

The exit is sent after the last of stderr, the thread reading stderr to its
end before it waits, so the dialog has sudo's last line before it learns how
sudo ended. A sudo that fails to start is reported in the dialog the same way.

Dropping the validation closes sudo's stdin, and `sudo -S` ends at the end of
its input: leaving the dialog needs no signal.
