//! The TUI's drawing: the header, each screen, the dialog, and the legend.

use crate::*;

/// The color of every highlighted widget.
pub(crate) const PURPLE: r::Color = r::Color::Rgb(160, 100, 240);
/// The color of the legend and other quiet text.
pub(crate) const QUIET: r::Color = r::Color::DarkGray;
/// The list's legend.
pub(crate) const LEGEND_SELECT: &str = "<< [esc]     [w][a][s][d] [space]     [enter] >>";
/// The legend of the preview and the finished run.
pub(crate) const LEGEND_STEP: &str = "<< [esc]     [enter] >>";
/// A card's rows: its two lines within its border's.
const CARD_ROWS: u16 = 4;
/// The columns between a card's border and its content.
const PADDING: u16 = 1;
/// The columns from the screen's edge to a row's content: a card's border and its padding.
const INSET: u16 = 1 + PADDING;
/// The columns a subcomponent is indented by, a level.
const INDENT: u16 = 4;
/// The radio box's columns, about square over a card's two lines.
pub(crate) const RADIO_WIDTH: u16 = 4;
/// The columns between a row's parts.
pub(crate) const GAP: u16 = 1;

/// Draws the screen showing.
pub(crate) fn draw(frame: &mut ratatui::Frame, app: &App, icons: &Icons) {
    let [head, body, foot] = r::Layout::vertical([
        r::Constraint::Length(3),
        r::Constraint::Min(1),
        r::Constraint::Length(1),
    ])
    .areas(frame.area());
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

/// The header, a row: the configuration's icon, and its title over the directory.
fn header(frame: &mut ratatui::Frame, area: r::Rect, app: &App, icons: &Icons) {
    let [row, _] = r::Layout::vertical([r::Constraint::Length(2), r::Constraint::Fill(1)])
        .areas(area.inner(r::Margin::new(INSET, 0)));
    let [icon_area, lines_area] =
        r::Layout::horizontal([r::Constraint::Length(tui::icon::ICON_SIZE.width), r::Constraint::Fill(1)])
            .spacing(GAP)
            .areas(row);
    if let Some(protocol) = &icons.header {
        frame.render_widget(r::Image::new(protocol), icon_area);
    }
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
    let visible = usize::from((area.height / CARD_ROWS).max(1));
    let start = (app.cursor + 1).saturating_sub(visible);
    let rows = r::Layout::vertical(vec![r::Constraint::Length(CARD_ROWS); visible])
        .flex(r::Flex::Start)
        .split(area);
    for (&row_area, (row, &i)) in rows.iter().zip(app.order.iter().enumerate().skip(start)) {
        card(frame, row_area, app, icons, i, row == app.cursor);
    }
}

/// A component's card, a row in its border: its radio box, its icon, and its title over its summary.
fn card(frame: &mut ratatui::Frame, area: r::Rect, app: &App, icons: &Icons, i: usize, selected: bool) {
    let component = &app.catalog.components[i];
    let border = r::Block::bordered()
        .border_type(r::BorderType::Rounded)
        .border_style(r::Style::new().fg(PURPLE))
        .padding(r::Padding::horizontal(PADDING));
    let inner = border.inner(area);
    if selected {
        frame.render_widget(border, area);
    }
    let indent = u16::try_from(component.depth).unwrap_or(u16::MAX).saturating_mul(INDENT).min(inner.width);
    let row = r::Rect { x: inner.x + indent, width: inner.width - indent, ..inner };
    let [radio_area, icon_area, lines_area] = r::Layout::horizontal([
        r::Constraint::Length(RADIO_WIDTH),
        r::Constraint::Length(tui::icon::ICON_SIZE.width),
        r::Constraint::Fill(1),
    ])
    .spacing(GAP)
    .areas(row);
    radio(frame, radio_area, app.marks[i]);
    if let Some(Some(protocol)) = icons.components.get(i) {
        frame.render_widget(r::Image::new(protocol), icon_area);
    }
    lines(
        frame,
        lines_area,
        r::Line::styled(component.title.as_str(), r::Style::new().add_modifier(r::Modifier::BOLD)),
        r::Line::styled(component.description.as_str(), r::Style::new().fg(r::Color::Gray)),
    );
}

/// A row's two lines, one over the other.
fn lines(frame: &mut ratatui::Frame, area: r::Rect, top: r::Line, bottom: r::Line) {
    let [top_area, bottom_area] = r::Layout::vertical([r::Constraint::Length(1); 2]).areas(area);
    frame.render_widget(top, top_area);
    frame.render_widget(bottom, bottom_area);
}

/// The radio box, a card's two lines tall: an outline on its cells' edges when absent, filled in its mark's color otherwise.
fn radio(frame: &mut ratatui::Frame, area: r::Rect, mark: Mark) {
    let fill = match mark {
        Mark::Absent => {
            let edge = usize::from(area.width.saturating_sub(2));
            let style = r::Style::new().fg(r::Color::Gray);
            let outline = vec![
                r::Line::styled(format!("🭽{}🭾", "▔".repeat(edge)), style),
                r::Line::styled(format!("🭼{}🭿", "▁".repeat(edge)), style),
            ];
            frame.render_widget(r::Paragraph::new(outline), area);
            return;
        }
        Mark::Added => r::Color::Gray,
        Mark::ToAdd => r::Color::Green,
        Mark::ToRemove => r::Color::Red,
    };
    frame.render_widget(r::Block::new().style(r::Style::new().bg(fill)), area);
}

/// The preview: each step, a green add or a red remove, in the order it runs.
fn preview(frame: &mut ratatui::Frame, area: r::Rect, app: &App) {
    let steps: Vec<r::Line> = app
        .plan
        .steps
        .iter()
        .map(|step| {
            let color = match step.action {
                StepAction::Add => r::Color::Green,
                StepAction::Remove => r::Color::Red,
            };
            r::Line::from(vec![
                r::Span::styled(format!("[{}]", step.action.label()), r::Style::new().fg(color)),
                r::Span::raw(format!(" {}", app.catalog.components[step.component].title)),
            ])
        })
        .collect();
    frame.render_widget(r::Paragraph::new(steps), area.inner(r::Margin::new(INSET, 0)));
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
    let block = r::Block::bordered()
        .border_type(r::BorderType::Rounded)
        .border_style(r::Style::new().fg(PURPLE))
        .title(" Proceed? ");
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
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
    let block = r::Block::bordered()
        .border_type(r::BorderType::Rounded)
        .border_style(r::Style::new().fg(QUIET))
        .title(" output ");
    let inner = block.inner(output_area);
    frame.render_widget(block, output_area);
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
        Screen::Run if app.running => "",
        _ => LEGEND_STEP,
    };
    frame.render_widget(r::Paragraph::new(r::Line::styled(text, r::Style::new().fg(QUIET))).centered(), area);
}

/// A rectangle at a position, cut to fit within an area.
fn clip(x: u16, y: u16, width: u16, height: u16, within: r::Rect) -> r::Rect {
    r::Rect::new(x, y, width, height).intersection(within)
}
