use crate::models::Event;
use crate::presentation::{print_header, print_warning};
use crate::storage;
use anyhow::Result;
use chrono::{Datelike, Utc};
use clap::Parser;
use std::collections::HashMap;
use owo_colors::OwoColorize;

#[derive(Parser, Debug, Clone)]
pub struct StatsArgs {
    #[arg(short, long, help = "Year for statistics (default: current year)")]
    pub year: Option<i32>,
}

pub fn run(args: &StatsArgs, json: bool) -> Result<()> {
    let store = storage::load_store()?;
    let current_year = Utc::now().year();
    let target_year = args.year.unwrap_or(current_year);

    let mut events_by_year: HashMap<i32, Vec<&Event>> = HashMap::new();
    for event in store.events.values() {
        let year = event.date.year();
        events_by_year.entry(year).or_default().push(event);
    }

    if let Some(events) = events_by_year.get(&target_year) {
        let total = events.len();

        let mut by_type: HashMap<String, usize> = HashMap::new();
        for e in events {
            *by_type.entry(e.event_type.to_string()).or_insert(0) += 1;
        }

        let mut by_month: HashMap<u32, usize> = HashMap::new();
        for e in events {
            let month = e.date.month();
            *by_month.entry(month).or_insert(0) += 1;
        }

        let mut all_tags: HashMap<String, usize> = HashMap::new();
        for e in events {
            for tag in &e.tags {
                *all_tags.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        if json {
            let type_json: String = by_type
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            let month_json: String = by_month
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            let tag_json: String = all_tags
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                r#"{{"success": true, "data": {{"year": {}, "total": {}, "by_type": {{{}}}, "by_month": {{{}}}, "tags": {{{}}}}}}}"#,
                target_year, total, type_json, month_json, tag_json
            );
        } else {
            print_header(&format!("Event Statistics - {}", target_year));
            println!("\n{} {} events", "Total:".cyan().bold(), total.to_string().green());

            println!("\n{}", "By Type:".cyan().bold());
            for (etype, count) in by_type.iter() {
                let label = match etype.as_str() {
                    "meeting" => "Meeting",
                    "gathering" => "Gathering",
                    "course" => "Course",
                    _ => "Other",
                };
                println!("  {}: {}", label, count.to_string().green());
            }

            println!("\n{}", "By Month:".cyan().bold());
            let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            for month in 1..=12 {
                if let Some(count) = by_month.get(&month) {
                    println!("  {}: {}", month_names[month as usize - 1], count.to_string().green());
                }
            }

            if !all_tags.is_empty() {
                println!("\n{}", "Top Tags:".cyan().bold());
                let mut sorted_tags: Vec<_> = all_tags.iter().collect();
                sorted_tags.sort_by(|a, b| b.1.cmp(a.1));
                for (tag, count) in sorted_tags.iter().take(10) {
                    println!("  {}: {}", tag, count.to_string().green());
                }
            }
        }
    } else {
        if json {
            println!(
                r#"{{"success": true, "data": {{"year": {}, "total": 0}}}}"#,
                target_year
            );
        } else {
            print_warning(&format!("No events found for year {}", target_year));
        }
    }

    Ok(())
}
