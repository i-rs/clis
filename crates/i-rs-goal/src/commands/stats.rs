use crate::models::SavingsGoal;
use crate::presentation::{print_header, OutputFormat};
use crate::storage;
use clap::Parser;
use owo_colors::OwoColorize;
use std::collections::HashMap;

#[derive(Parser, Debug)]
pub struct StatsArgs {
    #[arg(short, long, help = "Filter by tag")]
    pub tag: Option<String>,
}

pub fn stats(args: StatsArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;
    
    let goals: Vec<&SavingsGoal> = match &args.tag {
        Some(tag) => store.goals.iter().filter(|g| g.tags.contains(tag)).collect(),
        None => store.goals.iter().collect(),
    };
    
    if goals.is_empty() {
        println!("No goals found.");
        return Ok(());
    }
    
    let total_targets: f64 = goals.iter().map(|g| g.target_amount).sum();
    let total_current: f64 = goals.iter().map(|g| g.current_amount).sum();
    let total_remaining: f64 = total_targets - total_current;
    
    let overall_progress = if total_targets > 0.0 {
        (total_current / total_targets * 100.0).min(100.0)
    } else {
        0.0
    };
    
    let completed: usize = goals.iter().filter(|g| g.current_amount >= g.target_amount).count();
    let overdue: usize = goals.iter().filter(|g| g.days_until_deadline() < 0 && g.current_amount < g.target_amount).count();
    let on_track: usize = goals.len() - completed - overdue;
    
    let mut tag_stats: HashMap<String, (f64, f64)> = HashMap::new();
    for goal in &goals {
        for tag in &goal.tags {
            let entry = tag_stats.entry(tag.clone()).or_insert((0.0, 0.0));
            entry.0 += goal.target_amount;
            entry.1 += goal.current_amount;
        }
    }
    
    match output_format {
        OutputFormat::Json => {
            let tag_json: HashMap<String, serde_json::Value> = tag_stats.into_iter().map(|(k, v)| {
                let progress = if v.0 > 0.0 { v.1 / v.0 * 100.0 } else { 0.0 };
                (k, serde_json::json!({
                    "target": v.0,
                    "current": v.1,
                    "progress": progress
                }))
            }).collect();
            
            let stats_json = serde_json::json!({
                "total_goals": goals.len(),
                "total_target": total_targets,
                "total_current": total_current,
                "total_remaining": total_remaining,
                "overall_progress": overall_progress,
                "completed": completed,
                "overdue": overdue,
                "on_track": on_track,
                "tag_stats": tag_json
            });
            println!("{}", serde_json::to_string_pretty(&stats_json)?);
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Savings Statistics");
            
            println!("\n{} {}", "Overall Progress:".cyan().bold(), format!("{overall_progress:.1}%").green());
            println!("{}", crate::presentation::print_progress_bar(overall_progress, 50));
            
            println!("\n{} {:.2} / {:.2}", "Total Saved:".dimmed(), total_current, total_targets);
            println!("{} {:.2}", "Remaining:".dimmed(), total_remaining);
            
            println!("\n{} Goals", "Summary:".cyan().bold());
            println!("  {} {} completed", "✓".green(), completed);
            println!("  {} {} on track", "○".yellow(), on_track);
            println!("  {} {} overdue", "✗".red(), overdue);
            
            if !tag_stats.is_empty() {
                println!("\n{} By Tag", "Statistics:".cyan().bold());
                for (tag, (target, current)) in tag_stats {
                    let progress = if target > 0.0 { current / target * 100.0 } else { 0.0 };
                    println!("  {tag}: {progress:.1}% ({current:.2} / {target:.2})");
                }
            }
        }
    }
    
    Ok(())
}
