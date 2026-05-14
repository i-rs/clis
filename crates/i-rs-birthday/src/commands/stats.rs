use crate::presentation::{print_header, print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Datelike;
use chrono::Local;
use owo_colors::OwoColorize;
use std::collections::BTreeMap;

pub fn handle_stats(format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let birthdays = storage::get_all_birthdays(&store);

    if birthdays.is_empty() {
        print_warning("No birthdays to show statistics.");
        return Ok(());
    }

    let today = Local::now().date_naive();
    let today_month = today.month();
    let today_day = today.day();

    let mut this_month_count = 0;
    let mut today_count = 0;
    let mut upcoming_7_days = 0;
    let mut upcoming_30_days = 0;
    let mut this_week_count = 0;

    let mut by_relationship: BTreeMap<String, usize> = BTreeMap::new();
    let mut with_year_count = 0;
    let mut total_age: i64 = 0;

    for birthday in &birthdays {
        let days_until = birthday.days_until_birthday();

        if birthday.is_today() {
            today_count += 1;
            this_month_count += 1;
            this_week_count += 1;
            upcoming_7_days += 1;
            upcoming_30_days += 1;
        } else {
            let (month, day) = parse_birth_date(&birthday.birth_date);
            if month == today_month && day >= today_day {
                this_month_count += 1;
            }
            if days_until <= 7 {
                upcoming_7_days += 1;
                this_week_count += 1;
            }
            if days_until <= 30 {
                upcoming_30_days += 1;
            }
        }

        *by_relationship.entry(birthday.relationship.clone()).or_insert(0) += 1;

        if birthday.year.is_some() {
            with_year_count += 1;
            if let Some(age) = birthday.age() {
                total_age += age as i64;
            }
        }
    }

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct StatsOutput {
            total: usize,
            this_month: usize,
            today: usize,
            upcoming_7_days: usize,
            upcoming_30_days: usize,
            with_known_year: usize,
            average_age: Option<f64>,
            by_relationship: BTreeMap<String, usize>,
        }

        let average_age = if with_year_count > 0 {
            Some(total_age as f64 / with_year_count as f64)
        } else {
            None
        };

        let output = StatsOutput {
            total: birthdays.len(),
            this_month: this_month_count,
            today: today_count,
            upcoming_7_days,
            upcoming_30_days,
            with_known_year: with_year_count,
            average_age,
            by_relationship,
        };

        println!("{}", serde_json::to_string_pretty(&output).expect("stats output serialization must succeed"));
        return Ok(());
    }

    print_header("Birthday Statistics");
    println!();

    println!("{} {} birthdays", "Total:".bold(), birthdays.len().to_string().cyan());
    println!("{} {} birthdays this month", "This Month:".bold(), this_month_count.to_string().cyan());
    println!("{} {} birthdays", "Today:".bold().red(), today_count.to_string().red().bold());
    println!("{} {} birthdays", "This Week:".bold().yellow(), this_week_count.to_string().yellow());
    println!("{} {} birthdays", "Next 30 Days:".bold().cyan(), upcoming_30_days.to_string().cyan());
    println!();

    if with_year_count > 0 {
        let avg_age = total_age as f64 / with_year_count as f64;
        println!("{} {}", "Known Age:".bold(), with_year_count.to_string().cyan());
        println!("{} {:.1} years", "Average Age:".bold(), avg_age);
        println!();
    }

    println!("{}:", "By Relationship".bold());
    let mut relationships: Vec<_> = by_relationship.iter().collect();
    relationships.sort_by(|a, b| b.1.cmp(a.1));
    for (relationship, count) in relationships {
        println!("  {}: {}", relationship, count.to_string().cyan());
    }

    Ok(())
}

fn parse_birth_date(date: &str) -> (u32, u32) {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 2 {
        let month: u32 = parts[0].parse().unwrap_or(1);
        let day: u32 = parts[1].parse().unwrap_or(1);
        (month, day)
    } else {
        (1, 1)
    }
}
