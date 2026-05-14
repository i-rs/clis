use crate::presentation::{print_error, print_header};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_stats() -> Result<()> {
    let store = storage::load_store()?;

    if store.projects.is_empty() {
        print_error("No projects found");
        anyhow::bail!("No projects to show statistics");
    }

    let total_projects = store.projects.len();
    let active_projects = store.projects.iter().filter(|p| p.status == crate::models::ProjectStatus::Active).count();
    let completed_projects = store.projects.iter().filter(|p| p.status == crate::models::ProjectStatus::Completed).count();
    let onhold_projects = store.projects.iter().filter(|p| p.status == crate::models::ProjectStatus::OnHold).count();
    
    let total_milestones: usize = store.projects.iter().map(|p| p.milestones.len()).sum();
    let completed_milestones: usize = store.projects.iter().map(|p| p.milestones.iter().filter(|m| m.completed).count()).sum();
    
    let total_tasks: usize = store.projects.iter().map(|p| p.tasks.len()).sum();
    let completed_tasks: usize = store.projects.iter().map(|p| p.tasks.iter().filter(|t| t.completed).count()).sum();

    let urgent_projects = store.projects.iter().filter(|p| p.priority == crate::models::Priority::Urgent).count();
    let high_priority_projects = store.projects.iter().filter(|p| p.priority == crate::models::Priority::High).count();

    let overdue_milestones = store.projects.iter().flat_map(|p| &p.milestones)
        .filter(|m| !m.completed && m.due_date.map_or(false, |d| d < Utc::now()))
        .count();

    println!();
    print_header("Project Statistics");
    println!();
    
    println!("{}", "Projects:".bold().cyan());
    println!("  {:20} {}", "Total:".dimmed(), total_projects.to_string().green());
    println!("  {:20} {}", "Active:".dimmed(), active_projects.to_string().green());
    println!("  {:20} {}", "Completed:".dimmed(), completed_projects.to_string().green());
    println!("  {:20} {}", "On Hold:".dimmed(), onhold_projects.to_string().green());
    
    println!();
    println!("{}", "Priority:".bold().cyan());
    println!("  {:20} {}", "Urgent:".dimmed(), urgent_projects.to_string().red());
    println!("  {:20} {}", "High:".dimmed(), high_priority_projects.to_string().yellow());
    
    println!();
    println!("{}", "Milestones:".bold().cyan());
    println!("  {:20} {}", "Total:".dimmed(), total_milestones.to_string().green());
    println!("  {:20} {}", "Completed:".dimmed(), completed_milestones.to_string().green());
    println!("  {:20} {}", "Overdue:".dimmed(), overdue_milestones.to_string().red());
    
    println!();
    println!("{}", "Tasks:".bold().cyan());
    println!("  {:20} {}", "Total:".dimmed(), total_tasks.to_string().green());
    println!("  {:20} {}", "Completed:".dimmed(), completed_tasks.to_string().green());

    if total_milestones > 0 {
        let milestone_progress = (completed_milestones as f64 / total_milestones as f64 * 100.0) as usize;
        println!();
        println!("{}", "Progress:".bold().cyan());
        println!("  {:20} {}% (milestones)", "Completion:".dimmed(), milestone_progress);
    }

    if total_tasks > 0 {
        let task_progress = (completed_tasks as f64 / total_tasks as f64 * 100.0) as usize;
        println!("  {:20} {}% (tasks)", "".dimmed(), task_progress);
    }

    println!();
    Ok(())
}
