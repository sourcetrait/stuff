use crate::*;

use super::catalog::catalog;

fn render(app: &App, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height))
        .expect("a test terminal");
    let icons = Icons::none();
    terminal.draw(|frame| tui::view::draw(frame, app, &icons)).expect("a frame");
    terminal.backend().buffer().clone()
}

fn rows(buffer: &ratatui::buffer::Buffer) -> Vec<String> {
    (0..buffer.area.height)
        .map(|y| (0..buffer.area.width).map(|x| buffer[(x, y)].symbol()).collect::<String>())
        .collect()
}

fn find(buffer: &ratatui::buffer::Buffer, text: &str) -> Option<(u16, u16)> {
    rows(buffer).iter().enumerate().find_map(|(y, row)| {
        row.find(text).map(|byte| {
            let x = row[..byte].chars().count();
            (u16::try_from(x).unwrap_or(0), u16::try_from(y).unwrap_or(0))
        })
    })
}

#[test]
fn the_list_shows_the_title_the_cards_and_the_legend() {
    let app = App::new(catalog(), vec![false; 5]);
    let buffer = render(&app, 80, 24);
    let rows = rows(&buffer);
    assert!(rows[0].contains("Test"), "{rows:#?}");
    assert!(rows.iter().any(|row| row.contains("RUST/NIGHTLY")), "{rows:#?}");
    assert!(rows.iter().any(|row| row.contains("the zed component")), "{rows:#?}");
    assert!(rows[23].contains(tui::view::LEGEND_SELECT), "{rows:#?}");
}

#[test]
fn the_sudo_dialog_shows_its_title_and_sudo_s_exchange() {
    let mut app = App::new(catalog(), vec![false; 5]);
    let mut sudo = SudoDialog::new(Then::Step(0), true);
    sudo.terminal.push(b"\nWe trust you have received the usual lecture.\n\n[sudo] password for test: ");
    app.sudo = Some(sudo);
    let buffer = render(&app, 80, 24);
    let (title_x, title_y) = find(&buffer, " [sudo] ").expect("the title");
    let (_, prompt_y) = find(&buffer, "[sudo] password for test:").expect("the prompt");
    assert!(prompt_y > title_y + 1, "the prompt sits in the mini terminal, under the title");
    assert_eq!(buffer[(title_x + 1, title_y)].fg, tui::view::YELLOW);
    assert!(find(&buffer, "usual lecture").is_some());
}

#[test]
fn the_sudo_dialog_fits_a_small_screen() {
    let mut app = App::new(catalog(), vec![false; 5]);
    let mut sudo = SudoDialog::new(Then::Step(0), true);
    sudo.terminal.push(b"alpha\nbravo\ncharlie\ndelta\necho\nfoxtrot\n[sudo] password for test: ");
    app.sudo = Some(sudo);
    let buffer = render(&app, 40, 10);
    assert!(find(&buffer, "[sudo] password").is_some(), "the newest line shows");
    assert!(find(&buffer, "alpha").is_none(), "the oldest scroll away");
}

#[test]
fn the_health_page_titles_the_component_over_its_table() {
    let mut app = App::new(catalog(), vec![false; 5]);
    app.screen = Screen::Health;
    let mut table = Output::default();
    table.push(b"component  rust\npresent    true\n");
    app.health = Some(HealthPage { component: 1, table: Some(table), failure: None, scroll: 0 });
    let buffer = render(&app, 80, 24);
    let (_, title_y) = find(&buffer, "Health: RUST").expect("the title");
    let (_, row_y) = find(&buffer, "present    true").expect("the table");
    assert!(row_y > title_y);
    assert!(find(&buffer, tui::view::LEGEND_HEALTH).is_some());
}

#[test]
fn an_unseen_refresh_draws_no_dialog() {
    let mut app = App::new(catalog(), vec![false; 5]);
    app.sudo = Some(SudoDialog::new(Then::Step(1), false));
    assert!(find(&render(&app, 80, 24), "[sudo]").is_none());
}

#[test]
fn a_subcomponent_is_indented_under_its_parent() {
    let app = App::new(catalog(), vec![false; 5]);
    let buffer = render(&app, 80, 24);
    let (rust_x, _) = find(&buffer, "RUST").expect("rust's card");
    let (nightly_x, _) = find(&buffer, "RUST/NIGHTLY").expect("nightly's card");
    assert_eq!(nightly_x, rust_x + tui::view::INDENT);
}
