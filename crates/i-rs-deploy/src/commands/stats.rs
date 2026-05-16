use crate::presentation::{print_header, print_warning};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use std::collections::HashMap;

pub fn handle_stats(project: Option<String>, environment: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let mut entries: Vec<_> = store.entries.values().collect();

    if let Some(ref p) = project {
        entries.retain(|e| e.project == *p);
    }
    if let Some(ref e) = environment {
        entries.retain(|r| r.environment == *e);
    }

    if entries.is_empty() {
        print_warning("No deploy records found.");
        return Ok(());
    }

    print_header("Deploy Statistics");

    let total = entries.len();
    println!("{} {}", "Total Deployments:".cyan(), total);

    let mut status_counts: HashMap<String, usize> = HashMap::new();
    for entry in &entries {
        *status_counts.entry(entry.status.to_string()).or_insert(0) += 1;
    }
    println!("\n{} {}", "By Status:".cyan(), "".bold());
    for (status, count) in &status_counts {
        println!("  - {status}: {count}");
    }

    let projects: HashMap<String, usize> = entries.iter().fold(HashMap::new(), |mut acc, e| {
        *acc.entry(e.project.clone()).or_insert(0) += 1;
        acc
    });
    println!("\n{} {}", "By Project:".cyan(), "".bold());
    for (proj, count) in &projects {
        println!("  - {proj}: {count}");
    }

    let envs: HashMap<String, usize> = entries.iter().fold(HashMap::new(), |mut acc, e| {
        *acc.entry(e.environment.clone()).or_insert(0) += 1;
        acc
    });
    println!("\n{} {}", "By Environment:".cyan(), "".bold());
    for (env, count) in &envs {
        println!("  - {env}: {count}");
    }

    let success_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.status.to_string() == "success")
        .collect();
    if !success_entries.is_empty() {
        let latest = success_entries
            .iter()
            .max_by_key(|e| e.deployed_at)
            .expect("non-empty check above");
        println!("\n{} {}", "Latest Success:".cyan(), latest.project);
        println!("  {} {}", "Version:".dimmed(), latest.version);
        println!("  {} {}", "Environment:".dimmed(), latest.environment);
        println!(
            "  {} {}",
            "Deployed At:".dimmed(),
            latest.deployed_at.format("%Y-%m-%d %H:%M")
        );

        let days_since = (Utc::now() - latest.deployed_at).num_days();
        println!("  {} {} days ago", "Ago:".dimmed(), days_since);
    }

    let rollback_count = entries.iter().filter(|e| e.rollback_from.is_some()).count();
    println!("\n{} {}", "Rollback Count:".cyan(), rollback_count);

    Ok(())
}
