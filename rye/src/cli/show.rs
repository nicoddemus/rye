use std::path::Path;
use std::process::Command;

use anyhow::{bail, Error};
use clap::Parser;
use console::style;

use crate::bootstrap::ensure_self_venv;
use crate::config::load_python_version;
use crate::pyproject::PyProject;
use crate::utils::CommandOutput;

/// Prints the current state of the project.
#[derive(Parser, Debug)]
pub struct Args {
    /// Print the installed dependencies from the venv
    #[arg(long)]
    installed_deps: bool,
}

pub fn execute(cmd: Args) -> Result<(), Error> {
    let project = PyProject::discover()?;

    if cmd.installed_deps {
        return print_installed_deps(&project);
    }

    println!(
        "project: {}",
        style(project.name().unwrap_or("<unnamed>")).yellow()
    );
    println!("path: {}", style(project.root_path().display()).cyan());
    println!("venv: {}", style(project.venv_path().display()).cyan());
    if let Some(ver) = load_python_version() {
        println!("pinned python: {}", style(ver).cyan());
    }

    if let Some(workspace) = project.workspace() {
        println!(
            "workspace: {}",
            style(project.workspace_path().display()).cyan()
        );
        println!("  members:");
        let mut projects = workspace.iter_projects().collect::<Result<Vec<_>, _>>()?;
        projects.sort_by(|a, b| a.root_path().cmp(&b.root_path()));
        for child in projects {
            let root_path = child.root_path();
            let rel_path = Path::new(".").join(
                root_path
                    .strip_prefix(project.workspace_path())
                    .unwrap_or(&root_path),
            );
            println!(
                "    {} ({})",
                style(child.name().unwrap_or("<unnamed>")).cyan(),
                style(rel_path.display()).dim(),
            );
        }
    }

    Ok(())
}

fn print_installed_deps(project: &PyProject) -> Result<(), Error> {
    let python = project.venv_bin_path().join("python");
    if !python.is_file() {
        return Ok(());
    }
    let self_venv = ensure_self_venv(CommandOutput::Normal)?;

    let status = Command::new(self_venv.join("bin/pip"))
        .arg("--python")
        .arg(&python)
        .arg("freeze")
        .env("PYTHONWARNINGS", "ignore")
        .status()?;

    if !status.success() {
        bail!("failed to print dependencies via pip");
    }

    Ok(())
}
