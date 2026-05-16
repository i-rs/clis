use crate::presentation::{OutputFormat, output_item, print_header, print_success, print_warning};
use crate::storage;
use clap::Parser;
use owo_colors::OwoColorize;

#[derive(Parser, Debug)]
pub struct MilestoneArgs {
    #[arg(short = 'g', long, help = "Goal name")]
    pub goal: String,

    #[arg(short = 'n', long, help = "Milestone name")]
    pub name: Option<String>,

    #[arg(short = 'a', long, help = "Milestone amount")]
    pub amount: Option<f64>,

    #[arg(short = 'r', long, help = "Remove milestone by ID")]
    pub remove: Option<String>,

    #[arg(short = 'l', long, help = "List milestones only")]
    pub list: bool,
}

pub fn list_milestones(args: MilestoneArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let goal = store.goals.get(&args.goal);

    match goal {
        Some(goal) => match output_format {
            OutputFormat::Json => {
                println!("{}", output_item(&goal.milestones, output_format));
            }
            OutputFormat::Table | OutputFormat::Default => {
                print_header(&format!("Milestones for '{}'", goal.name));

                if goal.milestones.is_empty() {
                    print_warning("No milestones defined.");
                    return Ok(());
                }

                let milestone_rows: Vec<_> = goal
                    .milestones
                    .iter()
                    .map(crate::models::MilestoneRow::from_milestone)
                    .collect();
                let table_refs: Vec<_> = milestone_rows.iter().collect();
                println!(
                    "{}",
                    crate::presentation::format_milestones_table(&table_refs)
                );

                let reached = goal.milestones.iter().filter(|m| m.reached).count();
                println!(
                    "\n{} {} of {} milestones reached",
                    "Progress:".dimmed(),
                    reached,
                    goal.milestones.len()
                );
            }
        },
        None => {
            anyhow::bail!("Goal '{}' not found", args.goal);
        }
    }

    Ok(())
}

pub fn add_milestone(args: MilestoneArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    let name = args
        .name
        .ok_or_else(|| anyhow::anyhow!("Milestone name is required"))?;
    let amount = args
        .amount
        .ok_or_else(|| anyhow::anyhow!("Milestone amount is required"))?;

    if amount <= 0.0 {
        anyhow::bail!("Milestone amount must be greater than 0");
    }

    let mut store = storage::load_store()?;

    let updated_goal = storage::add_milestone(&mut store, &args.goal, name.clone(), amount)?;

    match output_format {
        OutputFormat::Json => {
            println!("{}", output_item(&updated_goal, output_format));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Milestone Added");
            print_success(&format!(
                "Milestone '{}' ({:.2}) added to '{}'",
                name, amount, args.goal
            ));

            let reached = updated_goal
                .milestones
                .iter()
                .any(|m| m.name == name && m.reached);
            if reached {
                println!("\n🎉 This milestone is already reached!");
            }
        }
    }

    Ok(())
}

pub fn remove_milestone(args: MilestoneArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    let milestone_id = args
        .remove
        .ok_or_else(|| anyhow::anyhow!("Milestone ID is required"))?;

    let mut store = storage::load_store()?;

    let updated_goal = storage::remove_milestone(&mut store, &args.goal, &milestone_id)?;

    match output_format {
        OutputFormat::Json => {
            println!("{}", output_item(&updated_goal, output_format));
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Milestone Removed");
            print_success(&format!(
                "Milestone '{}' removed from '{}'",
                milestone_id, args.goal
            ));
        }
    }

    Ok(())
}
