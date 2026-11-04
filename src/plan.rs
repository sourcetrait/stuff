//! A plan: the removals, additions, and updates a selection needs, in the order they run.

use crate::*;

/// What a step does to its component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepAction {
    /// Calls the component's add.
    Add,
    /// Calls the component's remove.
    Remove,
    /// Calls the component's update.
    Update,
}

impl StepAction {
    /// The action the step calls.
    pub(crate) fn action(self) -> Action {
        match self {
            Self::Add => Action::Add,
            Self::Remove => Action::Remove,
            Self::Update => Action::Update,
        }
    }

    /// The step's label, as the preview shows it.
    pub(crate) fn label(self) -> &'static str {
        self.action().name()
    }

    /// Whether the component is added once the step has run.
    pub(crate) fn leaves_added(self) -> bool {
        self != Self::Remove
    }
}

/// One step: an action on a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Step {
    /// What it does.
    pub(crate) action: StepAction,
    /// Its component, by index.
    pub(crate) component: usize,
}

/// The steps in order: removals, dependents first; additions, requirements first; then updates,
/// requirements first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Plan {
    /// The steps, in the order they run.
    pub(crate) steps: Vec<Step>,
}

impl Plan {
    /// The plan adding `add` and removing `remove`, with what each needs, and updating `update`,
    /// those added that stay.
    pub(crate) fn new(catalog: &Catalog, added: &[bool], add: &[usize], remove: &[usize], update: &[usize]) -> Self {
        let removing: Vec<usize> =
            catalog.remove_closure(remove).into_iter().filter(|&i| added[i]).collect();
        let adding: Vec<usize> = catalog
            .add_closure(add)
            .into_iter()
            .filter(|&i| !added[i] && !removing.contains(&i))
            .collect();
        let updating: Vec<usize> = catalog
            .add_closure(update)
            .into_iter()
            .filter(|&i| update.contains(&i) && added[i] && !removing.contains(&i))
            .collect();
        let step = |action: StepAction| move |component: usize| Step { action, component };
        let steps = removing
            .into_iter()
            .map(step(StepAction::Remove))
            .chain(adding.into_iter().map(step(StepAction::Add)))
            .chain(updating.into_iter().map(step(StepAction::Update)))
            .collect();
        Self { steps }
    }

    /// Whether the plan does nothing.
    pub(crate) fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Whether the plan needs sudo: a step's action is super, or its component's present, which
    /// runs again after the step.
    pub(crate) fn needs_sudo(&self, catalog: &Catalog) -> bool {
        self.steps.iter().any(|step| {
            let supers = catalog.components[step.component].supers;
            supers.has(step.action.action()) || supers.present
        })
    }
}
