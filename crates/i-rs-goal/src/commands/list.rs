use crate::models::SavingsGoal;
use crate::presentation::{output_list, print_header, print_warning, OutputFormat};
use crate::storage;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct ListArgs {
    #[arg(short, long, help = "Filter by tag")]
    pub tag: Option<String>,
}

pub fn list(args: ListArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;
    
    let goals: Vec<SavingsGoal> = match &args.tag {
        Some(tag) => store.goals.iter().filter(|g| g.tags.contains(tag)).cloned().collect(),
        None => store.goals.clone(),
    };
    
    if goals.is_empty() {
        if matches!(output_format, OutputFormat::Json) {
            let filter = args.tag.clone();
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), output_format));
        } else {
            print_warning("No goals found.");
            if args.tag.is_some() {
                println!("(Filtered by tag: {})", args.tag.as_ref().expect("args.tag.is_some() checked above"));
            }
        }
        return Ok(());
    }
    
    match output_format {
        OutputFormat::Json => {
            let filter = args.tag.clone();
            println!("{}", output_list(&goals, goals.len(), filter.as_deref(), output_format));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Savings Goals");
            
            let goal_rows: Vec<_> = goals.iter().map(|g| {
                crate::models::SavingsGoalRow::from_goal(g)
            }).collect();
            
            let table_refs: Vec<_> = goal_rows.iter().collect();
            println!("{}", crate::presentation::format_goals_table(&table_refs));
            
            crate::presentation::print_goal_count(goals.len());
        }
    }
    
    Ok(())
}
