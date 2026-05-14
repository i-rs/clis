use crate::models::{Debt, DebtType};
use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Debt name")]
    pub name: String,
    #[arg(long, help = "Debt type: credit_card, loan, borrowed")]
    pub debt_type: DebtType,
    #[arg(short, long, help = "Total debt amount")]
    pub amount: f64,
    #[arg(short, long, help = "Interest rate (percentage)")]
    pub interest_rate: Option<f64>,
    #[arg(short, long, help = "Due date (YYYY-MM-DD)")]
    pub due_date: Option<String>,
    #[arg(short, long, help = "Tags (comma separated)")]
    pub tags: Option<String>,
    #[arg(short, long, help = "Remarks (multiple)")]
    pub remark: Vec<String>,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.debts.contains_key(&args.name) {
        anyhow::bail!("Debt '{}' already exists", args.name);
    }

    let mut debt = Debt::new(args.name.clone(), args.debt_type, args.amount);

    if let Some(rate) = args.interest_rate {
        if rate < 0.0 || rate > 100.0 {
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

    if let Some(tags_str) = &args.tags {
        debt.tags = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    if !args.remark.is_empty() {
        debt.remark = args.remark.clone();
    }

    storage::add_debt(&mut store, debt);
    storage::save_store(&store)?;

    if output_format == OutputFormat::Json {
        println!("{{\"success\": true, \"data\": {{\"name\": \"{}\"}}}}", args.name);
    } else {
        print_success(&format!("Debt '{}' created successfully", args.name));
    }

    Ok(())
}
