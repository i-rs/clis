use crate::presentation::{output_item, print_header, OutputFormat};
use crate::storage;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct GetArgs {
    #[arg(help = "Goal name")]
    pub name: String,
}

pub fn get(args: GetArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;
    
    let goal = store.goals.iter().find(|g| g.name == args.name);
    
    match goal {
        Some(goal) => {
            match output_format {
                OutputFormat::Json => {
                    println!("{}", output_item(goal, output_format));
                }
                OutputFormat::Table | OutputFormat::Default => {
                    print_header(&format!("Goal: {}", goal.name));
                    println!("\nTarget Amount: {:.2}", goal.target_amount);
                    println!("Current Amount: {:.2}", goal.current_amount);
                    println!("Progress: {:.1}%", goal.progress_percentage());
                    println!("Remaining: {:.2}", goal.remaining_amount());
                    println!("Deadline: {}", goal.deadline.format("%Y-%m-%d"));
                    println!("Days Left: {}", goal.days_until_deadline());
                    
                    if !goal.tags.is_empty() {
                        println!("\nTags: {}", goal.tags.join(", "));
                    }
                    
                    if !goal.remark.is_empty() {
                        println!("\nRemarks:");
                        for r in &goal.remark {
                            println!("  - {}", r);
                        }
                    }
                    
                    if !goal.milestones.is_empty() {
                        println!("\nMilestones:");
                        let milestone_rows: Vec<_> = goal.milestones.iter().map(|m| {
                            crate::models::MilestoneRow::from_milestone(m)
                        }).collect();
                        let table_refs: Vec<_> = milestone_rows.iter().collect();
                        println!("{}", crate::presentation::format_milestones_table(&table_refs));
                    }
                    
                    println!("\nCreated: {}", goal.created_at.format("%Y-%m-%d %H:%M"));
                    println!("Updated: {}", goal.updated_at.format("%Y-%m-%d %H:%M"));
                }
            }
        }
        None => {
            anyhow::bail!("Goal '{}' not found", args.name);
        }
    }
    
    Ok(())
}
