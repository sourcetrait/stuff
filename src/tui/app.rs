//! The TUI: its state, its keys, and its loop.

use crate::*;

/// How long the loop waits for a key before drawing again.
const TICK: Duration = Duration::from_millis(50);

/// Which screen shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    /// The component list, its marks changeable.
    Select,
    /// The actions the marks make.
    Preview,
    /// The preview under the proceed dialog.
    Confirm,
    /// The run's output.
    Run,
}

/// How the TUI looks: fancy where the terminal scales images to its cells, simple anywhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum Look {
    /// Radio boxes and the header's icon as images, and rounded borders.
    Fancy,
    /// Colored titles for the marks, and horizontal bars and rules.
    Simple,
}

/// Which dialog button has the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Button {
    /// Closes the dialog.
    Cancel,
    /// Starts the run.
    Proceed,
}

/// A component's mark in the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mark {
    /// Not added.
    Absent,
    /// Added.
    Added,
    /// To be added.
    ToAdd,
    /// To be removed.
    ToRemove,
}

/// What a key asks of the loop beyond the state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Leave the TUI.
    Quit,
    /// Start the plan's run.
    Start,
    /// Check every component again and return to the list.
    Refresh,
}

/// The TUI's state.
#[derive(Debug, Clone)]
pub(crate) struct App {
    /// The components.
    pub(crate) catalog: Catalog,
    /// Whether each component is added, by index.
    pub(crate) added: Vec<bool>,
    /// The list's order, by index.
    pub(crate) order: Vec<usize>,
    /// The cursor's position in the list.
    pub(crate) cursor: usize,
    /// The components marked to add.
    pub(crate) adds: Vec<usize>,
    /// The components marked to remove.
    pub(crate) removes: Vec<usize>,
    /// Each component's mark, by index.
    pub(crate) marks: Vec<Mark>,
    /// The screen showing.
    pub(crate) screen: Screen,
    /// The dialog's focused button.
    pub(crate) button: Button,
    /// The plan the marks make.
    pub(crate) plan: Plan,
    /// The run's output.
    pub(crate) output: Output,
    /// The step running, by its index in the plan.
    pub(crate) step: Option<usize>,
    /// Whether a run is under way.
    pub(crate) running: bool,
    /// The run's failure, when it failed.
    pub(crate) failure: Option<String>,
    /// How the TUI looks.
    pub(crate) look: Look,
}

impl App {
    /// The TUI's state for a catalog and what is added.
    pub(crate) fn new(catalog: Catalog, added: Vec<bool>) -> Self {
        let order = catalog.display_order(&added);
        let mut app = Self {
            marks: Vec::new(),
            catalog,
            added,
            order,
            cursor: 0,
            adds: Vec::new(),
            removes: Vec::new(),
            screen: Screen::Select,
            button: Button::Cancel,
            plan: Plan::default(),
            output: Output::default(),
            step: None,
            running: false,
            failure: None,
            look: Look::Simple,
        };
        app.remark();
        app
    }

    /// The component under the cursor, by index.
    pub(crate) fn current(&self) -> Option<usize> {
        self.order.get(self.cursor).copied()
    }

    /// Toggles the component under the cursor, with what it needs or what needs it.
    pub(crate) fn toggle(&mut self) {
        let Some(i) = self.current() else { return };
        let catalog = &self.catalog;
        match self.marks[i] {
            Mark::ToAdd => self.adds.retain(|&a| a != i && !catalog.add_closure(&[a]).contains(&i)),
            Mark::ToRemove => {
                self.removes.retain(|&r| r != i && !catalog.remove_closure(&[r]).contains(&i))
            }
            Mark::Absent => {
                let needed = catalog.add_closure(&[i]);
                self.removes.retain(|&r| !catalog.remove_closure(&[r]).iter().any(|x| needed.contains(x)));
                self.adds.push(i);
            }
            Mark::Added => {
                let taken = catalog.remove_closure(&[i]);
                self.adds.retain(|&a| !catalog.add_closure(&[a]).iter().any(|x| taken.contains(x)));
                self.removes.push(i);
            }
        }
        self.remark();
    }

    /// Applies a key, returning what the loop must do beyond the state.
    pub(crate) fn key(&mut self, code: r::KeyCode) -> Option<Request> {
        use r::KeyCode::{Char, Down, Enter, Esc, Left, Right, Tab, Up};
        match self.screen {
            Screen::Select => match code {
                Char('w') | Up => self.cursor = self.cursor.saturating_sub(1),
                Char('s') | Down => self.cursor = (self.cursor + 1).min(self.order.len().saturating_sub(1)),
                Char('a') | Left => self.climb(),
                Char('d') | Right => self.descend(),
                Char(' ') => self.toggle(),
                Enter => {
                    self.plan = Plan::new(&self.catalog, &self.added, &self.adds, &self.removes);
                    if !self.plan.is_empty() {
                        self.screen = Screen::Preview;
                    }
                }
                Esc | Char('q') => return Some(Request::Quit),
                _ => {}
            },
            Screen::Preview => match code {
                Enter => {
                    self.button = Button::Cancel;
                    self.screen = Screen::Confirm;
                }
                Esc => self.screen = Screen::Select,
                _ => {}
            },
            Screen::Confirm => match code {
                Char('a') | Left => self.button = Button::Cancel,
                Char('d') | Right => self.button = Button::Proceed,
                Tab => {
                    self.button = match self.button {
                        Button::Cancel => Button::Proceed,
                        Button::Proceed => Button::Cancel,
                    }
                }
                Enter if self.button == Button::Proceed => {
                    self.screen = Screen::Run;
                    self.running = true;
                    self.output = Output::default();
                    self.failure = None;
                    self.step = None;
                    return Some(Request::Start);
                }
                Enter | Esc => self.screen = Screen::Preview,
                _ => {}
            },
            Screen::Run => {
                if !self.running && matches!(code, Enter | Esc) {
                    return Some(Request::Refresh);
                }
            }
        }
        None
    }

