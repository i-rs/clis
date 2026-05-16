use crate::models::Milestone;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use clap::Subcommand;
use owo_colors::OwoColorize;

use crate::presentation::OutputFormat;

#[derive(Subcommand, Debug)]
pub enum MilestoneCommand {
    Add {
        #[arg(value_name = "PROJECT")]
        project: String,
        #[arg(value_name = "MILESTONE")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short = 'D', long)]
        due_date: Option<String>,
    },
    Complete {
        #[arg(value_name = "PROJECT")]
        project: String,
        #[arg(value_name = "MILESTONE")]
        name: String,
    },
}

pub fn handle_milestone(command: MilestoneCommand, _format: OutputFormat) -> Result<()> {
    match command {
        MilestoneCommand::Add {
            project,
            name,
            description,
            due_date,
        } => handle_add_milestone(project, name, description, due_date),
        MilestoneCommand::Complete { project, name } => handle_complete_milestone(project, name),
    }
}

fn handle_add_milestone(
    project_name: String,
    name: String,
    description: Option<String>,
    due_date: Option<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let project = match store.get_entry_mut(&project_name) {
        Some(p) => p,
        None => {
            anyhow::bail!("Project '{project_name}' not found");
        }
    };

    if project
        .milestones
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case(&name))
    {
        anyhow::bail!("Milestone '{name}' already exists");
    }

    let due = due_date.and_then(|d| {
        chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d")
            .ok()
            .map(|date| {
                date.and_hms_opt(23, 59, 59)
                    .expect("23:59:59 is always valid")
            })
            .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    });

    let milestone = Milestone {
        name: name.clone(),
        description: description.unwrap_or_default(),
        due_date: due,
        completed: false,
    };

    project.milestones.push(milestone);
    project.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Milestone '{}' added to project '{}'",
        name.green(),
        project_name.green()
    ));

    Ok(())
}

fn handle_complete_milestone(project_name: String, name: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let project = match store.get_entry_mut(&project_name) {
        Some(p) => p,
        None => {
            anyhow::bail!("Project '{project_name}' not found");
        }
    };

    let milestone = match project
        .milestones
        .iter_mut()
        .find(|m| m.name.eq_ignore_ascii_case(&name))
    {
        Some(m) => m,
        None => {
            anyhow::bail!("Milestone '{name}' not found");
        }
    };

    milestone.completed = true;
    project.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!(
        "✓ Milestone '{}' marked as completed in project '{}'",
        name.green(),
        project_name.green()
    ));

    Ok(())
}
