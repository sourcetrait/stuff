//! The actions run: a component's mod.nu as an overlay in nu, with the user's env and config.

use crate::*;

/// A health check after its overlay: the action called, then its record's outer shape checked; the
/// record's form follows.
const HEALTH: &str = "let health = (component health)
let shape = ($health | describe)
if (not ($shape | str starts-with 'record')) {
    error make --unspanned {msg: $'returned ($shape), not record<component: path, present: oneof<bool, record>>'}
}
if (($health | columns | sort) != [component present]) or (($health.component | describe) != 'string') {
    error make --unspanned {msg: $'returned ($shape), not record<component: path, present: oneof<bool, record>>'}
}
let present = ($health.present | describe)
if (not ($present == 'bool' or ($present | str starts-with 'record'))) {
    error make --unspanned {msg: $'returned present as ($present), not a bool or a record'}
}
";

/// What a module's contract check prints after its overlay: its exports' names, one a line.
const EXPORTS: &str = "scope commands | where name starts-with 'component ' | get name | str join (char nl)";

/// The name of a contract check in an error: what it checks.
const CONTRACT: &str = "mod.nu";

/// The form a health record is given in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HealthForm {
    /// NUON, for the command line.
    Nuon,
    /// nu's expanded table, its colors kept, for the TUI's page.
    Table {
        /// The columns the table may take.
        width: u16,
    },
}

/// What a run and sudo tell the TUI as they go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RunEvent {
    /// A step begins, by its index in the plan.
    Begin { step: usize },
    /// An action wrote to stdout or stderr.
    Output { bytes: Vec<u8> },
    /// A step failed and the run stopped.
    Fail { message: String },
    /// A step ended well, by its index in the plan.
    Done { step: usize },
    /// sudo wrote its prompt or a message.
    SudoOutput { bytes: Vec<u8> },
    /// sudo ended: whether the credential is cached.
    SudoExit { ok: bool },
    /// A component's health check ended: nu's table of its record, or the failure.
    Health { component: usize, table: std::result::Result<String, String> },
}

/// The kind of a [`RunEvent`].
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunEventKind {
    Begin,
    Output,
    Fail,
    Done,
    SudoOutput,
    SudoExit,
    Health,
}

impl RunEvent {
    /// The event's kind.
    #[allow(dead_code)]
    pub(crate) fn kind(&self) -> RunEventKind {
        match self {
            Self::Begin { .. } => RunEventKind::Begin,
            Self::Output { .. } => RunEventKind::Output,
            Self::Fail { .. } => RunEventKind::Fail,
            Self::Done { .. } => RunEventKind::Done,
            Self::SudoOutput { .. } => RunEventKind::SudoOutput,
            Self::SudoExit { .. } => RunEventKind::SudoExit,
            Self::Health { .. } => RunEventKind::Health,
        }
    }
}

/// Starts nu for the components' actions.
#[derive(Debug, Clone)]
pub(crate) struct Runner {
    /// The nu program.
    nu: OsString,
    /// nu's flags: the user's env and config, or none.
    flags: Vec<OsString>,
}

impl Runner {
    /// The runner, with the user's own nushell config.
    pub(crate) fn new() -> Self {
        let nu = OsString::from("nu");
        let flags = config_flags(&nu);
        Self { nu, flags }
    }

    /// The command calling a component's action.
    pub(crate) fn command(&self, component: &Component, action: Action) -> Command {
        self.program(component, &format!("component {}\n", action.name()))
    }

    /// nu with the user's config, from a component's directory, running a program after the
    /// component's overlay.
    fn program(&self, component: &Component, program: &str) -> Command {
        let mut command = Command::new(&self.nu);
        command
            .args(&self.flags)
            .current_dir(&component.dir)
            .stdin(Stdio::null())
            .arg("-c")
            .arg(format!("{}{program}", overlay(component)));
        command
    }

    /// A component's health record, in the form asked for.
    pub(crate) fn health(&self, component: &Component, form: HealthForm) -> RunnerResult<String> {
        let program = match form {
            HealthForm::Nuon => format!("{HEALTH}$health | to nuon\n"),
            HealthForm::Table { width } => format!(
                "{HEALTH}$env.config.use_ansi_coloring = true\n$health | table --expand --width {width}\n"
            ),
        };
        self.answer(component, Action::Health.name(), &program)
    }

