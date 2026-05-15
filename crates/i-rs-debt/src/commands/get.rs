use crate::presentation::{format_debt_detail, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Debt name")]
    pub name: String,
    #[arg(short, long, help = "Show payment history")]
    pub payments: bool,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let debt = match store.get_entry(&args.name) {
        Some(d) => d,
        None => {
            anyhow::bail!("Debt '{}' not found", args.name);
        }
    };

    if output_format == OutputFormat::Json {
        let mut data = serde_json::json!({
            "name": debt.name,
            "debt_type": format!("{:?}", debt.debt_type),
            "total_amount": debt.total_amount,
            "paid_amount": debt.paid_amount(),
            "remaining": debt.remaining,
            "interest_rate": debt.interest_rate,
            "due_date": debt.due_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            "progress": format!("{:.1}%", debt.progress_percentage()),
            "is_overdue": debt.is_overdue(),
            "tags": debt.tags,
            "remark": debt.remark,
            "created_at": debt.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": debt.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        });

        if args.payments {
            let payments: Vec<_> = debt.payments.iter().map(|p| {
                serde_json::json!({
                    "amount": p.amount,
                    "paid_at": p.paid_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    "note": p.note
                })
            }).collect();
            data["payments"] = serde_json::json!(payments);
        }

        println!("{}", output_item(&data, output_format));
    } else {
        println!("{}", format_debt_detail(debt));

        if args.payments && !debt.payments.is_empty() {
            println!("\n{}", "=== Payment History ===".cyan().bold());
            for (i, payment) in debt.payments.iter().enumerate() {
                println!(
                    "{} {} {:.2} ({})",
                    (i + 1).to_string().cyan(),
                    payment.paid_at.format("%Y-%m-%d %H:%M"),
                    payment.amount,
                    payment.note.as_deref().unwrap_or("-")
                );
            }
        }
    }

    Ok(())
}

use owo_colors::OwoColorize;
