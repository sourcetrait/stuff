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
/// The rows a card takes, its spacing included.
const CARD_ROWS: u16 = 3;
/// The columns a subcomponent is indented by, a level.
const INDENT: u16 = 4;

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

/// The header: the configuration's icon, its title, and the directory.
fn header(frame: &mut ratatui::Frame, area: r::Rect, app: &App, icons: &Icons) {
    if let Some(protocol) = &icons.header {
        frame.render_widget(r::Image::new(protocol), clip(area.x + 1, area.y, 4, 2, area));
    }
    let text = vec![
        r::Line::styled(app.catalog.config.title.clone(), r::Style::new().add_modifier(r::Modifier::BOLD)),
        r::Line::styled(app.catalog.dir.display().to_string(), r::Style::new().fg(QUIET)),
    ];
    frame.render_widget(r::Paragraph::new(text), clip(area.x + 7, area.y, area.width, 2, area));
}

/// The component list: a card a component, the cursor's in purple.
fn select(frame: &mut ratatui::Frame, area: r::Rect, app: &App, icons: &Icons) {
    if app.order.is_empty() {
        frame.render_widget(r::Paragraph::new(r::Line::styled("no components", r::Style::new().fg(QUIET))), area);
        return;
    }
    let visible = usize::from((area.height / CARD_ROWS).max(1));
    let start = (app.cursor + 1).saturating_sub(visible);
    for (row, &i) in app.order.iter().enumerate().skip(start).take(visible) {
        let y = area.y + u16::try_from(row - start).unwrap_or(0) * CARD_ROWS;
        card(frame, clip(area.x, y, area.width, 2, area), app, icons, i, row == app.cursor);
    }
}

/// A component's card: its radio box, its icon, its title, and its summary.
fn card(frame: &mut ratatui::Frame, area: r::Rect, app: &App, icons: &Icons, i: usize, selected: bool) {
    let component = &app.catalog.components[i];
    if selected {
        let bar = r::Paragraph::new(vec![r::Line::raw("▌"), r::Line::raw("▌")]).style(r::Style::new().fg(PURPLE));
        frame.render_widget(bar, clip(area.x, area.y, 1, 2, area));
    }
    let x = area.x + 2 + u16::try_from(component.depth).unwrap_or(0) * INDENT;
    let (glyph, color) = match app.marks[i] {
        Mark::Absent => ("○", r::Color::Gray),
        Mark::Added => ("●", r::Color::Gray),
        Mark::ToAdd => ("●", r::Color::Green),
        Mark::ToRemove => ("●", r::Color::Red),
    };
    frame.render_widget(r::Paragraph::new(r::Line::styled(glyph, r::Style::new().fg(color))), clip(x, area.y, 1, 1, area));
    if let Some(Some(protocol)) = icons.components.get(i) {
        frame.render_widget(r::Image::new(protocol), clip(x + 2, area.y, 4, 2, area));
    }
    let title_style = if selected {
        r::Style::new().fg(PURPLE).add_modifier(r::Modifier::BOLD)
    } else {
        r::Style::new().add_modifier(r::Modifier::BOLD)
    };
    let text = vec![
        r::Line::styled(component.title.clone(), title_style),
        r::Line::styled(component.description.clone(), r::Style::new().fg(r::Color::Gray)),
    ];
    frame.render_widget(r::Paragraph::new(text), clip(x + 7, area.y, area.width, 2, area));
}

/// The preview: each step, a green add or a red remove, in the order it runs.
fn preview(frame: &mut ratatui::Frame, area: r::Rect, app: &App) {
    let lines: Vec<r::Line> = app
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
    frame.render_widget(r::Paragraph::new(lines), clip(area.x + 2, area.y, area.width, area.height, area));
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