    /// Whether a component is added, as its present answers. A failure is a failure, never an
    /// answer.
    pub(crate) fn present(&self, component: &Component) -> RunnerResult<bool> {
        let action = Action::Present.name();
        let text = self.answer(component, action, "component present | to nuon\n")?;
        match text.lines().last().map(str::trim) {
            Some("true") => Ok(true),
            Some("false") => Ok(false),
            _ => ActionSnafu {
                component: &component.id,
                action,
                reason: format!("answered {text:?}, not a bool"),
            }
            .fail(),
        }
    }

    /// What a program after a component's overlay writes to stdout, where it succeeds.
    fn answer(&self, component: &Component, action: &str, program: &str) -> RunnerResult<String> {
        let output = self
            .program(component, program)
            .output()
            .context(SpawnSnafu { component: &component.id, action })?;
        if !output.status.success() {
            return ActionSnafu {
                component: &component.id,
                action,
                reason: format!(
                    "exited {}: {}",
                    describe(output.status),
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            }
            .fail();
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
    }

    /// Holds a component's module to its component.toml: it parses, it exports every action listed,
    /// and it exports nothing else.
    pub(crate) fn contract(&self, component: &Component) -> RunnerResult<()> {
        let text = self.answer(component, CONTRACT, &format!("{EXPORTS}\n"))?;
        let exports: Vec<&str> = text.lines().filter_map(|line| line.trim().strip_prefix("component ")).collect();
        let listed = component.actions.names();
        let broken = if let Some(missing) = listed.iter().find(|name| !exports.contains(name)) {
            format!("doesn't export {missing}, which actions lists")
        } else if let Some(extra) = exports.iter().find(|name| !listed.contains(name)) {
            format!("exports {extra}, which actions doesn't list")
        } else {
            return Ok(());
        };
        ActionSnafu { component: &component.id, action: CONTRACT, reason: broken }.fail()
    }

    /// Every component's module held to its component.toml, at once.
    pub(crate) fn contract_all(&self, catalog: &Catalog) -> RunnerResult<()> {
        thread::scope(|scope| {
            let running: Vec<_> =
                catalog.components.iter().map(|component| scope.spawn(move || self.contract(component))).collect();
            running
                .into_iter()
                .map(|check| check.join().expect("a contract check panicked"))
                .collect::<RunnerResult<Vec<()>>>()
        })?;
        Ok(())
    }

    /// Every component's present: the plain ones at once, then the super ones one at a time, so
    /// only the first can ask for the password.
    pub(crate) fn present_all(&self, catalog: &Catalog) -> RunnerResult<Vec<bool>> {
        let components = &catalog.components;
        let (supers, plain): (Vec<usize>, Vec<usize>) =
            (0..components.len()).partition(|&i| components[i].supers.present);
        let checks: Vec<(usize, RunnerResult<bool>)> = thread::scope(|scope| {
            let running: Vec<_> =
                plain.iter().map(|&i| (i, scope.spawn(move || self.present(&components[i])))).collect();
            running
                .into_iter()
                .map(|(i, check)| (i, check.join().expect("a present check panicked")))
                .collect()
        });
        let mut added = vec![false; components.len()];
        for (i, check) in checks {
            added[i] = check?;
        }
        for i in supers {
            added[i] = self.present(&components[i])?;
        }
        Ok(added)
    }

    /// Runs a step with its output passed through to this process's own.
    pub(crate) fn step_inherit(&self, component: &Component, action: StepAction) -> RunnerResult<()> {
        let status = self
            .command(component, action.action())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context(SpawnSnafu { component: &component.id, action: action.label() })?;
        self.settle(component, action, status)
    }

    /// Runs a step with its output sent to the TUI.
    pub(crate) fn step_piped(
        &self,
        component: &Component,
        action: StepAction,
        events: &mpsc::Sender<RunEvent>,
    ) -> RunnerResult<()> {
        let spawn = SpawnSnafu { component: &component.id, action: action.label() };
        let mut child = self
            .command(component, action.action())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(spawn)?;
        let mut streams: Vec<Box<dyn Read + Send>> = Vec::new();
        if let Some(stdout) = child.stdout.take() {
            streams.push(Box::new(stdout));
        }
        if let Some(stderr) = child.stderr.take() {
            streams.push(Box::new(stderr));
        }
        let status = thread::scope(|scope| {
            for stream in streams {
                let events = events.clone();
                scope.spawn(move || forward(stream, &events));
            }
            child.wait()
        })
        .context(spawn)?;
        self.settle(component, action, status)
    }

    /// Checks a component's health on a thread of its own, nu's table of it sent to the TUI.
    pub(crate) fn spawn_health(
        &self,
        component: &Component,
        index: usize,
        width: u16,
        events: mpsc::Sender<RunEvent>,
    ) -> thread::JoinHandle<()> {
        let runner = self.clone();
        let component = component.clone();
        thread::spawn(move || {
            let table = runner.health(&component, HealthForm::Table { width }).map_err(|error| error.to_string());
            let _ = events.send(RunEvent::Health { component: index, table });
        })
    }

    /// Runs one step on a thread of its own, its progress sent to the TUI.
    pub(crate) fn spawn_step(
        &self,
        component: &Component,
        step: usize,
        action: StepAction,
        events: mpsc::Sender<RunEvent>,
    ) -> thread::JoinHandle<()> {
        let runner = self.clone();
        let component = component.clone();
        thread::spawn(move || {
            let _ = events.send(RunEvent::Begin { step });
            let event = match runner.step_piped(&component, action, &events) {
                Ok(()) => RunEvent::Done { step },
                Err(error) => RunEvent::Fail { message: error.to_string() },
            };
            let _ = events.send(event);
        })
    }

    /// Checks a step's exit, then that present agrees with what ran.
    fn settle(&self, component: &Component, action: StepAction, status: ExitStatus) -> RunnerResult<()> {
        if !status.success() {
            return ActionSnafu {
                component: &component.id,
                action: action.label(),
                reason: format!("exited {}", describe(status)),
            }
            .fail();
        }
        let added = self.present(component)?;
        if added != action.leaves_added() {
            let state = if added { "added" } else { "not added" };
            return ActionSnafu {
                component: &component.id,
                action: Action::Present.name(),
                reason: format!("reports {state} after {}", action.label()),
            }
            .fail();
        }
        Ok(())
    }
}

/// A component's module taken as an overlay, its exports `component present` and so on.
fn overlay(component: &Component) -> String {
    let dir = run::nuon_string(&component.dir.display().to_string());
    format!("overlay use --prefix {dir} as component\n")
}

/// Sends a stream's bytes to the TUI until it closes.
fn forward(stream: Box<dyn Read + Send>, events: &mpsc::Sender<RunEvent>) {
    forward_as(stream, events, |bytes| RunEvent::Output { bytes });
}

/// Sends a stream's bytes to the TUI as the event given, until it closes.
pub(crate) fn forward_as(
    mut stream: Box<dyn Read + Send>,
    events: &mpsc::Sender<RunEvent>,
    event: fn(Vec<u8>) -> RunEvent,
) {
    let mut buffer = [0u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                if events.send(event(buffer[..count].to_vec())).is_err() {
                    break;
                }
            }
        }
    }
}

/// The user's env and config as nu flags: each file nu names that exists.
fn config_flags(nu: &OsStr) -> Vec<OsString> {
    let query = Command::new(nu)
        .args(["-n", "-c", "print $nu.env-path; print $nu.config-path"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    let mut flags: Vec<OsString> = Vec::new();
    if let Ok(output) = query {
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        for (flag, line) in ["--env-config", "--config"].into_iter().zip(text.lines()) {
            let path = line.trim();
            if !path.is_empty() && Path::new(path).is_file() {
                flags.push(flag.into());
                flags.push(path.into());
            }
        }
    }
    if flags.is_empty() {
        flags.push("-n".into());
    }
    flags
}

/// An exit status as an error shows it.
fn describe(status: ExitStatus) -> String {
    status.code().map_or_else(|| "by a signal".to_string(), |code| code.to_string())
}
