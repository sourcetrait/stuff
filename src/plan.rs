//! A plan: the removals and additions a selection needs, in the order they run.

use crate::*;

/// What a step does to its component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepAction {
    /// Runs the component's add script.
    Add,
    /// Runs the component's remove script.
    Remove,
}

impl StepAction {
    /// The script the step runs.
    pub(crate) fn script(self) -> Script {
        match self {
            Self::Add => Script::Add,
            Self::Remove => Script::Remove,
        }
    }

    /// The step's label, as the preview shows it.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Remove => "remove",
        }
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

/// The steps in order: removals, dependents first; then additions, requirements first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Plan {
    /// The steps, in the order they run.
    pub(crate) steps: Vec<Step>,
}

impl Plan {
    /// The plan adding `add` and removing `remove`, with what each needs.
    pub(crate) fn new(catalog: &Catalog, added: &[bool], add: &[usize], remove: &[usize]) -> Self {
        let removing: Vec<usize> =
            catalog.remove_closure(remove).into_iter().filter(|&i| added[i]).collect();
        let adding: Vec<usize> = catalog
            .add_closure(add)
            .into_iter()
            .filter(|&i| !added[i] && !removing.contains(&i))
            .collect();
        let steps = removing
            .into_iter()
            .map(|component| Step { action: StepAction::Remove, component })
            .chain(adding.into_iter().map(|component| Step { action: StepAction::Add, component }))
            .collect();
        Self { steps }
    }

    /// Whether the plan does nothing.
    pub(crate) fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Whether the plan needs sudo: a step's script is super, or its component's present script,
    /// which runs again after the step.
    pub(crate) fn needs_sudo(&self, catalog: &Catalog) -> bool {
        self.steps.iter().any(|step| {
            let supers = catalog.components[step.component].supers;
            supers.of(step.action.script()) || supers.present
        })
    }
}
