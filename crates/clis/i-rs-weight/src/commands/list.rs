use crate::models::{ListItem, WeightRecord, WeightRow};
use crate::presentation::{
    OutputFormat, format_table, output_list, print_chart, print_entry_count, print_warning,
};
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_list(
    days: Option<usize>,
    chart: bool,
    stats: bool,
    limit: Option<usize>,
    offset: Option<usize>,
    format: OutputFormat,
) -> Result<()> {
    let store = crate::storage::load_store()?;
    let mut records = crate::service::list_weights(&store, days)?;

    let total = records.len();
    // Apply pagination
    if limit.is_some() || offset.is_some() {
        let start = offset.unwrap_or(0).min(total);
        let end = limit.map(|l| (start + l).min(total)).unwrap_or(total);
        records = records[start..end].to_vec();
    }
    let shown = records.len();

    if records.is_empty() {
        if format.is_json() {
            let filter = days.map(|d| format!("last {d} days"));
            println!(
                "{}",
                output_list::<serde_json::Value>(&[], total, filter.as_deref(), format)
            );
        } else {
            print_warning("No weight records found.");
        }
        return Ok(());
    }

    if format.is_json() {
        let items: Vec<ListItem> = records.iter().map(ListItem::from).collect();
        let filter = days.map(|d| format!("last {d} days"));
        println!(
            "{}",
            output_list(&items, total, filter.as_deref(), format)
        );
        return Ok(());
    }

    let rows: Vec<WeightRow> = records.iter().map(WeightRow::from_record).collect();
    println!("\n{}", format_table(&rows));
    if shown < total {
        println!(
            "  {} {}-{} / {}",
            "Showing:".dimmed(),
            offset.unwrap_or(0) + 1,
            offset.unwrap_or(0) + shown,
            total
        );
    }
    print_entry_count(shown);

    if stats {
        let (min, max, avg, change) = calculate_stats(&records);

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
        let records_ref: Vec<&WeightRecord> = records.iter().collect();
        print_chart(&records_ref, days);
    }

    Ok(())
}

fn calculate_stats(
    records: &[WeightRecord],
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if records.is_empty() {
        return (None, None, None, None);
    }

    let weights: Vec<f64> = records.iter().map(|r| r.weight).collect();
    let min = weights.iter().copied().fold(f64::INFINITY, f64::min);
    let max = weights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let avg = weights.iter().sum::<f64>() / weights.len() as f64;

    let change = if records.len() >= 2 {
        Some(
            records
                .last()
                .expect("records.len() >= 2 checked above")
                .weight
                - records
                    .first()
                    .expect("records.len() >= 2 checked above")
                    .weight,
        )
    } else {
        None
    };

    (Some(min), Some(max), Some(avg), change)
}
