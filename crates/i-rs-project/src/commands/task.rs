use crate::models::Task;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use clap::Subcommand;
use owo_colors::OwoColorize;

#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    Add {
        #[arg(value_name = "PROJECT")]
        project: String,
        #[arg(value_name = "TASK")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
    },
    Complete {
        #[arg(value_name = "PROJECT")]
        project: String,
        #[arg(value_name = "TASK")]
        name: String,
    },
}

pub fn handle_task(command: TaskCommand) -> Result<()> {
    match command {
        TaskCommand::Add { project, name, description, tag } => {
            handle_add_task(project, name, description, tag)
        }
        TaskCommand::Complete { project, name } => {
            handle_complete_task(project, name)
        }
    }
}

fn handle_add_task(
    project_name: String,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let project = match storage::find_project_mut(&mut store, &project_name) {
        Some(p) => p,
        None => {
            anyhow::bail!("Project '{project_name}' not found");
        }
    };

    if project.tasks.iter().any(|t| t.name.eq_ignore_ascii_case(&name)) {
        anyhow::bail!("Task '{name}' already exists");
    }

    let task = Task {
        name: name.clone(),
        description: description.unwrap_or_default(),
        completed: false,
        tags,
    };

    project.tasks.push(task);
    project.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!("✓ Task '{}' added to project '{}'", name.green(), project_name.green()));

    Ok(())
}

fn handle_complete_task(project_name: String, name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let project = match storage::find_project_mut(&mut store, &project_name) {
        Some(p) => p,
        None => {
            anyhow::bail!("Project '{project_name}' not found");
        }
    };

    let task = match project.tasks.iter_mut().find(|t| t.name.eq_ignore_ascii_case(&name)) {
        Some(t) => t,
        None => {
            anyhow::bail!("Task '{name}' not found");
        }
    };

    task.completed = true;
    project.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Task '{}' marked as completed in project '{}'",
        name.green(),
        project_name.green()
    ));

    Ok(())
}
