use crate::models::ProjectDetail;
use crate::presentation::{output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let project = match store.get_entry(&name) {
        Some(p) => p,
        None => {
            anyhow::bail!("Project '{name}' not found");
        }
    };

    if format.is_json() {
        let detail = ProjectDetail::from(project);
        println!("{}", output_item(&detail, format));
        return Ok(());
    }

    println!("\n{}", format!("Project: {}", project.name.green().bold()));
    println!("{}", "=".repeat(50));
    println!("  {:12} {}", "Status:".dimmed(), format!("{:?}", project.status).to_lowercase());
    println!("  {:12} {}", "Priority:".dimmed(), format!("{:?}", project.priority).to_lowercase());
    println!("  {:12} {}", "Created:".dimmed(), project.created_at.format("%Y-%m-%d %H:%M"));
    println!("  {:12} {}", "Updated:".dimmed(), project.updated_at.format("%Y-%m-%d %H:%M"));

    if !project.description.is_empty() {
        println!("\n{}", "Description:".dimmed());
        println!("  {}", project.description);
    }

    if !project.tags.is_empty() {
        println!("\n{}", "Tags:".dimmed());
        println!("  {}", project.tags.join(", "));
    }

    if !project.remark.is_empty() {
        println!("\n{}", "Remarks:".dimmed());
        for r in &project.remark {
            println!("  - {r}");
        }
    }

    if project.milestones.is_empty() {
        println!("\n{}", "Milestones:".bold().cyan());
        println!("  {}", "No milestones yet".dimmed());
    } else {
        println!("\n{}", "Milestones:".bold().cyan());
        let completed = project.milestones.iter().filter(|m| m.completed).count();
        println!("  {:12} {}/{} completed", "Progress:".dimmed(), completed, project.milestones.len());
        
        for (i, milestone) in project.milestones.iter().enumerate() {
            let status = if milestone.completed { "✓" } else { "○" };
            let due = milestone.due_date
                .map(|d| {
                    let days = (d.date_naive() - Utc::now().date_naive()).num_days();
                    if days < 0 {
                        format!(" ({} days overdue)", -days)
                    } else if days == 0 {
                        " (due today)".to_string()
                    } else {
                        format!(" ({days} days left)")
                    }
                })
                .unwrap_or_default();
            
            println!("  {}. {} {}{}", i + 1, status, milestone.name, due);
            if !milestone.description.is_empty() {
                println!("     {}", milestone.description.dimmed());
            }
        }
    }

    if project.tasks.is_empty() {
        println!("\n{}", "Tasks:".bold().cyan());
        println!("  {}", "No tasks yet".dimmed());
    } else {
        println!("\n{}", "Tasks:".bold().cyan());
        let completed = project.tasks.iter().filter(|t| t.completed).count();
        println!("  {:12} {}/{} completed", "Progress:".dimmed(), completed, project.tasks.len());
        
        for (i, task) in project.tasks.iter().enumerate() {
            let status = if task.completed { "✓" } else { "○" };
            println!("  {}. {} {}", i + 1, status, task.name);
            if !task.description.is_empty() {
                println!("     {}", task.description.dimmed());
            }
            if !task.tags.is_empty() {
                println!("     Tags: {}", task.tags.join(", ").dimmed());
            }
        }
    }

    Ok(())
}
