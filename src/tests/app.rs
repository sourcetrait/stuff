use crate::*;

use super::catalog::{catalog, catalog_with, catalog_with_supers};

fn app(added: [bool; 5]) -> App {
    App::new(catalog(), added.to_vec())
}

fn at(app: &mut App, id: &str) {
    let i = app.catalog.components.iter().position(|c| c.id == id).expect("a component");
    app.cursor = app.order.iter().position(|&o| o == i).expect("listed");
}

fn marks(app: &App) -> Vec<Mark> {
    app.marks.clone()
}

/// Marks a component, then walks the preview and the dialog to proceed.
fn proceed(app: &mut App, id: &str) -> Option<Request> {
    at(app, id);
    app.toggle();
    app.key(r::KeyCode::Enter);
    app.key(r::KeyCode::Enter);
    app.key(r::KeyCode::Char('d'));
    app.key(r::KeyCode::Enter)
}

/// The fixture, nothing added, git's add super.
fn super_add() -> App {
    App::new(catalog_with_supers("git", Actions { add: true, ..Actions::default() }), vec![false; 5])
}

#[test]
fn marking_an_addition_marks_what_it_needs() {
    let mut app = app([false; 5]);
    at(&mut app, "zed");
    app.toggle();
    use Mark::{Absent, ToAdd};
    assert_eq!(marks(&app), [ToAdd, ToAdd, ToAdd, Absent, ToAdd]);
}

#[test]
fn unmarking_a_requirement_unmarks_what_needed_it() {
    let mut app = app([false; 5]);
    at(&mut app, "zed");
    app.toggle();
    at(&mut app, "rust");
    app.toggle();
    assert_eq!(marks(&app), [Mark::Absent; 5]);
    assert!(app.adds.is_empty());
}

#[test]
fn marking_a_removal_marks_what_needs_it() {
    let mut app = app([true, true, true, true, true]);
    at(&mut app, "rust");
    app.toggle();
    use Mark::{Added, ToRemove};
    assert_eq!(marks(&app), [Added, ToRemove, ToRemove, ToRemove, ToRemove]);
}

#[test]
fn an_addition_drops_a_removal_it_conflicts_with() {
    let mut app = app([true, true, false, false, false]);
    at(&mut app, "rust");
    app.toggle();
    at(&mut app, "rust/riscv");
    app.toggle();
    assert!(app.removes.is_empty());
    use Mark::{Absent, Added, ToAdd};
    assert_eq!(marks(&app), [Added, Added, Absent, ToAdd, Absent]);
}

#[test]
fn enter_with_nothing_marked_stays_on_the_list() {
    let mut app = app([false; 5]);
    assert_eq!(app.key(r::KeyCode::Enter), None);
    assert_eq!(app.screen, Screen::Select);
}

#[test]
fn proceeding_walks_the_preview_and_the_dialog() {
    let mut app = app([false; 5]);
    at(&mut app, "git");
    app.key(r::KeyCode::Char(' '));
    app.key(r::KeyCode::Enter);
    assert_eq!(app.screen, Screen::Preview);
    app.key(r::KeyCode::Enter);
    assert_eq!((app.screen, app.button), (Screen::Confirm, Button::Cancel));
    app.key(r::KeyCode::Enter);
    assert_eq!(app.screen, Screen::Preview, "cancel is the default");
    app.key(r::KeyCode::Enter);
    app.key(r::KeyCode::Char('d'));
    assert_eq!(app.key(r::KeyCode::Enter), Some(Request::Step(0)));
    assert_eq!(app.screen, Screen::Run);
    assert!(app.sudo.is_none(), "a plan without super actions needs no sudo");
}

#[test]
fn a_and_d_move_to_the_parent_and_the_first_child() {
    let mut app = app([false; 5]);
    at(&mut app, "rust");
    app.key(r::KeyCode::Char('d'));
    assert_eq!(app.catalog.components[app.current().expect("a component")].id, "rust/nightly");
    app.key(r::KeyCode::Char('a'));
    assert_eq!(app.catalog.components[app.current().expect("a component")].id, "rust");
}

