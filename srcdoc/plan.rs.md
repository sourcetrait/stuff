# plan.rs

## struct Plan

### fn new

Removals run before additions: a removal can only take things away that an
addition might otherwise find in place. Anything a removal takes is left out
of the additions even if requested, since adding it would need what is being
removed.

### fn needs_sudo

A step's present script runs again after the step to confirm it, so a super
present script makes its component's steps need sudo as a super add or remove
does.
