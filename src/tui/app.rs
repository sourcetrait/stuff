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

/// What a key or an event asks of the loop beyond the state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Leave the TUI.
    Quit,
    /// Run the plan's step, by its index.
    Step(usize),
    /// Check every component, then show the list.
    Check,
    /// Start sudo validating the credential.
    Validate,
    /// Write the typed password to sudo.
    Send,
    /// End sudo, its dialog left.
    Cancel,
}

/// What follows once sudo holds the credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Then {
    /// The plan's step, by its index.
    Step(usize),
    /// Every component checked.
    Check,
}

/// The sudo dialog: sudo's own exchange in a mini terminal, the password typed unseen.
#[derive(Debug, Clone)]
pub(crate) struct SudoDialog {
    /// What sudo wrote, a line end after each password sent.
    pub(crate) terminal: Output,
    /// The password being typed.
    pub(crate) password: Secret,
    /// Whether the dialog shows: from the start before a run or a check, and for a refresh once
    /// sudo writes, a refresh with the credential cached writing nothing.
    pub(crate) shown: bool,
    /// Whether sudo ended without the credential.
    pub(crate) failed: bool,
    /// What follows once sudo holds the credential.
    pub(crate) then: Then,
}

impl SudoDialog {
    /// The dialog for a validation, shown from the start or not.
    pub(crate) fn new(then: Then, shown: bool) -> Self {
        Self { terminal: Output::default(), password: Secret::default(), shown, failed: false, then }
    }
}

/// The TUI's state.
#[derive(Debug, Clone)]
pub(crate) struct App {
    /// The components.
    pub(crate) catalog: Catalog,
    /// Whether each component is added, by index.
    pub(crate) added: Vec<bool>,
    /// Whether the components have been checked, which the first check waits on the dialog for
    /// where a present script is super.
    pub(crate) checked: bool,
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
    /// The sudo dialog, while sudo validates.
    pub(crate) sudo: Option<SudoDialog>,
    /// How the TUI looks.
    pub(crate) look: Look,
    /// The preview's first line showing.
    pub(crate) scroll: u16,
    /// The body's rows at the last frame, which bound the preview's scroll.
    pub(crate) body_rows: u16,
}

impl App {
    /// The TUI's state for a catalog and what is added.
    pub(crate) fn new(catalog: Catalog, added: Vec<bool>) -> Self {
        let order = catalog.display_order(&added);
        let mut app = Self {
            marks: Vec::new(),
            catalog,
            added,
            checked: true,
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
            sudo: None,
            look: Look::Simple,
            scroll: 0,
            body_rows: 0,
        };
        app.remark();
        app
    }

    /// The TUI's state before its first check, which waits on the sudo dialog.
    pub(crate) fn unchecked(catalog: Catalog) -> Self {
        let added = vec![false; catalog.components.len()];
        let mut app = Self::new(catalog, added);
        app.checked = false;
        app.sudo = Some(SudoDialog::new(Then::Check, true));
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

    /// Applies a key, returning what the loop must do beyond the state. `w` `a` `s` `d`, helix's
    /// `k` `h` `j` `l`, and the arrows move alike, and `q` is escape. While the sudo dialog shows,
    /// every key but enter and escape is the password's.
    pub(crate) fn key(&mut self, code: r::KeyCode) -> Option<Request> {
        use r::KeyCode::{Backspace, Char, Down, Enter, Esc, Left, Right, Tab, Up};
        if let Some(sudo) = self.sudo.as_mut() {
            if !sudo.shown {
                return None;
            }
            match code {
                Esc => return self.leave_sudo(),
                Enter if !sudo.failed => {
                    sudo.terminal.push(b"\n");
                    return Some(Request::Send);
                }
                Backspace => sudo.password.pop(),
                Char(c) if !sudo.failed => sudo.password.push(c),
                _ => {}
            }
            return None;
        }
        let code = match code {
            Char('w' | 'k') => Up,
            Char('s' | 'j') => Down,
            Char('a' | 'h') => Left,
            Char('d' | 'l') => Right,
            Char('q') => Esc,
            other => other,
        };
        match self.screen {
            Screen::Select => match code {
                Up => self.cursor = self.cursor.saturating_sub(1),
                Down => self.cursor = (self.cursor + 1).min(self.order.len().saturating_sub(1)),
                Left => self.climb(),
                Right => self.descend(),
                Char(' ') => self.toggle(),
                Enter => {
                    self.plan = Plan::new(&self.catalog, &self.added, &self.adds, &self.removes);
                    if !self.plan.is_empty() {
                        self.scroll = 0;
                        self.screen = Screen::Preview;
                    }
                }
                Esc => return Some(Request::Quit),
                _ => {}
            },
            Screen::Preview => match code {
                Up => self.scroll = self.scroll.saturating_sub(1),
                Down => self.scroll = (self.scroll + 1).min(self.scroll_limit()),
                Enter => {
                    self.button = Button::Cancel;
                    self.screen = Screen::Confirm;
                }
                Esc => self.screen = Screen::Select,
                _ => {}
            },
            Screen::Confirm => match code {
                Left => self.button = Button::Cancel,
                Right => self.button = Button::Proceed,
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
                    if self.plan.needs_sudo(&self.catalog) {
                        self.sudo = Some(SudoDialog::new(Then::Step(0), true));
                        return Some(Request::Validate);
                    }
                    return Some(Request::Step(0));
                }
                Enter | Esc => self.screen = Screen::Preview,
                _ => {}
            },
            Screen::Run => {
                if !self.running && matches!(code, Enter | Esc) {
                    return Some(self.check());
                }
            }
        }
        None
    }