#[test]
fn each_step_follows_the_last_until_the_run_ends() {
    let mut app = app([false; 5]);
    assert_eq!(proceed(&mut app, "rust/riscv"), Some(Request::Step(0)));
    app.event(RunEvent::Begin { step: 0 });
    assert_eq!(app.event(RunEvent::Done { step: 0 }), Some(Request::Step(1)));
    assert_eq!(app.event(RunEvent::Done { step: 1 }), None);
    assert!(!app.running);
}

#[test]
fn a_finished_run_returns_to_a_refreshed_list() {
    let mut app = app([false; 5]);
    proceed(&mut app, "git");
    app.event(RunEvent::Begin { step: 0 });
    app.event(RunEvent::Output { bytes: b"installing\n".to_vec() });
    assert_eq!(app.key(r::KeyCode::Enter), None, "a running run holds the screen");
    app.event(RunEvent::Done { step: 0 });
    assert_eq!(app.key(r::KeyCode::Enter), Some(Request::Check));
    app.refresh(vec![true, false, false, false, false]);
    assert_eq!(app.screen, Screen::Select);
    assert_eq!(app.marks[0], Mark::Added);
    assert_eq!(app.catalog.components[app.current().expect("a component")].id, "git");
}

#[test]
fn a_plan_needing_sudo_shows_the_dialog_before_its_first_step() {
    let mut app = super_add();
    assert_eq!(proceed(&mut app, "git"), Some(Request::Validate));
    assert_eq!(app.screen, Screen::Run);
    let sudo = app.sudo.as_ref().expect("the dialog");
    assert!(sudo.shown);
    assert_eq!(sudo.then, Then::Step(0));
}

#[test]
fn the_dialog_takes_every_key_but_enter_and_escape_as_the_password() {
    let mut app = super_add();
    proceed(&mut app, "git");
    app.event(RunEvent::SudoOutput { bytes: b"[sudo] password for test: ".to_vec() });
    for c in ['q', 'w', 'a', 's', 'd'] {
        assert_eq!(app.key(r::KeyCode::Char(c)), None);
    }
    app.key(r::KeyCode::Backspace);
    assert_eq!(app.key(r::KeyCode::Enter), Some(Request::Send));
    assert_eq!(app.screen, Screen::Run);
    let sudo = app.sudo.as_mut().expect("the dialog");
    assert_eq!(sudo.password.take().expose(), "qwas");
    let screen = sudo.terminal.screen();
    assert_eq!(screen.len(), 2, "enter ends the prompt's line");
    assert_eq!(screen[0], b"[sudo] password for test: ", "the password never shows");
}

#[test]
fn the_credential_starts_the_step_it_was_asked_for() {
    let mut app = super_add();
    proceed(&mut app, "git");
    assert_eq!(app.event(RunEvent::SudoExit { ok: true }), Some(Request::Step(0)));
    assert!(app.sudo.is_none());
}

#[test]
fn a_failed_sudo_holds_the_dialog_until_escape_returns_to_the_preview() {
    let mut app = super_add();
    proceed(&mut app, "git");
    app.event(RunEvent::SudoOutput { bytes: b"sudo: 3 incorrect password attempts\n".to_vec() });
    assert_eq!(app.event(RunEvent::SudoExit { ok: false }), None);
    assert!(app.sudo.as_ref().is_some_and(|sudo| sudo.failed));
    assert_eq!(app.key(r::KeyCode::Enter), None, "sudo has ended");
    assert_eq!(app.key(r::KeyCode::Esc), Some(Request::Cancel));
    assert!(app.sudo.is_none());
    assert_eq!(app.screen, Screen::Preview);
    assert!(!app.running);
}

#[test]
fn each_later_step_refreshes_the_credential_unseen_unless_sudo_asks() {
    let mut app = super_add();
    proceed(&mut app, "zed");
    assert_eq!(app.event(RunEvent::SudoExit { ok: true }), Some(Request::Step(0)));
    assert_eq!(app.event(RunEvent::Done { step: 0 }), Some(Request::Validate));
    let sudo = app.sudo.as_ref().expect("a refresh");
    assert!(!sudo.shown);
    assert_eq!(sudo.then, Then::Step(1));
    assert_eq!(app.key(r::KeyCode::Char('x')), None, "keys wait while a refresh runs unseen");
    assert_eq!(app.sudo.as_ref().map(|sudo| sudo.password.expose().len()), Some(0));
    app.event(RunEvent::SudoOutput { bytes: b"[sudo] password for test: ".to_vec() });
    assert!(app.sudo.as_ref().is_some_and(|sudo| sudo.shown), "sudo asking shows the dialog");
}

