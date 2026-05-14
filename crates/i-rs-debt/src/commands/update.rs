use crate::models::DebtType;
use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Debt name")]
    pub name: String,
    #[arg(short, long, help = "New debt name")]
    pub rename: Option<String>,
    #[arg(short, long, help = "New debt type")]
    pub debt_type: Option<DebtType>,
    #[arg(short, long, help = "New total amount")]
    pub amount: Option<f64>,
    #[arg(short, long, help = "New interest rate")]
    pub interest_rate: Option<f64>,
    #[arg(short, long, help = "New due date (YYYY-MM-DD)")]
    pub due_date: Option<String>,
    #[arg(short, long, help = "Add tags (comma separated)")]
    pub add_tags: Option<String>,
    #[arg(short, long, help = "Remove tags (comma separated)")]
    pub remove_tags: Option<String>,
    #[arg(short, long, help = "Add remarks")]
    pub add_remark: Vec<String>,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Some(new_name) = &args.rename
        && new_name != &args.name && store.debts.contains_key(new_name) {
            anyhow::bail!("Debt '{new_name}' already exists");
        }

    {
        if !store.debts.contains_key(&args.name) {
            anyhow::bail!("Debt '{}' not found", args.name);
        }

        if let Some(new_name) = &args.rename
            && new_name != &args.name {
                let mut old_debt = store.debts.remove(&args.name).expect("contains_key check above guarantees existence");
                old_debt.name = new_name.clone();
                store.debts.insert(new_name.clone(), old_debt);
            }
    }

    let final_name = args.rename.clone().unwrap_or_else(|| args.name.clone());

    {
        let debt = match store.debts.get_mut(&final_name) {
            Some(d) => d,
            None => {
                anyhow::bail!("Debt '{final_name}' not found");
            }
        };

        if let Some(dt) = args.debt_type {
            debt.debt_type = dt;
        }

        if let Some(amount) = args.amount {
            if amount < 0.0 {
                anyhow::bail!("Amount cannot be negative");
            }
            let diff = amount - debt.total_amount;
            debt.total_amount = amount;
            debt.remaining = (debt.remaining + diff).max(0.0);
        }

        if let Some(rate) = args.interest_rate {
            if !(0.0..=100.0).contains(&rate) {
                anyhow::bail!("Interest rate must be between 0 and 100");
            }
            debt.interest_rate = Some(rate);
        }

        if let Some(date_str) = &args.due_date {
            let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| anyhow::anyhow!("Invalid date format, use YYYY-MM-DD"))?;
            let datetime: DateTime<Utc> = naive.and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid date"))?
                .and_utc();
            debt.due_date = Some(datetime);
        }

        if let Some(tags_str) = &args.add_tags {
            let new_tags: Vec<String> = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            for tag in new_tags {
                if !debt.tags.contains(&tag) {
                    debt.tags.push(tag);
                }
            }
        }

        if let Some(tags_str) = &args.remove_tags {
            let remove_tags: Vec<String> = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            debt.tags.retain(|t| !remove_tags.contains(t));
        }

        if !args.add_remark.is_empty() {
            debt.remark.extend(args.add_remark.clone());
        }

        debt.updated_at = Utc::now();
    }

    storage::save_store(&store)?;

    if output_format == OutputFormat::Json {
        println!("{{\"success\": true, \"data\": {{\"name\": \"{final_name}\"}}}}");
    } else {
        print_success(&format!("Debt '{final_name}' updated successfully"));
    }

    Ok(())
}
