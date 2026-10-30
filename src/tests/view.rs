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
