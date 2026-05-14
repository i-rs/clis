use crate::presentation::{output_item, print_error, print_header, print_success, OutputFormat};
use crate::storage;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct DepositArgs {
    #[arg(help = "Goal name")]
    pub name: String,
    
    #[arg(short, long, help = "Amount to deposit")]
    pub amount: f64,
}

pub fn deposit(args: DepositArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    if args.amount <= 0.0 {
        print_error("Deposit amount must be greater than 0");
        anyhow::bail!("Deposit amount must be greater than 0");
    }
    
    let mut store = storage::load_store()?;
    
    let goal = store.goals.iter().find(|g| g.name == args.name);
    
    match goal {
        Some(old_goal) => {
            let previous_amount = old_goal.current_amount;
            let previous_progress = old_goal.progress_percentage();
            
            let updated_goal = storage::deposit_to_goal(&mut store, &args.name, args.amount)?;
            
            let new_progress = updated_goal.progress_percentage();
            let newly_reached: Vec<_> = updated_goal.milestones.iter()
                .filter(|m| {
                    m.reached && 
                    m.reached_at.is_some() &&
                    m.amount > previous_amount &&
                    m.amount <= updated_goal.current_amount
                })
                .collect();
            
            match output_format {
                OutputFormat::Json => {
                    println!("{}", output_item(&updated_goal, output_format));
                }
                OutputFormat::Table | OutputFormat::Default => {
                    print_header("Deposit Successful");
                    print_success(&format!("Deposited {:.2} to '{}'", args.amount, updated_goal.name));
                    println!("\nPrevious: {:.2} ({:.1}%)", previous_amount, previous_progress);
                    println!("Current: {:.2} ({:.1}%)", updated_goal.current_amount, new_progress);
                    println!("Remaining: {:.2}", updated_goal.remaining_amount());
                    
                    if !newly_reached.is_empty() {
                        println!("\n🎉 Newly reached milestones:");
                        for m in newly_reached {
                            println!("  ✓ {} ({:.2})", m.name, m.amount);
                        }
                    }
                    
                    let reached_count = updated_goal.milestones.iter().filter(|m| m.reached).count();
                    let total_count = updated_goal.milestones.len();
                    if total_count > 0 {
                        println!("\nMilestones: {}/{} reached", reached_count, total_count);
                    }
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
