# catalog.rs

## struct Catalog

### fn load

The directory is canonicalized first, so every component's overlay takes an
absolute path, whatever path the command was given.

### fn from_specs

A spec's parent must precede it, which `collect`'s depth-first walk
guarantees; the check refuses a hand-built spec list that breaks it. A
`depends` naming no component, and any cycle through parents and
`depends` together, refuse the whole directory: a plan over a broken graph
would add or remove in an order no one chose.

### fn add_closure, fn remove_closure

A depth-first post-order over requirements, the parent and `depends`, gives
each component after what it needs; over dependents, its children and
whatever names it in `depends`, each before what it needs. The order is the
run order, so a plan needs no separate sort.

### fn display_order

The added float to the top among their siblings and nowhere else, so a
subcomponent always stays under its parent.

### fn check_acyclic

Kahn's algorithm over the same edges; whatever is left pending is named in
the error.

### fn super_present

Every present runs when the TUI opens, so one super present makes the first
check wait on the sudo dialog.

## fn collect

A subdirectory is a subcomponent only when it holds a component.toml, so a
component may keep other directories of its own for its module. Siblings
are taken in name order, the order the list and the CLI show them.

mod.nu is the one required file beside the component.toml. What it offers is
the component.toml's `actions`, which reads no nushell, so the graph loads
without starting nu; {runner.rs} holds the two together. A health action brings
health.nutype with it, the record's type being its reader's only description of
it.