    /// Applies a run's event.
    pub(crate) fn event(&mut self, event: RunEvent) {
        match event {
            RunEvent::Begin { step } => {
                self.step = Some(step);
                if let Some(&Step { action, component }) = self.plan.steps.get(step) {
                    let color = match action {
                        StepAction::Add => 32,
                        StepAction::Remove => 31,
                    };
                    let title = &self.catalog.components[component].title;
                    self.output.note(&format!("\x1b[{color}m[{}]\x1b[0m {title}", action.label()));
                }
            }
            RunEvent::Output { bytes } => self.output.push(&bytes),
            RunEvent::Fail { message } => {
                self.output.note(&format!("\x1b[31m{message}\x1b[0m"));
                self.failure = Some(message);
            }
            RunEvent::Finish => self.running = false,
        }
    }

    /// Takes every component's state afresh, clearing the marks and the list's order.
    pub(crate) fn refresh(&mut self, added: Vec<bool>) {
        let current = self.current();
        self.added = added;
        self.adds.clear();
        self.removes.clear();
        self.order = self.catalog.display_order(&self.added);
        self.cursor = current.and_then(|i| self.order.iter().position(|&o| o == i)).unwrap_or(0);
        self.screen = Screen::Select;
        self.remark();
    }

    /// Recomputes every mark from the explicit marks and what they carry with them.
    fn remark(&mut self) {
        let adding = self.catalog.add_closure(&self.adds);
        let removing = self.catalog.remove_closure(&self.removes);
        self.marks = (0..self.catalog.components.len())
            .map(|i| match (self.added[i], removing.contains(&i), adding.contains(&i)) {
                (true, true, _) => Mark::ToRemove,
                (true, false, _) => Mark::Added,
                (false, _, true) => Mark::ToAdd,
                (false, _, false) => Mark::Absent,
            })
            .collect();
    }

    /// Moves the cursor to the current component's parent.
    fn climb(&mut self) {
        let parent = self.current().and_then(|i| self.catalog.components[i].parent);
        if let Some(position) = parent.and_then(|p| self.order.iter().position(|&o| o == p)) {
            self.cursor = position;
        }
    }

    /// Moves the cursor to the current component's first subcomponent in the list.
    fn descend(&mut self) {
        let Some(i) = self.current() else { return };
        let children = &self.catalog.components[i].children;
        if let Some(position) = self.order.iter().position(|o| children.contains(o)) {
            self.cursor = position;
        }
    }
}

/// Runs the TUI until the user leaves it, in the look given or the one the terminal gets.
pub(crate) fn run(catalog: Catalog, runner: Runner, added: Vec<bool>, look: Option<Look>) -> Result<ExitCode> {
    let mut terminal = ratatui::init();
    let picker = tui::icon::picker();
    let look = look.unwrap_or_else(|| tui::look::detect(&picker));
    let icons = Icons::load(&catalog, &picker, tui::icon::background(&picker), look);
    let mut app = App::new(catalog, added);
    app.look = look;
    let outcome = drive(&mut terminal, app, &runner, &icons);
    ratatui::restore();
    outcome
}

/// The loop: draw, take the run's events, then a key.
fn drive(
    terminal: &mut ratatui::DefaultTerminal,
    mut app: App,
    runner: &Runner,
    icons: &Icons,
) -> Result<ExitCode> {
    let (sender, events) = mpsc::channel();
    loop {
        while let Ok(event) = events.try_recv() {
            app.event(event);
        }
        terminal.draw(|frame| tui::view::draw(frame, &app, icons)).context(TerminalSnafu)?;
        if !ratatui::crossterm::event::poll(TICK).context(TerminalSnafu)? {
            continue;
        }
        let r::Event::Key(r::KeyEvent { code, kind, .. }) =
            ratatui::crossterm::event::read().context(TerminalSnafu)?
        else {
            continue;
        };
        if kind != r::KeyEventKind::Press {
            continue;
        }
        match app.key(code) {
            Some(Request::Quit) => return Ok(ExitCode::SUCCESS),
            Some(Request::Start) => {
                runner.spawn_plan(&app.catalog, &app.plan, sender.clone());
            }
            Some(Request::Refresh) => {
                let added = runner.exists_all(&app.catalog)?;
                app.refresh(added);
            }
            None => {}
        }
    }
}
