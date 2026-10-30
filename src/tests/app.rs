use crate::*;

use super::catalog::catalog;

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
    assert_eq!(app.key(r::KeyCode::Enter), Some(Request::Start));
    assert_eq!(app.screen, Screen::Run);
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
fn a_finished_run_returns_to_a_refreshed_list() {
    let mut app = app([false; 5]);
    at(&mut app, "git");
    app.toggle();
    app.key(r::KeyCode::Enter);
    app.key(r::KeyCode::Enter);
    app.key(r::KeyCode::Char('d'));
    app.key(r::KeyCode::Enter);
    app.event(RunEvent::Begin { step: 0 });
    app.event(RunEvent::Output { bytes: b"installing\n".to_vec() });
    assert_eq!(app.key(r::KeyCode::Enter), None, "a running run holds the screen");
    app.event(RunEvent::Finish);
    assert_eq!(app.key(r::KeyCode::Enter), Some(Request::Refresh));
    app.refresh(vec![true, false, false, false, false]);
    assert_eq!(app.screen, Screen::Select);
    assert_eq!(app.marks[0], Mark::Added);
    assert_eq!(app.catalog.components[app.current().expect("a component")].id, "git");
}
