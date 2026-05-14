use crate::models::Debt;
use crate::presentation::{format_debt_table, output_list, print_debt_count, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;
use serde_json::Value;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, help = "Filter by tag")]
    pub tag: Option<String>,
    #[arg(short, long, help = "Show overdue only")]
    pub overdue: bool,
    #[arg(short, long, help = "Show paid off only")]
    pub paid_off: bool,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let mut debts: Vec<&Debt> = storage::list_debts(&store);

    if let Some(tag) = &args.tag {
        debts.retain(|d| d.tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()));
    }

    if args.overdue {
        debts.retain(|d| d.is_overdue());
    }

    if args.paid_off {
        debts.retain(|d| d.remaining == 0.0);
    }

    debts.sort_by(|a, b| a.name.cmp(&b.name));

    if debts.is_empty() {
        if output_format == OutputFormat::Json {
            println!("{}", output_list::<Value>(&[], 0, args.tag.as_deref(), output_format));
        } else {
            println!("No debts found.");
        }
        return Ok(());
    }

    if output_format == OutputFormat::Json {
        let data: Vec<_> = debts.iter().map(|d| {
            serde_json::json!({
                "name": d.name,
                "debt_type": format!("{:?}", d.debt_type),
                "total_amount": d.total_amount,
                "paid_amount": d.paid_amount(),
                "remaining": d.remaining,
                "progress": format!("{:.1}%", d.progress_percentage()),
                "is_overdue": d.is_overdue(),
                "tags": d.tags
            })
        }).collect();
        println!("{}", output_list(&data, debts.len(), args.tag.as_deref(), output_format));
    } else {
        let table = format_debt_table(&debts);
        if !table.is_empty() {
            println!("{}", table);
        }
        print_debt_count(debts.len());
    }

    Ok(())
}
