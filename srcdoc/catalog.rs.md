# catalog.rs

## struct Catalog

### fn load

The directory is canonicalized first: every script gets absolute
`STUFF_DIR` and `COMPONENT_DIR`, whatever path the command was given.

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

## fn collect

A subdirectory is a subcomponent only when it holds a config.toml, so a
component may keep other directories of its own for its scripts. Siblings
are taken in name order, the order the list and the CLI show them.
