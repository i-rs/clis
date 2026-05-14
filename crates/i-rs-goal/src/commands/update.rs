use crate::presentation::{output_item, print_error, print_header, print_success, OutputFormat};
use crate::storage;
use chrono::{TimeZone, Utc};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct UpdateArgs {
    #[arg(help = "Goal name")]
    pub name: String,
    
    #[arg(short, long, help = "New target amount")]
    pub target: Option<f64>,
    
    #[arg(short, long, help = "New deadline (YYYY-MM-DD)")]
    pub deadline: Option<String>,
    
    #[arg(short, long, value_delimiter = ',', help = "New tags (comma-separated)")]
    pub tags: Option<Vec<String>>,
    
    #[arg(short, long, value_delimiter = ',', help = "New remarks (comma-separated)")]
    pub remark: Option<Vec<String>>,
}

pub fn update(args: UpdateArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;
    
    let goal = store.goals.iter_mut().find(|g| g.name == args.name);
    
    match goal {
        Some(goal) => {
            if let Some(target) = args.target {
                if target <= 0.0 {
                    print_error("Target amount must be greater than 0");
                    anyhow::bail!("Target amount must be greater than 0");
                }
                goal.target_amount = target;
            }
            
            if let Some(deadline_str) = args.deadline {
                let naive = chrono::NaiveDate::parse_from_str(&deadline_str, "%Y-%m-%d")
                    .map_err(|_| anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", deadline_str))?;
                goal.deadline = Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap());
            }
            
            if let Some(tags) = args.tags {
                goal.tags = tags;
            }
            
            if let Some(remark) = args.remark {
                goal.remark = remark;
            }
            
            goal.updated_at = Utc::now();
            goal.check_milestones();
            
            let updated_goal = goal.clone();
            storage::save_store(&store)?;
            
            match output_format {
                OutputFormat::Json => {
                    println!("{}", output_item(&updated_goal, output_format));
                }
                OutputFormat::Table => {
                    print_header("Goal Updated");
                    print_success(&format!("'{}' updated successfully", updated_goal.name));
                    println!("\nProgress: {:.1}% ({:.2} / {:.2})", 
                        updated_goal.progress_percentage(),
                        updated_goal.current_amount,
                        updated_goal.target_amount);
                }
            }
        }
        None => {
            print_error(&format!("Goal '{}' not found", args.name));
            anyhow::bail!("Goal '{}' not found", args.name);
        }
    }
    
    Ok(())
}
