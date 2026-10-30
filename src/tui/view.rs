//! The TUI's drawing: the header, each screen, the dialog, and the legend.

use crate::*;

/// The color of every highlighted widget.
pub(crate) const PURPLE: r::Color = r::Color::Rgb(160, 100, 240);
/// The color of the legend and other quiet text.
pub(crate) const QUIET: r::Color = r::Color::DarkGray;
/// In the simple look, the title's color of an added component: blue.
const ADDED: r::Color = r::Color::Rgb(90, 150, 255);
/// The color of a component to add: green, its title's in the simple look and the preview's rule.
const TO_ADD: r::Color = r::Color::Rgb(70, 200, 110);
/// The color of a component to remove: red, its title's in the simple look and the preview's rule.
const TO_REMOVE: r::Color = r::Color::Rgb(235, 85, 85);
/// The list's legend.
pub(crate) const LEGEND_SELECT: &str = "<< [esc]     [w][a][s][d] [space]     [enter] >>";
/// The preview's legend.
pub(crate) const LEGEND_PREVIEW: &str = "<< [esc]     [w][s]     [enter] >>";
/// The legend of the dialog and the finished run.
pub(crate) const LEGEND_STEP: &str = "<< [esc]     [enter] >>";
/// A card's rows: its two lines within its border's.
const CARD_ROWS: u16 = 4;
/// The columns between a card's border and its content.
const PADDING: u16 = 1;
/// The columns from the screen's edge to a row's content: a card's border and its padding.
const INSET: u16 = 1 + PADDING;
/// The columns between a row's parts.
const GAP: u16 = 1;
/// The columns a subcomponent is indented by, a level: a radio box and its gap, so its radio box
/// lines up under its parent's title.
pub(crate) const INDENT: u16 = tui::icon::ICON_SIZE.width + GAP;

/// The screen's three regions: the header, the body, and the legend.
pub(crate) fn regions(area: r::Rect) -> [r::Rect; 3] {
    r::Layout::vertical([r::Constraint::Length(3), r::Constraint::Min(1), r::Constraint::Length(1)]).areas(area)
}

/// Draws the screen showing.
pub(crate) fn draw(frame: &mut ratatui::Frame, app: &App, icons: &Icons) {
    let [head, body, foot] = regions(frame.area());
    header(frame, head, app, icons);
    match app.screen {
        Screen::Select => select(frame, body, app, icons),
        Screen::Preview => preview(frame, body, app),
        Screen::Confirm => {
            preview(frame, body, app);
            confirm(frame, frame.area(), app);
        }
        Screen::Run => run(frame, body, app),
    }
    legend(frame, foot, app);
}

/// The header, a row: the configuration's icon in the fancy look, and its title over the directory.
fn header(frame: &mut ratatui::Frame, area: r::Rect, app: &App, icons: &Icons) {
    let [row, _] = r::Layout::vertical([r::Constraint::Length(2), r::Constraint::Fill(1)])
        .areas(area.inner(r::Margin::new(INSET, 0)));
    let lines_area = if app.look == Look::Fancy {
        let [icon_area, lines_area] =
            r::Layout::horizontal([r::Constraint::Length(tui::icon::ICON_SIZE.width), r::Constraint::Fill(1)])
                .spacing(GAP)
                .areas(row);
        image(frame, icon_area, icons.header.as_ref());
        lines_area
    } else {
        row
    };
    lines(
        frame,
        lines_area,
        r::Line::styled(app.catalog.config.title.as_str(), r::Style::new().add_modifier(r::Modifier::BOLD)),
        r::Line::styled(app.catalog.dir.display().to_string(), r::Style::new().fg(QUIET)),
    );
}

/// The component list: a column of cards, the cursor's framed in purple.
fn select(frame: &mut ratatui::Frame, area: r::Rect, app: &App, icons: &Icons) {
    if app.order.is_empty() {
        frame.render_widget(r::Paragraph::new(r::Line::styled("no components", r::Style::new().fg(QUIET))), area);
        return;
    }
    // In the simple look neighbouring cards share a border row, which holds a bar only for the cursor's.
    let stride = if app.look == Look::Fancy { CARD_ROWS } else { CARD_ROWS - 1 };
    let visible = usize::from((area.height.saturating_sub(CARD_ROWS - stride) / stride).max(1));
    let start = (app.cursor + 1).saturating_sub(visible);
    for (row, &i) in app.order.iter().enumerate().skip(start).take(visible) {
        let y = area.y + u16::try_from(row - start).unwrap_or(0) * stride;
        let card_area = r::Rect::new(area.x, y, area.width, CARD_ROWS).intersection(area);
        card(frame, card_area, app, icons, i, row == app.cursor);
    }
}

