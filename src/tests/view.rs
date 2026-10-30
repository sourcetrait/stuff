use crate::*;

use super::catalog::catalog;

fn render(app: &App, width: u16, height: u16) -> ratatui::buffer::Buffer {
    render_with(app, &Icons::none(app.catalog.components.len()), width, height)
}

fn render_with(app: &App, icons: &Icons, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height))
        .expect("a test terminal");
    terminal.draw(|frame| tui::view::draw(frame, app, icons)).expect("a frame");
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

/// A card's radio box, its top-left cell, found from the card's title.
fn radio(buffer: &ratatui::buffer::Buffer, title: &str) -> (u16, u16) {
    let (x, y) = find(buffer, title).expect("a card");
    (x - tui::view::RADIO_WIDTH - tui::icon::ICON_SIZE.width - 2 * tui::view::GAP, y)
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
fn the_radio_box_is_filled_in_its_mark_color() {
    let mut app = App::new(catalog(), vec![false, true, false, false, false]);
    app.cursor = app.order.iter().position(|&o| o == 0).expect("git listed");
    app.toggle();
    let buffer = render(&app, 80, 24);
    let (x, y) = radio(&buffer, "GIT");
    assert_eq!(buffer[(x, y)].bg, r::Color::Green);
    let lower = &buffer[(x + tui::view::RADIO_WIDTH - 1, y + 1)];
    assert_eq!(lower.bg, r::Color::Green, "the box fills both of the card's lines");
    let (x, y) = radio(&buffer, "RUST");
    assert_eq!(buffer[(x, y)].bg, r::Color::Gray);
}

#[test]
fn an_absent_component_shows_an_empty_box() {
    let app = App::new(catalog(), vec![false; 5]);
    let buffer = render(&app, 80, 24);
    let (x, y) = radio(&buffer, "ZED");
    let top_left = &buffer[(x, y)];
    let bottom_right = &buffer[(x + tui::view::RADIO_WIDTH - 1, y + 1)];
    assert_eq!((top_left.symbol(), top_left.fg), ("🭽", r::Color::Gray));
    assert_eq!(bottom_right.symbol(), "🭿", "the outline spans both of the card's lines");
}

#[test]
fn the_icon_sits_between_the_radio_box_and_the_lines() {
    let app = App::new(catalog(), vec![false; 5]);
    let icons = Icons::load(&app.catalog, &r::Picker::halfblocks(), None);
    let buffer = render_with(&app, &icons, 80, 24);
    let (x, y) = radio(&buffer, "GIT");
    let left = x + tui::view::RADIO_WIDTH + tui::view::GAP;
    let drawn = (left..left + tui::icon::ICON_SIZE.width)
        .flat_map(|x| [(x, y), (x, y + 1)])
        .any(|cell| buffer[cell].symbol() != " " || buffer[cell].bg != r::Color::Reset);
    assert!(drawn, "the icon's cells hold its picture");
}

#[test]
fn the_cursor_frames_its_whole_row_in_purple() {
    let mut app = App::new(catalog(), vec![false; 5]);
    app.cursor = 1;
    let buffer = render(&app, 80, 24);
    let (_, y) = find(&buffer, "RUST").expect("rust's card");
    let top_left = &buffer[(0, y - 1)];
    let bottom_right = &buffer[(79, y + 2)];
    assert_eq!((top_left.symbol(), top_left.fg), ("╭", tui::view::PURPLE));
    assert_eq!((bottom_right.symbol(), bottom_right.fg), ("╯", tui::view::PURPLE));
    assert_eq!((buffer[(0, y)].symbol(), buffer[(0, y + 1)].symbol()), ("│", "│"), "the border runs both lines");
    let (_, git_y) = find(&buffer, "GIT").expect("git's card");
    assert_eq!(buffer[(0, git_y)].symbol(), " ", "the other rows have no border");
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
