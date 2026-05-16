use crate::models::{Debt, Stats, TypeStats};
use crate::presentation::{OutputFormat, format_stats, output_item};
use crate::storage;
use anyhow::Result;
use clap::Parser;
use std::collections::BTreeMap;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, help = "Show statistics by debt type")]
    pub by_type: bool,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let debts: Vec<&Debt> = storage::list_entries(&store);

    if debts.is_empty() {
        if output_format == OutputFormat::Json {
            println!(
                "{}",
                output_item(
                    &serde_json::json!({
                        "total_debts": 0,
                        "total_amount": 0.0,
                        "total_paid": 0.0,
                        "total_remaining": 0.0,
                        "overdue_count": 0,
                        "by_type": {}
                    }),
                    output_format
                )
            );
        } else {
            println!("No debts found.");
        }
        return Ok(());
    }

    let total_amount: f64 = debts.iter().map(|d| d.total_amount).sum();
    let total_paid: f64 = debts.iter().map(|d| d.paid_amount()).sum();
    let total_remaining: f64 = debts.iter().map(|d| d.remaining).sum();
    let overdue_count = debts.iter().filter(|d| d.is_overdue()).count();

    let mut by_type: BTreeMap<String, TypeStats> = BTreeMap::new();

    if args.by_type {
        for debt in &debts {
            let type_name = format!("{:?}", debt.debt_type);
            let entry = by_type.entry(type_name).or_insert(TypeStats {
                count: 0,
                total: 0.0,
                paid: 0.0,
                remaining: 0.0,
            });
            entry.count += 1;
            entry.total += debt.total_amount;
            entry.paid += debt.paid_amount();
            entry.remaining += debt.remaining;
        }
    }

    let stats = Stats {
        total_debts: debts.len(),
        total_amount,
        total_paid,
        total_remaining,
        overdue_count,
        by_type,
    };

    if output_format == OutputFormat::Json {
        let by_type_json: BTreeMap<String, serde_json::Value> = stats
            .by_type
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::json!({
                        "count": v.count,
                        "total": v.total,
                        "paid": v.paid,
                        "remaining": v.remaining
                    }),
                )
            })
            .collect();

        println!(
            "{}",
            output_item(
                &serde_json::json!({
                    "total_debts": stats.total_debts,
                    "total_amount": stats.total_amount,
                    "total_paid": stats.total_paid,
                    "total_remaining": stats.total_remaining,
                    "overdue_count": stats.overdue_count,
                    "by_type": by_type_json
                }),
                output_format
            )
        );
    } else {
        println!("{}", format_stats(&stats));
    }

    Ok(())
}
