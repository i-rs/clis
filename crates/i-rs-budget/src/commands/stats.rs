use crate::models::Budget;
use crate::presentation::{format_budget_stats_table, output_list, print_total_spent, print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use owo_colors::OwoColorize;

pub fn handle_stats(
    category: Option<String>,
    period: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let now = Utc::now().date_naive();
    let (start, end) = get_period_dates(&now, period.as_deref())?;

    let budgets: Vec<&Budget> = if let Some(cat) = &category {
        store.budgets.get(cat).into_iter().collect()
    } else {
        store.budgets.values().collect()
    };

    if budgets.is_empty() {
        if matches!(format, OutputFormat::Json) {
            let filter = category.as_deref().or(period.as_deref());
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter, format));
        } else {
            print_warning("No budgets to show stats for.");
        }
        return Ok(());
    }

    let mut spent_map = std::collections::HashMap::new();
    for budget in &budgets {
        let spent = store.get_expenses_in_period(start, end)
            .into_iter()
            .filter(|e| e.category == budget.category)
            .map(|e| e.amount)
            .sum();
        spent_map.insert(budget.category.clone(), spent);
    }

    let total_budget: f64 = budgets.iter().map(|b| b.amount).sum();
    let total_spent: f64 = spent_map.values().sum();

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct StatsItem {
            category: String,
            budget: f64,
            spent: f64,
            remaining: f64,
            percentage: f64,
        }

        let items: Vec<StatsItem> = budgets.iter().map(|b| {
            let spent = spent_map.get(&b.category).copied().unwrap_or(0.0);
            let remaining = b.amount - spent;
            let percentage = if b.amount > 0.0 {
                (spent / b.amount * 100.0).min(100.0)
            } else {
                0.0
            };
            StatsItem {
                category: b.category.clone(),
                budget: b.amount,
                spent,
                remaining,
                percentage,
            }
        }).collect();

        let filter = category.as_deref().or(period.as_deref());
        let meta = serde_json::json!({
            "period": format!("{} to {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d")),
            "total_budget": total_budget,
            "total_spent": total_spent,
            "total_remaining": total_budget - total_spent,
        });
        println!("{}", output_list_with_meta(&items, items.len(), filter, format, meta));
    } else {
        println!("\n{}", format!("Budget Stats ({} to {})", start.format("%Y-%m-%d"), end.format("%Y-%m-%d")).cyan().bold());
        println!("{}\n", "─".repeat(50).dimmed());

        let table = format_budget_stats_table(&budgets, &spent_map);
        println!("{}", table);

        println!("\n{}", "Summary:".bold().cyan());
        println!("  {} {:.2}", "Total Budget:".dimmed(), total_budget);
        print_total_spent(total_spent);
        println!("  {} {:.2}", "Remaining:".dimmed(), total_budget - total_spent);

        if total_budget > 0.0 {
            let overall_percentage = (total_spent / total_budget * 100.0).min(100.0);
            println!("  {} {:.1}%", "Overall:".dimmed(), overall_percentage);
        }
    }

    Ok(())
}

fn get_period_dates(now: &NaiveDate, period: Option<&str>) -> Result<(NaiveDate, NaiveDate)> {
    match period {
        Some("daily" | "d") => {
            Ok((*now, *now))
        }
        Some("weekly" | "w") => {
            let days_from_monday = now.weekday().num_days_from_monday();
            let start = *now - chrono::Duration::days(days_from_monday as i64);
            let end = start + chrono::Duration::days(6);
            Ok((start, end))
        }
        Some("yearly" | "y") => {
            let start = NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap();
            let end = NaiveDate::from_ymd_opt(now.year(), 12, 31).unwrap();
            Ok((start, end))
        }
        Some("monthly" | "m") | None => {
            let start = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
            let end = if now.month() == 12 {
                NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap() - chrono::Duration::days(1)
            } else {
                NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap() - chrono::Duration::days(1)
            };
            Ok((start, end))
        }
        Some(p) => {
            anyhow::bail!("Invalid period '{}'. Use: daily, weekly, monthly, yearly", p);
        }
    }
}

fn output_list_with_meta<T: serde::Serialize>(
    data: &[T],
    count: usize,
    filter: Option<&str>,
    _format: OutputFormat,
    meta: serde_json::Value,
) -> String {
    serde_json::json!({
        "success": true,
        "data": data,
        "meta": {
            "count": count,
            "filter": filter,
            "extra": meta,
        }
    }).to_string()
}