#[test]
fn escape_from_a_later_dialog_ends_the_run() {
    let mut app = super_add();
    proceed(&mut app, "zed");
    app.event(RunEvent::SudoExit { ok: true });
    app.event(RunEvent::Begin { step: 0 });
    app.event(RunEvent::Done { step: 0 });
    app.event(RunEvent::SudoOutput { bytes: b"[sudo] password for test: ".to_vec() });
    assert_eq!(app.key(r::KeyCode::Esc), Some(Request::Cancel));
    assert_eq!(app.screen, Screen::Run);
    assert!(!app.running);
    assert!(app.failure.is_some());
}

#[test]
fn a_super_present_check_waits_on_the_dialog_and_escape_leaves() {
    let present = Actions { present: true, ..Actions::default() };
    let mut app = App::unchecked(catalog_with_supers("git", present));
    assert!(!app.checked);
    let sudo = app.sudo.as_ref().expect("the dialog");
    assert!(sudo.shown);
    assert_eq!(sudo.then, Then::Check);
    assert_eq!(app.key(r::KeyCode::Esc), Some(Request::Quit));
}

#[test]
fn a_removal_without_a_remove_action_is_skipped_and_warned_of_at_the_end() {
    let catalog = catalog_with("rust/riscv", |spec| spec.config.actions.remove = false);
    let mut app = App::new(catalog, vec![false, true, true, true, false]);
    assert_eq!(proceed(&mut app, "rust"), Some(Request::Step(0)), "nightly goes first");
    app.event(RunEvent::Begin { step: 0 });
    assert_eq!(app.event(RunEvent::Done { step: 0 }), Some(Request::Step(2)), "riscv is passed over");
    app.event(RunEvent::Begin { step: 2 });
    assert_eq!(app.event(RunEvent::Done { step: 2 }), None);
    assert!(!app.running);
    assert!(app.failure.is_none(), "a warning, not a failure");
    let text: Vec<String> = app.output.lines().iter().map(|line| String::from_utf8_lossy(line).into_owned()).collect();
    assert!(text.iter().any(|line| line == "\x1b[33m[stuff]\x1b[0m remove not supported: rust/riscv"), "{text:?}");
}

#[test]
fn a_successful_run_says_a_fresh_login_is_needed_as_its_last_line() {
    let mut app = app([false; 5]);
    proceed(&mut app, "git");
    app.event(RunEvent::Begin { step: 0 });
    app.event(RunEvent::Done { step: 0 });
    let last = app.output.lines().last().map(|line| String::from_utf8_lossy(line).into_owned());
    assert_eq!(last.as_deref(), Some("\x1b[34m[sysbox]\x1b[0m Environment changes require a fresh terminal login"));
}

#[test]
fn a_failed_run_says_nothing_of_a_fresh_login() {
    let mut app = app([false; 5]);
    proceed(&mut app, "git");
    app.event(RunEvent::Begin { step: 0 });
    app.event(RunEvent::Fail { message: "git add: exited 1".to_string() });
    assert!(!app.running);
    let text: Vec<String> = app.output.lines().iter().map(|line| String::from_utf8_lossy(line).into_owned()).collect();
    assert!(!text.iter().any(|line| line.contains("fresh terminal login")), "{text:?}");
}

/// The fixture, git added and offering update.
fn updatable() -> App {
    App::new(catalog_with("git", |spec| spec.config.actions.update = true), vec![true, false, false, false, false])
}

#[test]
fn u_marks_an_added_component_to_update_and_takes_the_mark_off() {
    let mut app = updatable();
    at(&mut app, "git");
    assert_eq!(app.key(r::KeyCode::Char('u')), None);
    assert_eq!(app.marks[0], Mark::ToUpdate);
    app.key(r::KeyCode::Char('u'));
    assert_eq!(app.marks[0], Mark::Added);
}

