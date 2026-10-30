use crate::*;

use super::catalog::catalog;

fn render(app: &App, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height))
        .expect("a test terminal");
    let icons = Icons::none(app.catalog.components.len());
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
fn a_subcomponent_is_indented_under_its_parent() {
    let app = App::new(catalog(), vec![false; 5]);
    let buffer = render(&app, 80, 24);
    let (rust_x, _) = find(&buffer, "RUST").expect("rust's card");
    let (nightly_x, _) = find(&buffer, "RUST/NIGHTLY").expect("nightly's card");
    assert_eq!(nightly_x, rust_x + 4);
}

#[test]
fn the_radio_box_shows_the_mark_in_its_color() {
    let mut app = App::new(catalog(), vec![false, true, false, false, false]);
    let i = app.order.iter().position(|&o| o == 0).expect("git listed");
    app.cursor = i;
    app.toggle();
    let buffer = render(&app, 80, 24);
    let (title_x, title_y) = find(&buffer, "GIT").expect("git's card");
    let radio = &buffer[(title_x - 7, title_y)];
    assert_eq!((radio.symbol(), radio.fg), ("●", r::Color::Green));
    let (rust_x, rust_y) = find(&buffer, "RUST").expect("rust's card");
    let added = &buffer[(rust_x - 7, rust_y)];
    assert_eq!((added.symbol(), added.fg), ("●", r::Color::Gray));
}

#[test]
fn the_dialog_defaults_to_cancel_in_purple() {
    let mut app = App::new(catalog(), vec![false; 5]);
    app.toggle();
    app.key(r::KeyCode::Enter);
    app.key(r::KeyCode::Enter);
    let buffer = render(&app, 80, 24);
    assert!(find(&buffer, "WARNING:").is_some());
    let (x, y) = find(&buffer, "[cancel]").expect("the cancel button");
    assert_eq!(buffer[(x, y)].bg, tui::view::PURPLE);
    let (x, y) = find(&buffer, "[proceed]").expect("the proceed button");
    assert_ne!(buffer[(x, y)].bg, tui::view::PURPLE);
}
