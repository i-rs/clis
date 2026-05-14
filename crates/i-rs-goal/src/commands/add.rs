use crate::presentation::{output_item, print_error, print_header, print_success, OutputFormat};
use crate::storage;
use chrono::{TimeZone, Utc};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct AddArgs {
    #[arg(help = "Goal name")]
    pub name: String,
    
    #[arg(short, long, help = "Target amount to save")]
    pub target: f64,
    
    #[arg(short, long, help = "Deadline (YYYY-MM-DD)")]
    pub deadline: String,
    
    #[arg(short, long, value_delimiter = ',', help = "Tags (comma-separated)")]
    pub tags: Vec<String>,
    
    #[arg(short, long, value_delimiter = ',', help = "Remarks (comma-separated)")]
    pub remark: Vec<String>,
    
    #[arg(short = 'm', long, value_delimiter = ',', help = "Milestones as name:amount pairs (comma-separated)")]
    pub milestones: Vec<String>,
}

pub fn add(args: AddArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    if args.target <= 0.0 {
        print_error("Target amount must be greater than 0");
        anyhow::bail!("Target amount must be greater than 0");
    }
    
    let naive = i_rs_core::parse_date(&args.deadline)?;
    let deadline = Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap());
    
    let milestone_pairs: Vec<(String, f64)> = args.milestones
        .iter()
        .filter_map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() == 2 {
                let name = parts[0].to_string();
                let amount: f64 = parts[1].parse().ok()?;
                Some((name, amount))
            } else {
                None
            }
        })
        .collect();
    
    let mut store = storage::load_store()?;
    
    if store.goals.iter().any(|g| g.name == args.name) {
        print_error(&format!("Goal '{}' already exists", args.name));
        anyhow::bail!("Goal '{}' already exists", args.name);
    }
    
    let goal = storage::add_goal(
        &mut store,
        args.name,
        args.target,
        deadline,
        args.tags,
        args.remark,
        milestone_pairs,
    )?;
    
    match output_format {
        OutputFormat::Json => {
            println!("{}", output_item(&goal, output_format));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Goal Added");
            print_success(&format!("'{}' created with target of {:.2}", goal.name, goal.target_amount));
            println!("\nDeadline: {}", goal.deadline.format("%Y-%m-%d"));
            println!("Progress: {:.1}% (0.00 / {:.2})", goal.progress_percentage(), goal.target_amount);
        }
    }
    
    Ok(())
}