    /// Leaves the sudo dialog, as escape leaves every screen: before a run, back to the preview;
    /// during one, its end; before the first check, out of the TUI; before a later one, back to the
    /// finished run.
    fn leave_sudo(&mut self) -> Option<Request> {
        let sudo = self.sudo.take()?;
        match sudo.then {
            Then::Check if !self.checked => return Some(Request::Quit),
            Then::Check => {}
            Then::Step(0) => {
                self.running = false;
                self.screen = Screen::Preview;
            }
            Then::Step(_) => {
                let message = "sudo: left without the credential".to_string();
                self.output.note(&format!("\x1b[31m{message}\x1b[0m"));
                self.failure = Some(message);
                self.running = false;
            }
        }
        Some(Request::Cancel)
    }

    /// The check after a run, through the sudo dialog where a present script is super.
    fn check(&mut self) -> Request {
        if self.catalog.super_present() {
            self.sudo = Some(SudoDialog::new(Then::Check, false));
            Request::Validate
        } else {
            Request::Check
        }
    }

    /// How far the preview scrolls: until its last line sits at the body's bottom.
    fn scroll_limit(&self) -> u16 {
        let lines = u16::try_from(tui::view::preview_lines(self, 0).len()).unwrap_or(u16::MAX);
        lines.saturating_sub(self.body_rows)
    }

    /// Applies a run's or sudo's event, returning what the loop must do next.
    pub(crate) fn event(&mut self, event: RunEvent) -> Option<Request> {
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
                self.running = false;
            }
            RunEvent::Done { step } => return self.next(step + 1),
            RunEvent::SudoOutput { bytes } => {
                if let Some(sudo) = self.sudo.as_mut() {
                    sudo.terminal.push(&bytes);
                    sudo.shown = true;
                }
            }
            RunEvent::SudoExit { ok } => {
                let sudo = self.sudo.as_mut()?;
                if !ok {
                    sudo.failed = true;
                    sudo.shown = true;
                    return None;
                }
                let then = sudo.then;
                self.sudo = None;
                return Some(match then {
                    Then::Step(step) => Request::Step(step),
                    Then::Check => Request::Check,
                });
            }
        }
        None
    }

    /// What follows a step: the next, its credential refreshed first where the plan needs sudo, or
    /// the run's end.
    fn next(&mut self, step: usize) -> Option<Request> {
        if step >= self.plan.steps.len() {
            self.running = false;
            return None;
        }
        if self.plan.needs_sudo(&self.catalog) {
            self.sudo = Some(SudoDialog::new(Then::Step(step), false));
            return Some(Request::Validate);
        }
        Some(Request::Step(step))
    }

    /// Takes every component's state afresh, clearing the marks and the list's order.
    pub(crate) fn refresh(&mut self, added: Vec<bool>) {
        let current = self.current();
        self.added = added;
        self.checked = true;
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

/// Runs the TUI until the user leaves it, in the look given or the one the terminal gets. Where a
/// present script is super, the first check waits on the sudo dialog.
pub(crate) fn run(catalog: Catalog, runner: Runner, look: Option<Look>) -> Result<ExitCode> {
    let (mut app, first) = if catalog.super_present() {
        (App::unchecked(catalog), Some(Request::Validate))
    } else {
        let added = runner.present_all(&catalog)?;
        (App::new(catalog, added), None)
    };
    let mut terminal = ratatui::init();
    let picker = tui::icon::picker();
    let look = look.unwrap_or_else(|| tui::look::detect(&picker));
    let icons = Icons::load(&app.catalog, &picker, tui::icon::background(&picker), look);
    app.look = look;
    let outcome = drive(&mut terminal, app, &runner, &icons, first);
    ratatui::restore();
    outcome
}

/// The loop: take the run's and sudo's events, draw, then a key.
fn drive(
    terminal: &mut ratatui::DefaultTerminal,
    mut app: App,
    runner: &Runner,
    icons: &Icons,
    first: Option<Request>,
) -> Result<ExitCode> {
    let (sender, events) = mpsc::channel();
    let mut validation: Option<Validation> = None;
    if let Some(request) = first
        && let Some(code) = serve(request, &mut app, runner, &sender, &mut validation)?
    {
        return Ok(code);
    }
    loop {
        while let Ok(event) = events.try_recv() {
            if let Some(request) = app.event(event)
                && let Some(code) = serve(request, &mut app, runner, &sender, &mut validation)?
            {
                return Ok(code);
            }
        }
        let area = terminal.draw(|frame| tui::view::draw(frame, &app, icons)).context(TerminalSnafu)?.area;
        let [_, body, _] = tui::view::regions(area);
        app.body_rows = body.height;
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
        if let Some(request) = app.key(code)
            && let Some(code) = serve(request, &mut app, runner, &sender, &mut validation)?
        {
            return Ok(code);
        }
    }
}

/// Does what a request asks; a quit returns the exit code.
fn serve(
    request: Request,
    app: &mut App,
    runner: &Runner,
    sender: &mpsc::Sender<RunEvent>,
    validation: &mut Option<Validation>,
) -> Result<Option<ExitCode>> {
    match request {
        Request::Quit => return Ok(Some(ExitCode::SUCCESS)),
        Request::Step(i) => {
            if let Some(&Step { action, component }) = app.plan.steps.get(i) {
                runner.spawn_step(&app.catalog.components[component], i, action, sender.clone());
            }
        }
        Request::Check => {
            let added = runner.present_all(&app.catalog)?;
            app.refresh(added);
        }
        Request::Validate => *validation = Some(Validation::start(OsStr::new(sudo::SUDO), sender.clone())),
        Request::Send => {
            if let (Some(validation), Some(sudo)) = (validation.as_mut(), app.sudo.as_mut()) {
                validation.send(&sudo.password.take());
            }
        }
        Request::Cancel => *validation = None,
    }
    Ok(None)
}
