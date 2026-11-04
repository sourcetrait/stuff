use crate::*;

use super::catalog::{catalog, catalog_with_supers};

fn step(action: StepAction, component: usize) -> Step {
    Step { action, component }
}

#[test]
fn an_addition_brings_what_it_needs_first() {
    let plan = Plan::new(&catalog(), &[false; 5], &[4], &[], &[]);
    let expected = [1, 2, 0, 4].map(|i| step(StepAction::Add, i));
    assert_eq!(plan.steps, expected);
}

#[test]
fn an_addition_skips_what_is_added() {
    let plan = Plan::new(&catalog(), &[true, true, false, false, false], &[4], &[], &[]);
    let expected = [2, 4].map(|i| step(StepAction::Add, i));
    assert_eq!(plan.steps, expected);
}

#[test]
fn a_removal_takes_what_needs_it_first_and_skips_the_absent() {
    let plan = Plan::new(&catalog(), &[true, true, true, false, true], &[], &[1], &[]);
    let expected = [4, 2, 1].map(|i| step(StepAction::Remove, i));
    assert_eq!(plan.steps, expected);
}

#[test]
fn removals_run_before_additions() {
    let plan = Plan::new(&catalog(), &[true, true, false, false, false], &[3], &[0], &[]);
    assert_eq!(plan.steps, vec![step(StepAction::Remove, 0), step(StepAction::Add, 3)]);
}

#[test]
fn an_update_takes_only_what_is_named_and_added() {
    let plan = Plan::new(&catalog(), &[true, true, true, false, false], &[], &[], &[2, 3]);
    assert_eq!(plan.steps, vec![step(StepAction::Update, 2)], "rust is required, not named; riscv is absent");
}

#[test]
fn updates_run_last_each_after_what_it_needs() {
    let plan = Plan::new(&catalog(), &[true, true, true, false, true], &[3], &[], &[4, 2]);
    assert_eq!(
        plan.steps,
        vec![step(StepAction::Add, 3), step(StepAction::Update, 2), step(StepAction::Update, 4)],
        "zed needs nightly"
    );
}

#[test]
fn a_removal_drops_an_update_of_what_it_takes() {
    let plan = Plan::new(&catalog(), &[true, true, true, false, false], &[], &[1], &[2]);
    assert!(!plan.steps.iter().any(|s| s.action == StepAction::Update), "{:?}", plan.steps);
}

#[test]
fn only_a_removal_leaves_its_component_absent() {
    assert!(StepAction::Add.leaves_added());
    assert!(StepAction::Update.leaves_added());
    assert!(!StepAction::Remove.leaves_added());
}

#[test]
fn a_plan_without_super_actions_needs_no_sudo() {
    let catalog = catalog();
    assert!(!Plan::new(&catalog, &[false; 5], &[4], &[], &[]).needs_sudo(&catalog));
}

#[test]
fn a_super_action_among_the_steps_needs_sudo() {
    let catalog = catalog_with_supers("git", Actions { add: true, ..Actions::default() });
    assert!(Plan::new(&catalog, &[false; 5], &[4], &[], &[]).needs_sudo(&catalog));
    assert!(!Plan::new(&catalog, &[true; 5], &[], &[0], &[]).needs_sudo(&catalog), "git's remove is plain");
    assert!(!Plan::new(&catalog, &[true; 5], &[], &[], &[0]).needs_sudo(&catalog), "git's update is plain");
}

#[test]
fn a_super_update_needs_sudo() {
    let catalog = catalog_with_supers("git", Actions { update: true, ..Actions::default() });
    assert!(Plan::new(&catalog, &[true; 5], &[], &[], &[0]).needs_sudo(&catalog));
}

#[test]
fn a_super_present_makes_its_steps_need_sudo() {
    let catalog = catalog_with_supers("git", Actions { present: true, ..Actions::default() });
    assert!(Plan::new(&catalog, &[true; 5], &[], &[0], &[]).needs_sudo(&catalog));
    assert!(!Plan::new(&catalog, &[false; 5], &[3], &[], &[]).needs_sudo(&catalog), "git takes no step");
}
