use crate::models::{WeightRecord, WeightRow};
use owo_colors::OwoColorize;
pub use i_rs_core::presentation::{print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::output_list;
pub fn format_table(records: &[&WeightRecord]) -> String {
    let rows: Vec<WeightRow> = records
        .iter()
        .map(|r| WeightRow::from_record(r))
        .collect();
    i_rs_core::render_table(&rows)
}
pub fn print_record_count(count: usize) {
    println!("\n{} {} records", "Total:".dimmed(), count.to_string().cyan());
}
pub fn print_chart(records: &[&WeightRecord], days: Option<usize>) {
    if records.is_empty() {
        print_warning("No records to display chart.");
        return;
    }
    let title = match days {
        Some(d) => format!("Weight Trend (Last {} days)", d),
        None => "Weight Trend (All Time)".to_string(),
    };
    println!("\n{}", title.bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    let weights: Vec<f64> = records.iter().map(|r| r.weight).collect();
    let min_w = weights.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_w = weights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (max_w - min_w).abs() < 0.1 {
        let avg = weights.iter().sum::<f64>() / weights.len() as f64;
        println!(" {:.1} ────────────────── {:.1}", avg, avg);
        println!("      │");
        println!("  ────┼────");
        println!("      │");
        println!(" {:.1} ────────────────── {:.1}", avg, avg);
        println!("\n  All values around {:.1} kg", avg);
        return;
    }
    let chart_height = 8;
    let range = max_w - min_w;
    let scale = |w: f64| -> usize {
        if range == 0.0 {
            chart_height / 2
        } else {
            ((w - min_w) / range * (chart_height - 1) as f64) as usize
        }
    };
    let mut chart: Vec<Vec<String>> = (0..=chart_height)
        .map(|_| vec![' '; records.len()].into_iter().map(|c| c.to_string()).collect())
        .collect();
    for (i, &w) in weights.iter().enumerate() {
        let y = chart_height - 1 - scale(w);
        chart[y][i] = "●".cyan().to_string();
    }
    for (i, &w) in weights.iter().enumerate() {
        if i > 0 {
            let prev_y = chart_height - 1 - scale(weights[i - 1]);
            let curr_y = chart_height - 1 - scale(w);
            if prev_y != curr_y {
                let (lo, hi) = if prev_y < curr_y {
                    (prev_y, curr_y)
                } else {
                    (curr_y, prev_y)
                };
                for y in (lo + 1)..=hi {
                    if chart[y][i].trim().is_empty() {
                        chart[y][i] = "│".dimmed().to_string();
                    }
                }
            } else {
                chart[prev_y][i] = "─".dimmed().to_string();
            }
        }
    }
    let weight_labels: Vec<String> = (0..=chart_height)
        .rev()
        .map(|i| {
            let w = min_w + (range * i as f64 / chart_height as f64);
            format!("{:.1}", w)
        })
        .collect();
    for (i, row) in chart.iter().enumerate() {
        let label = format!("{:>5}", weight_labels[i]);
        let line: String = row.iter().map(|c| c.as_str()).collect();
        println!("{} {}", label.dimmed(), line);
    }
    if !records.is_empty() {
        let first_date = records.first().expect("!records.is_empty() checked above").date.format("%m-%d").to_string();
        let last_date = records.last().expect("!records.is_empty() checked above").date.format("%m-%d").to_string();
        let padding = records.len().saturating_sub(first_date.len() + last_date.len() + 2);
        println!(
            "{}{}{}",
            first_date.dimmed(),
            " ".repeat(padding),
            last_date.dimmed()
        );
    }
    println!("\n  {} → {}", "Start".dimmed(), "End".dimmed());
    if let Some(first) = records.first() {
        if let Some(last) = records.last() {
            let change = last.weight - first.weight;
            let sign = if change >= 0.0 { "+" } else { "" };
            let direction = if change > 0.0 {
                "↑".red().to_string()
            } else if change < 0.0 {
                "↓".green().to_string()
            } else {
                "→".dimmed().to_string()
            };
            println!(
                "  {:.1} → {:.1} ({}{:.1} kg {})",
                first.weight,
                last.weight,
                sign,
                change,
                direction
            );
        }
    }
}