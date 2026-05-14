use crate::models::{HeightRecord, HeightRow};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};

pub fn format_table(records: &[&HeightRecord]) -> String {
    let rows: Vec<HeightRow> = records
        .iter()
        .map(|r| HeightRow::from_record(r))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_record_count(count: usize) {
    println!("\n{} {} records", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_height_chart(records: &[&HeightRecord], days: Option<usize>) {
    if records.is_empty() {
        print_warning("No records to display chart.");
        return;
    }

    let title = match days {
        Some(d) => format!("Height Trend (Last {} days)", d),
        None => "Height Trend (All Time)".to_string(),
    };
    println!("\n{}", title.bold().cyan());
    println!("{}", "─".repeat(40).dimmed());

    let heights: Vec<f64> = records.iter().map(|r| r.height_cm).collect();
    let min_h = heights.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_h = heights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if (max_h - min_h).abs() < 0.1 {
        let avg = heights.iter().sum::<f64>() / heights.len() as f64;
        println!(" {:.1} ────────────────── {:.1}", avg, avg);
        println!("      │");
        println!("  ────┼────");
        println!("      │");
        println!(" {:.1} ────────────────── {:.1}", avg, avg);
        println!("\n  All values around {:.1} cm", avg);
        return;
    }

    let chart_height = 8;
    let range = max_h - min_h;
    let scale = |h: f64| -> usize {
        if range == 0.0 {
            chart_height / 2
        } else {
            ((h - min_h) / range * (chart_height - 1) as f64) as usize
        }
    };

    let mut chart: Vec<Vec<String>> = (0..=chart_height)
        .map(|_| vec![' '; records.len()].into_iter().map(|c| c.to_string()).collect())
        .collect();

    for (i, &h) in heights.iter().enumerate() {
        let y = chart_height - 1 - scale(h);
        chart[y][i] = "●".cyan().to_string();
    }

    for (i, &h) in heights.iter().enumerate() {
        if i > 0 {
            let prev_y = chart_height - 1 - scale(heights[i - 1]);
            let curr_y = chart_height - 1 - scale(h);
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

    let height_labels: Vec<String> = (0..=chart_height)
        .rev()
        .map(|i| {
            let h = min_h + (range * i as f64 / chart_height as f64);
            format!("{:.1}", h)
        })
        .collect();

    for (i, row) in chart.iter().enumerate() {
        let label = format!("{:>5}", height_labels[i]);
        let line: String = row.iter().map(|c| c.as_str()).collect();
        println!("{} {}", label.dimmed(), line);
    }

    if !records.is_empty() {
        let first_date = records.first().unwrap().date.format("%m-%d").to_string();
        let last_date = records.last().unwrap().date.format("%m-%d").to_string();
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
            let change = last.height_cm - first.height_cm;
            let sign = if change >= 0.0 { "+" } else { "" };
            let direction = if change > 0.0 {
                "↑".green().to_string()
            } else if change < 0.0 {
                "↓".red().to_string()
            } else {
                "→".dimmed().to_string()
            };
            println!(
                "  {:.1} → {:.1} ({}{:.1} cm {})",
                first.height_cm,
                last.height_cm,
                sign,
                change,
                direction
            );
        }
    }
}

pub fn print_stats(records: &[&HeightRecord], store: &crate::models::HeightStore) {
    if records.is_empty() {
        return;
    }

    let heights: Vec<f64> = records.iter().map(|r| r.height_cm).collect();
    let min = heights.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = heights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg = heights.iter().sum::<f64>() / heights.len() as f64;

    let change = if records.len() >= 2 {
        Some(records.last().unwrap().height_cm - records.first().unwrap().height_cm)
    } else {
        None
    };

    println!("\n{}", "Statistics:".bold().cyan());
    println!("  {:12} {:.1} cm", "Min:".dimmed(), min);
    println!("  {:12} {:.1} cm", "Max:".dimmed(), max);
    println!("  {:12} {:.1} cm", "Average:".dimmed(), avg);

    if let Some(change) = change {
        let sign = if change >= 0.0 { "+" } else { "" };
        println!("  {:12} {}{:.1} cm", "Change:".dimmed(), sign, change);
    }

    if let Some(target) = store.get_target() {
        if let Some(last) = records.last() {
            let diff = target - last.height_cm;
            let sign = if diff >= 0.0 { "+" } else { "" };
            println!("  {:12} {:.1} cm", "Target:".dimmed(), target);
            println!("  {:12} {}{:.1} cm to target", "Gap:".dimmed(), sign, diff);
        }
    }
}
