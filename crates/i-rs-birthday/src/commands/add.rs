use crate::models::Birthday;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    birth_date: String,
    year: Option<i32>,
    relationship: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.birthdays.contains_key(&name) {
        print_error(&format!("Birthday '{}' already exists", name));
        anyhow::bail!("Birthday '{}' already exists", name);
    }

    if let Err(e) = validate_birth_date(&birth_date) {
        print_error(&e);
        anyhow::bail!("{}", e);
    }

    let now = Utc::now();
    let birthday = Birthday {
        name: name.clone(),
        birth_date,
        year,
        relationship,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_birthday(&mut store, birthday);
    storage::save_store(&store)?;

    print_success(&format!("✓ Birthday '{}' added successfully", name.green()));

    Ok(())
}

fn validate_birth_date(date: &str) -> Result<(), String> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid birth date format: {}. Use MM-DD (e.g., 06-15)", date));
    }

    let month: u32 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid month: {}. Must be 01-12", parts[0]))?;
    let day: u32 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid day: {}. Must be 01-31", parts[1]))?;

    if month < 1 || month > 12 {
        return Err(format!("Invalid month: {}. Must be 01-12", month));
    }

    let max_day = days_in_month(month);
    if day < 1 || day > max_day {
        return Err(format!("Invalid day: {} for month {}. Must be 01-{}", day, month, max_day));
    }

    Ok(())
}

fn days_in_month(month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 29,
        _ => 31,
    }
}
