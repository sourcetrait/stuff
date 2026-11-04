# plan.rs

## struct Plan

### fn new

Removals run before additions: a removal can only take things away that an
addition might otherwise find in place. Anything a removal takes is left out
of the additions even if requested, since adding it would need what is being
removed.

Updates run last, so an update finds what the additions brought, each after what
it needs. Only the components named update: what one needs stays as it is, and
one absent or being removed is left out, as an addition skips what is added.

### fn needs_sudo

A step's present runs again after the step to confirm it, so a super present
makes its component's steps need sudo as a super action does.