/// A component's card, a row in its border. In the fancy look: its radio box, and its title over its
/// summary. In the simple look, no box: the title in its mark's color, `[present]` after it where the
/// component is added.
fn card(frame: &mut ratatui::Frame, area: r::Rect, app: &App, icons: &Icons, i: usize, selected: bool) {
    let component = &app.catalog.components[i];
    let images = app.look == Look::Fancy;
    if selected {
        border(frame, area, app.look, PURPLE, None);
    }
    let shade = shade(app.marks[i]).filter(|_| !images);
    let mut title = r::Line::styled(
        component.title.as_str(),
        shade.map_or(r::Style::new(), |color| r::Style::new().fg(color)).add_modifier(r::Modifier::BOLD),
    );
    if !images && app.added[i] {
        title.push_span(r::Span::styled(" [present]", shade.map_or(r::Style::new(), |color| r::Style::new().fg(color))));
    }
    let inner = area.inner(r::Margin::new(INSET, 1));
    let indent = u16::try_from(component.depth).unwrap_or(u16::MAX).saturating_mul(INDENT).min(inner.width);
    let row = r::Rect { x: inner.x + indent, width: inner.width - indent, ..inner };
    let lines_area = if let Some(radios) = icons.radios.as_ref().filter(|_| images) {
        let [radio_area, lines_area] =
            r::Layout::horizontal([r::Constraint::Length(tui::icon::ICON_SIZE.width), r::Constraint::Fill(1)])
                .spacing(GAP)
                .areas(row);
        image(frame, radio_area, radios.get(app.marks[i]));
        lines_area
    } else {
        row
    };
    lines(frame, lines_area, title, r::Line::styled(component.description.as_str(), r::Style::new().fg(r::Color::Gray)));
}

/// The color a mark gives its title in the simple look.
fn shade(mark: Mark) -> Option<r::Color> {
    match mark {
        Mark::Absent => None,
        Mark::Added => Some(ADDED),
        Mark::ToAdd => Some(TO_ADD),
        Mark::ToRemove => Some(TO_REMOVE),
    }
}

/// A row's two lines, one over the other.
fn lines(frame: &mut ratatui::Frame, area: r::Rect, top: r::Line, bottom: r::Line) {
    let [top_area, bottom_area] = r::Layout::vertical([r::Constraint::Length(1); 2]).areas(area);
    frame.render_widget(top, top_area);
    frame.render_widget(bottom, bottom_area);
}

/// An icon or a radio box in its cells, an image through the graphics protocol.
fn image(frame: &mut ratatui::Frame, area: r::Rect, protocol: Option<&r::Protocol>) {
    if let Some(protocol) = protocol {
        frame.render_widget(r::Image::new(protocol), area);
    }
}

/// A border around an area, a title in its top: rounded lines in the fancy look; in the simple look,
/// horizontal bars above and below, which come out whole where a terminal breaks a vertical line
/// between rows.
fn border(frame: &mut ratatui::Frame, area: r::Rect, look: Look, color: r::Color, title: Option<&str>) {
    let block = match look {
        Look::Simple => r::Block::new().borders(r::Borders::TOP | r::Borders::BOTTOM),
        Look::Fancy => r::Block::bordered().border_type(r::BorderType::Rounded),
    }
    .border_style(r::Style::new().fg(color));
    frame.render_widget(if let Some(title) = title { block.title(title) } else { block }, area);
}

/// The preview, scrolled by the app's scroll.
fn preview(frame: &mut ratatui::Frame, area: r::Rect, app: &App) {
    let area = area.inner(r::Margin::new(INSET, 0));
    frame.render_widget(r::Paragraph::new(preview_lines(app, area.width)).scroll((app.scroll, 0)), area);
}

