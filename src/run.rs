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
    let added = runner.present_all(&catalog)?;
    let kind = action.kind();
    if action.components().is_empty() {
        list(&catalog, &added, kind == CliActionKind::Remove);
        return Ok(ExitCode::SUCCESS);
    }
    let picked = catalog.resolve(action.components())?;
    let plan = match kind {
        CliActionKind::Add => Plan::new(&catalog, &added, &picked, &[]),
        CliActionKind::Remove => Plan::new(&catalog, &added, &[], &picked),
    };
    execute(&catalog, &runner, &plan, kind)?;
    Ok(ExitCode::SUCCESS)
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
            CliActionKind::Remove => "remove",
        };
        println!("nothing to {verb}");
        return Ok(());
    }
    for step in &plan.steps {
        println!("[{}] {}", step.action.label(), catalog.components[step.component].id);
    }
    for step in &plan.steps {
        let component = &catalog.components[step.component];
        println!("== {} {}", step.action.label(), component.id);
        runner.step_inherit(component, step.action)?;
    }
    println!("done");
    Ok(())
}
