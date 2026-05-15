use crate::models::WeightRecord;
use crate::presentation::{format_table, print_chart, print_record_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_list(days: Option<usize>, chart: bool, stats: bool, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let records: Vec<WeightRecord> = if let Some(d) = days {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(d as i64);
        store
            .records
            .values()
            .filter(|r| r.date >= cutoff)
            .cloned()
            .collect()
    } else {
        store.records.values().cloned().collect()
    };

    if records.is_empty() {
        if format.is_json() {
            let filter = days.map(|d| format!("last {d} days"));
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format));
        } else {
            print_warning("No weight records found.");
        }
        return Ok(());
    }

    let records_ref: Vec<&WeightRecord> = records.iter().collect();

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            date: String,
            weight: f64,
            remark: Vec<String>,
        }

        let items: Vec<ListItem> = records.iter().map(|r| ListItem {
            date: r.date.format("%Y-%m-%d").to_string(),
            weight: r.weight,
            remark: r.remark.clone(),
        }).collect();

        let filter = days.map(|d| format!("last {d} days"));
        println!("{}", output_list(&items, items.len(), filter.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&records_ref);
    println!("\n{table}");

    print_record_count(records_ref.len());

    if stats {
        let (min, max, avg, change) = calculate_stats(&records_ref);

        println!("\n{}", "Statistics:".bold().cyan());
        if let Some(min) = min {
            println!("  {:12} {:.1} kg", "Min:".dimmed(), min);
        }
        if let Some(max) = max {
            println!("  {:12} {:.1} kg", "Max:".dimmed(), max);
        }
        if let Some(avg) = avg {
            println!("  {:12} {:.1} kg", "Average:".dimmed(), avg);
        }
        if let Some(change) = change {
            let sign = if change >= 0.0 { "+" } else { "" };
            println!("  {:12} {}{:.1} kg", "Change:".dimmed(), sign, change);
        }
    }

    if chart {
        print_chart(&records_ref, days);
    }

    Ok(())
}

fn calculate_stats(records: &[&WeightRecord]) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if records.is_empty() {
        return (None, None, None, None);
    }

    let weights: Vec<f64> = records.iter().map(|r| r.weight).collect();
    let min = weights.iter().copied().fold(f64::INFINITY, f64::min);
    let max = weights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let avg = weights.iter().sum::<f64>() / weights.len() as f64;

    let change = if records.len() >= 2 {
        Some(records.last().expect("records.len() >= 2 checked above").weight - records.first().expect("records.len() >= 2 checked above").weight)
    } else {
        None
    };

    (Some(min), Some(max), Some(avg), change)
}