/// The preview's lines: a section for what will be installed and one for what will be uninstalled,
/// each a heading, a sentence, a rule in its color as wide as given, and its components in the order
/// they run; a section with none is left out.
pub(crate) fn preview_lines(app: &App, width: u16) -> Vec<r::Line<'_>> {
    let sections = [
        (StepAction::Add, "Installing", "The following components will be installed", TO_ADD),
        (StepAction::Remove, "Uninstalling", "The following components will be uninstalled", TO_REMOVE),
    ];
    let mut lines = Vec::new();
    for (action, heading, sentence, color) in sections {
        let mut titles = app
            .plan
            .steps
            .iter()
            .filter(|step| step.action == action)
            .map(|step| app.catalog.components[step.component].title.as_str())
            .peekable();
        if titles.peek().is_none() {
            continue;
        }
        if !lines.is_empty() {
            lines.push(r::Line::default());
        }
        lines.push(r::Line::styled(heading, r::Style::new().add_modifier(r::Modifier::BOLD)));
        lines.push(r::Line::raw(sentence));
        lines.push(r::Line::styled("─".repeat(usize::from(width)), r::Style::new().fg(color)));
        lines.extend(titles.map(|title| r::Line::raw(format!("  {title}"))));
    }
    lines
}

/// The proceed dialog, centered over the screen, its focused button purple.
fn confirm(frame: &mut ratatui::Frame, area: r::Rect, app: &App) {
    let width = area.width.min(62);
    let height = area.height.min(8);
    let popup = r::Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(r::Clear, popup);
    border(frame, popup, app.look, PURPLE, Some(" Proceed? "));
    let inner = popup.inner(r::Margin::new(1, 1));
    let [text_area, button_area] =
        r::Layout::vertical([r::Constraint::Min(1), r::Constraint::Length(1)]).areas(inner);
    let text = vec![
        r::Line::default(),
        r::Line::from(vec![
            r::Span::styled("WARNING:", r::Style::new().fg(r::Color::Yellow).add_modifier(r::Modifier::BOLD)),
            r::Span::raw(" This will add/remove components in your $HOME."),
        ]),
        r::Line::default(),
        r::Line::raw("Proceed?"),
    ];
    frame.render_widget(r::Paragraph::new(text).centered(), text_area);
    let buttons = r::Line::from(vec![
        button("cancel", app.button == Button::Cancel),
        r::Span::raw("   "),
        button("proceed", app.button == Button::Proceed),
    ]);
    frame.render_widget(r::Paragraph::new(buttons).centered(), button_area);
}

/// A dialog button, purple when it has the focus.
fn button(label: &str, focused: bool) -> r::Span<'static> {
    let style = if focused {
        r::Style::new().fg(r::Color::Black).bg(PURPLE).add_modifier(r::Modifier::BOLD)
    } else {
        r::Style::new()
    };
    r::Span::styled(format!("[{label}]"), style)
}

/// The run: the scripts' output in a frame, and the run's status under it.
fn run(frame: &mut ratatui::Frame, area: r::Rect, app: &App) {
    let [output_area, status_area] =
        r::Layout::vertical([r::Constraint::Min(3), r::Constraint::Length(1)]).areas(area);
    border(frame, output_area, app.look, QUIET, Some(" output "));
    let inner = output_area.inner(r::Margin::new(1, 1));
    frame.render_widget(r::Paragraph::new(app.output.tail(usize::from(inner.height))), inner);
    let status = if app.running {
        let step = app.step.map_or(0, |s| s + 1);
        r::Line::styled(format!("running {step} of {}", app.plan.steps.len()), r::Style::new().fg(PURPLE))
    } else if app.failure.is_some() {
        r::Line::styled("failed", r::Style::new().fg(r::Color::Red))
    } else {
        r::Line::styled("done", r::Style::new().fg(r::Color::Green))
    };
    frame.render_widget(r::Paragraph::new(status), clip(area.x + 1, status_area.y, area.width, 1, area));
}

/// The legend, quiet and centered at the bottom.
fn legend(frame: &mut ratatui::Frame, area: r::Rect, app: &App) {
    let text = match app.screen {
        Screen::Select => LEGEND_SELECT,
        Screen::Preview => LEGEND_PREVIEW,
        Screen::Run if app.running => "",
        _ => LEGEND_STEP,
    };
    frame.render_widget(r::Paragraph::new(r::Line::styled(text, r::Style::new().fg(QUIET))).centered(), area);
}

/// A rectangle at a position, cut to fit within an area.
fn clip(x: u16, y: u16, width: u16, height: u16, within: r::Rect) -> r::Rect {
    r::Rect::new(x, y, width, height).intersection(within)
}
