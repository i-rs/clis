use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    birth_date: Option<String>,
    year: Option<i32>,
    relationship: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let birthday = match store.get_entry_mut(&name) {
        Some(b) => b,
        None => {
            anyhow::bail!("Birthday '{name}' not found");
        }
    };

    if let Some(date) = birth_date {
        if let Err(e) = validate_birth_date(&date) {
            anyhow::bail!("{e}");
        }
        birthday.birth_date = date;
    }
    if let Some(year) = year {
        birthday.year = Some(year);
    }
    if let Some(relationship) = relationship {
        birthday.relationship = relationship;
    }
    if let Some(tag) = tag {
        birthday.tags = tag;
    }
    if let Some(remark) = remark {
        birthday.remark = remark;
    }

    birthday.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Birthday '{}' updated successfully", name.green()));

    Ok(())
}

fn validate_birth_date(date: &str) -> Result<(), String> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid birth date format: {date}. Use MM-DD (e.g., 06-15)"));
    }

    let month: u32 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid month: {}. Must be 01-12", parts[0]))?;
    let day: u32 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid day: {}. Must be 01-31", parts[1]))?;

    if !(1..=12).contains(&month) {
        return Err(format!("Invalid month: {month}. Must be 01-12"));
    }

    let max_day = days_in_month(month);
    if day < 1 || day > max_day {
        return Err(format!("Invalid day: {day} for month {month}. Must be 01-{max_day}"));
    }

    Ok(())
}

const fn days_in_month(month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 29,
        _ => 31,
    }
}
