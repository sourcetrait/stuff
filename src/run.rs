//! The command's run: the source loaded, then the TUI or the CLI's action.

use crate::*;

/// Runs the stuff command, returning its exit code.
pub fn run() -> ExitCode {
    let cli = <Cli as clap::Parser>::parse();
    match dispatch(cli) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{}", line(Tone::Error, &error.to_string()));
            ExitCode::FAILURE
        }
    }
}

/// The tone of a user-facing line, its `[stuff]`'s color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    /// Blue: what is happening.
    Info,
    /// Green: done, and done well.
    Success,
    /// Red: what failed, on stderr.
    Error,
    /// Yellow: a warning on stderr, or the line before a prompt.
    Attention,
}

/// A user-facing line: `[stuff]` in its tone's color, then the message.
pub(crate) fn line(tone: Tone, message: &str) -> String {
    let color = match tone {
        Tone::Info => 34,
        Tone::Success => 32,
        Tone::Error => 31,
        Tone::Attention => 33,
    };
    format!("\x1b[{color}m[stuff]\x1b[0m {message}")
}

/// What a successful run reports just before its success.
pub(crate) fn fresh_login() -> String {
    line(Tone::Info, "Environment changes require a fresh terminal login")
}

/// Reports on the terminal, or loads the stuff, then runs the TUI or the action named.
fn dispatch(cli: Cli) -> Result<ExitCode> {
    if cli.probe {
        return tui::look::probe();
    }
    let source = Source::parse(&cli.source.unwrap_or_default()).directory()?;
    let catalog = Catalog::load(&source)?;
    let runner = Runner::new();
    runner.contract_all(&catalog)?;
    let Some(action) = cli.action else {
        return tui::app::run(catalog, runner, cli.look);
    };
    let kind = action.kind();
    if kind == CliActionKind::Health {
        let picked = catalog.resolve(action.components())?;
        println!("{}", health(&catalog, &runner, &picked)?);
        return Ok(ExitCode::SUCCESS);
    }
    let added = runner.present_all(&catalog)?;
    if action.components().is_empty() {
        list(&catalog, &added, kind);
        return Ok(ExitCode::SUCCESS);
    }
    let picked = catalog.resolve(action.components())?;
    let plan = match kind {
        CliActionKind::Add => Plan::new(&catalog, &added, &picked, &[], &[]),
        CliActionKind::Update => Plan::new(&catalog, &added, &[], &[], &picked),
        _ => Plan::new(&catalog, &added, &[], &picked, &[]),
    };
    execute(&catalog, &runner, &plan, kind)?;
    Ok(ExitCode::SUCCESS)
}

/// The health of the components picked, as NUON: a record for one, a record of records keyed by
/// name for several. A component without a health action reports its present's answer.
fn health(catalog: &Catalog, runner: &Runner, picked: &[usize]) -> Result<String> {
    let mut records: Vec<(&str, String)> = Vec::with_capacity(picked.len());
    for &i in picked {
        let component = &catalog.components[i];
        if records.iter().any(|(id, _)| *id == component.id) {
            continue;
        }
        let record = if component.actions.health {
            runner.health(component, HealthForm::Nuon)?
        } else {
            let present = runner.present(component)?;
            format!("{{component: {}, present: {present}}}", nuon_string(&component.id))
        };
        records.push((component.id.as_str(), record));
    }
    if let [(_, record)] = records.as_slice() {
        return Ok(record.clone());
    }
    let fields: Vec<String> = records.iter().map(|(id, record)| format!("{}: {record}", nuon_string(id))).collect();
    Ok(format!("{{{}}}", fields.join(", ")))
}

/// A string as NUON and nu's own double quotes write it.
pub(crate) fn nuon_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The warning a step gives where its component doesn't offer its action.
pub(crate) fn unsupported(action: StepAction, id: &str) -> String {
    line(Tone::Attention, &format!("{} not supported: {id}", action.label()))
}

/// Prints what an action with no names would accept, one a line: those not added, those added, or
/// those added that can be updated.
fn list(catalog: &Catalog, added: &[bool], kind: CliActionKind) {
    let accepted = |i: usize| match kind {
        CliActionKind::Add => !added[i],
        CliActionKind::Update => added[i] && catalog.components[i].actions.update,
        _ => added[i],
    };
    let shown: Vec<usize> = catalog.display_order(added).into_iter().filter(|&i| accepted(i)).collect();
    let width = shown.iter().map(|&i| catalog.components[i].id.len()).max().unwrap_or(0);
    for i in shown {
        let component = &catalog.components[i];
        let line = format!("{:width$}  {}  {}", component.id, component.title, component.description);
        println!("{}", line.trim_end());
    }
}

/// Runs a plan's steps in order, their output passed through.
fn execute(catalog: &Catalog, runner: &Runner, plan: &Plan, kind: CliActionKind) -> Result<()> {
    if plan.is_empty() {
        let verb = match kind {
            CliActionKind::Add => "add",
            CliActionKind::Update => "update",
            _ => "remove",
        };
        println!("{}", line(Tone::Info, &format!("nothing to {verb}")));
        return Ok(());
    }
    for step in &plan.steps {
        let message = format!("plan: {} {}", step.action.label(), catalog.components[step.component].id);
        println!("{}", line(Tone::Info, &message));
    }
    let mut passed_over = Vec::new();
    let mut outcome = Ok(());
    for step in &plan.steps {
        let component = &catalog.components[step.component];
        if !component.actions.has(step.action.action()) {
            passed_over.push((step.action, component.id.as_str()));
            continue;
        }
        println!("{}", line(Tone::Info, &format!("{} {}", step.action.label(), component.id)));
        if let Err(error) = runner.step_inherit(component, step.action) {
            outcome = Err(error.into());
            break;
        }
    }
    if outcome.is_ok() {
        println!("{}", fresh_login());
        println!("{}", line(Tone::Success, "done"));
    }
    for (action, id) in passed_over {
        eprintln!("{}", unsupported(action, id));
    }
    outcome
}
