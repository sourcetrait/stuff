# source.rs

The source comes first on the line and takes one of three forms, told apart by
how it is written and never by what happens to sit on the disk, so a line means
the same wherever it runs.

## enum Source

- A URI begins with one of the four schemes or reads as git's `user@host:path`,
  the form people type for ssh.
- A path holds a `/` or starts with `.` or `~`. It is never tried as an alias, so
  a mistyped path is reported as the path it is.
- Anything else is a plain word, always an alias, even where a directory of that
  name sits in the current folder. `./rust` names that directory.

## fn Source::directory

A word's alias is parsed again as a source, so it may name a URI or a path. An
alias naming another word is refused, which leaves no chain to walk and no cycle
to guard against.

## fn aliases

`alias.toml` is a table of words to sources, nothing else being kept there. A
missing file is no aliases rather than a failure, and the refusal names the file
looked in, so a person learns where an alias goes.

## fn fetch

A URI names a git remote: a static directory over HTTP can't be listed, and git
is on every system Stuff serves. Its head is brought up on every run, as what it
fetched is what runs next, and a stale copy is never used after a failed fetch.
The directory kept for it is a cache, reset and cleaned, so nothing a component
wrote into it outlasts the next run.

A clone lands in a `.partial` directory and is renamed into place, so one cut off
part way leaves nothing later taken for stuff. Two fetches of one URI wait on each
other by a lock file beside it, which its holder's end releases: each would
otherwise take the other's clone in progress for one cut off, and remove it.

Everything Stuff and git say goes to stderr, each git command told `-q`: stdout
stays what agents read, a list or a health record.
