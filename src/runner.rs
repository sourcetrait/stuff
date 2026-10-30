//! The scripts run: nu with the user's env and config, from each component's directory.

use crate::*;

/// The variable naming the stuff directory to a script.
pub(crate) const ENV_STUFF_DIR: &str = "STUFF_DIR";
/// The variable naming the script's component.
pub(crate) const ENV_COMPONENT_ID: &str = "COMPONENT_ID";
/// The variable naming the script's component directory.
pub(crate) const ENV_COMPONENT_DIR: &str = "COMPONENT_DIR";

/// A component's script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Script {
    /// Exits 0 when the component is added, 1 when not.
    Exists,
    /// Adds the component.
    Add,
    /// Removes the component.
    Remove,
}

impl Script {
    /// The script's file name.
    pub(crate) fn file(self) -> &'static str {
        match self {
            Self::Exists => "exists.nu",
            Self::Add => "add.nu",
            Self::Remove => "remove.nu",
        }
    }
}

/// What a run tells the TUI as it goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RunEvent {
    /// A step begins, by its index in the plan.
    Begin { step: usize },
    /// A script wrote to stdout or stderr.
    Output { bytes: Vec<u8> },
    /// A step failed and the run stopped.
    Fail { message: String },
    /// The run ended.
    Finish,
}

/// The kind of a [`RunEvent`].
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunEventKind {
    Begin,
    Output,
    Fail,
    Finish,
}

impl RunEvent {
    /// The event's kind.
    #[allow(dead_code)]
    pub(crate) fn kind(&self) -> RunEventKind {
        match self {
            Self::Begin { .. } => RunEventKind::Begin,
            Self::Output { .. } => RunEventKind::Output,
            Self::Fail { .. } => RunEventKind::Fail,
            Self::Finish => RunEventKind::Finish,
        }
    }
}

/// Starts nu for the components' scripts.
#[derive(Debug, Clone)]
pub(crate) struct Runner {
    /// The nu program.
    nu: OsString,
    /// nu's flags: the user's env and config, or none.
    flags: Vec<OsString>,
    /// The stuff directory, absolute.
    dir: PathBuf,
}

impl Runner {
    /// The runner for a stuff directory, with the user's own nushell config.
    pub(crate) fn new(dir: &Path) -> Self {
        let nu = OsString::from("nu");
        let flags = config_flags(&nu);
        Self { nu, flags, dir: dir.to_path_buf() }
    }

    /// The command running a component's script.
    pub(crate) fn command(&self, component: &Component, script: Script) -> Command {
        let mut command = Command::new(&self.nu);
        command
            .args(&self.flags)
            .arg(component.dir.join(script.file()))
            .current_dir(&component.dir)
            .env(ENV_STUFF_DIR, &self.dir)
            .env(ENV_COMPONENT_ID, &component.id)
            .env(ENV_COMPONENT_DIR, &component.dir)
            .stdin(Stdio::null());
        command
    }

    /// Whether a component is added, as its exists script reports.
    pub(crate) fn exists(&self, component: &Component) -> RunnerResult<bool> {
        let output = self
            .command(component, Script::Exists)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .context(SpawnSnafu { component: &component.id, script: Script::Exists.file() })?;
        match output.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => ScriptSnafu {
                component: &component.id,
                script: Script::Exists.file(),
                reason: format!(
                    "exited {}: {}",
                    describe(output.status),
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            }
            .fail(),
        }
    }

    /// Every component's exists, run at once.
    pub(crate) fn exists_all(&self, catalog: &Catalog) -> RunnerResult<Vec<bool>> {
        thread::scope(|scope| {
            let checks: Vec<_> = catalog
                .components
                .iter()
                .map(|component| scope.spawn(move || self.exists(component)))
                .collect();
            checks.into_iter().map(|check| check.join().expect("an exists check panicked")).collect()
        })
    }

    /// Runs a step with its output passed through to this process's own.
    pub(crate) fn step_inherit(&self, component: &Component, action: StepAction) -> RunnerResult<()> {
        let script = action.script();
        let status = self
            .command(component, script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context(SpawnSnafu { component: &component.id, script: script.file() })?;
        self.settle(component, action, status)
    }

    /// Runs a step with its output sent to the TUI.
    pub(crate) fn step_piped(
        &self,
        component: &Component,
        action: StepAction,
        events: &mpsc::Sender<RunEvent>,
    ) -> RunnerResult<()> {
        let script = action.script();
        let spawn = SpawnSnafu { component: &component.id, script: script.file() };
        let mut child = self
            .command(component, script)
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

    /// Runs a plan on a thread of its own, its progress sent to the TUI.
    pub(crate) fn spawn_plan(
        &self,
        catalog: &Catalog,
        plan: &Plan,
        events: mpsc::Sender<RunEvent>,
    ) -> thread::JoinHandle<()> {
        let runner = self.clone();
        let catalog = catalog.clone();
        let plan = plan.clone();
        thread::spawn(move || {
            for (step, &Step { action, component }) in plan.steps.iter().enumerate() {
                let _ = events.send(RunEvent::Begin { step });
                let result = runner.step_piped(&catalog.components[component], action, &events);
                if let Err(error) = result {
                    let _ = events.send(RunEvent::Fail { message: error.to_string() });
                    break;
                }
            }
            let _ = events.send(RunEvent::Finish);
        })
    }

    /// Checks a step's exit, then that exists agrees with what ran.
    fn settle(&self, component: &Component, action: StepAction, status: ExitStatus) -> RunnerResult<()> {
        if !status.success() {
            return ScriptSnafu {
                component: &component.id,
                script: action.script().file(),
                reason: format!("exited {}", describe(status)),
            }
            .fail();
        }
        let added = self.exists(component)?;
        if added != (action == StepAction::Add) {
            let state = if added { "added" } else { "not added" };
            return ScriptSnafu {
                component: &component.id,
                script: Script::Exists.file(),
                reason: format!("reports {state} after {}", action.label()),
            }
            .fail();
        }
        Ok(())
    }
}

/// Sends a stream's bytes to the TUI until it closes.
fn forward(mut stream: Box<dyn Read + Send>, events: &mpsc::Sender<RunEvent>) {
    let mut buffer = [0u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                if events.send(RunEvent::Output { bytes: buffer[..count].to_vec() }).is_err() {
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