#[test]
fn u_does_nothing_where_the_component_does_not_offer_it_or_is_absent() {
    let mut app = updatable();
    at(&mut app, "rust");
    app.key(r::KeyCode::Char('u'));
    assert!(app.updates.is_empty(), "rust doesn't offer it");
    let mut app = App::new(catalog_with("git", |spec| spec.config.actions.update = true), vec![false; 5]);
    at(&mut app, "git");
    app.key(r::KeyCode::Char('u'));
    assert!(app.updates.is_empty(), "git is absent");
}

#[test]
fn space_takes_an_update_mark_off_and_a_removal_takes_the_update_with_it() {
    let mut app = updatable();
    at(&mut app, "git");
    app.key(r::KeyCode::Char('u'));
    app.key(r::KeyCode::Char(' '));
    assert_eq!(app.marks[0], Mark::Added);
    assert!(app.updates.is_empty());
    app.key(r::KeyCode::Char(' '));
    assert_eq!(app.marks[0], Mark::ToRemove);
    app.key(r::KeyCode::Char('u'));
    assert_eq!(app.marks[0], Mark::ToRemove, "a removal isn't an update");
}

#[test]
fn an_update_mark_makes_an_update_step() {
    let mut app = updatable();
    at(&mut app, "git");
    app.key(r::KeyCode::Char('u'));
    app.key(r::KeyCode::Enter);
    assert_eq!(app.screen, Screen::Preview);
    assert_eq!(app.plan.steps, vec![Step { action: StepAction::Update, component: 0 }]);
    app.key(r::KeyCode::Enter);
    app.key(r::KeyCode::Char('d'));
    assert_eq!(app.key(r::KeyCode::Enter), Some(Request::Step(0)));
}

#[test]
fn a_refresh_clears_the_update_marks() {
    let mut app = updatable();
    at(&mut app, "git");
    app.key(r::KeyCode::Char('u'));
    app.refresh(vec![true, false, false, false, false]);
    assert!(app.updates.is_empty());
    assert_eq!(app.marks[0], Mark::Added);
}

#[test]
fn h_opens_the_health_page_of_a_component_with_a_health_action() {
    let mut app = App::new(catalog_with("rust/nightly", |spec| spec.config.actions.health = true), vec![false; 5]);
    at(&mut app, "rust/nightly");
    let nightly = app.current().expect("a component");
    assert_eq!(app.key(r::KeyCode::Char('h')), Some(Request::Health(nightly)));
    assert_eq!(app.screen, Screen::Health);
    app.event(RunEvent::Health { component: nightly, table: Ok("╭──╮\n│ok│\n╰──╯".to_string()) });
    let table = app.health.as_ref().and_then(|page| page.table.as_ref()).expect("the table");
    assert_eq!(table.lines().len(), 3);
    assert_eq!(app.key(r::KeyCode::Esc), None);
    assert_eq!(app.screen, Screen::Select);
    assert!(app.health.is_none());
}

#[test]
fn h_does_nothing_where_the_component_has_no_health_action() {
    let mut app = app([false; 5]);
    at(&mut app, "rust/nightly");
    assert_eq!(app.key(r::KeyCode::Char('h')), None);
    assert_eq!(app.screen, Screen::Select);
    assert_eq!(app.catalog.components[app.current().expect("a component")].id, "rust/nightly", "h moves nothing on the list");
}

#[test]
fn a_finished_run_rechecks_a_super_present_through_the_dialog() {
    let present = Actions { present: true, ..Actions::default() };
    let mut app = App::new(catalog_with_supers("rust/riscv", present), vec![false; 5]);
    proceed(&mut app, "git");
    app.event(RunEvent::Done { step: 0 });
    assert_eq!(app.key(r::KeyCode::Enter), Some(Request::Validate));
    let sudo = app.sudo.as_ref().expect("a refresh");
    assert!(!sudo.shown);
    assert_eq!(sudo.then, Then::Check);
    assert_eq!(app.event(RunEvent::SudoExit { ok: true }), Some(Request::Check));
}
