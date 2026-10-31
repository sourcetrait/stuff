//! The command's run: the source loaded, then the TUI or the CLI's action.

use crate::*;

/// Runs the stuff command, returning its exit code.
pub fn run() -> ExitCode {
    let cli = <Cli as clap::Parser>::parse();
    match dispatch(cli) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("stuff: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Reports on the terminal, or loads the stuff, then runs the TUI or the action named.
fn dispatch(cli: Cli) -> Result<ExitCode> {
    if cli.probe {
        return tui::look::probe();
    }
    let source = cli.source.unwrap_or_default();
    if !crate::catalog::holds_stuff(&source) {
        return UnknownStuffSnafu { name: source.display().to_string() }.fail();
    }
    let catalog = Catalog::load(&source)?;
    let runner = Runner::new(&catalog.dir);
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
        list(&catalog, &added, kind == CliActionKind::Remove);
        return Ok(ExitCode::SUCCESS);
    }
    let picked = catalog.resolve(action.components())?;
    let plan = match kind {
        CliActionKind::Add => Plan::new(&catalog, &added, &picked, &[]),
        _ => Plan::new(&catalog, &added, &[], &picked),
    };
    execute(&catalog, &runner, &plan, kind)?;
    Ok(ExitCode::SUCCESS)
}

/// The health of the components picked, as NUON: a record for one, a record of records keyed by
/// name for several. A component without a health script reports its present script's answer.
fn health(catalog: &Catalog, runner: &Runner, picked: &[usize]) -> Result<String> {
    let mut records: Vec<(&str, String)> = Vec::with_capacity(picked.len());
    for &i in picked {
        let component = &catalog.components[i];
        if records.iter().any(|(id, _)| *id == component.id) {
            continue;
        }
        let record = if component.health {
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

/// A string as NUON writes it quoted.
fn nuon_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The warning a removal gives where its component has no remove script: yellow `[stuff]`, then
/// the component's name.
pub(crate) fn unsupported_remove(id: &str) -> String {
    format!("\x1b[33m[stuff]\x1b[0m remove not supported: {id}")
}

/// Prints the components added, or those not, one a line.
fn list(catalog: &Catalog, added: &[bool], want_added: bool) {
    let shown: Vec<usize> =
        catalog.display_order(added).into_iter().filter(|&i| added[i] == want_added).collect();
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
            _ => "remove",
        };
        println!("nothing to {verb}");
        return Ok(());
    }
    for step in &plan.steps {
        println!("[{}] {}", step.action.label(), catalog.components[step.component].id);
    }
    let mut unsupported = Vec::new();
    let mut outcome = Ok(());
    for step in &plan.steps {
        let component = &catalog.components[step.component];
        if step.action == StepAction::Remove && !component.removable {
            unsupported.push(component.id.as_str());
            continue;
        }
        println!("== {} {}", step.action.label(), component.id);
        if let Err(error) = runner.step_inherit(component, step.action) {
            outcome = Err(error.into());
            break;
        }
    }
    if outcome.is_ok() {
        println!("done");
    }
    for id in unsupported {
        eprintln!("{}", unsupported_remove(id));
    }
    outcome
}
