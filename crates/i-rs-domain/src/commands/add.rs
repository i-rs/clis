use crate::models::Domain;
use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    expiry_date: String,
    registrar: Option<String>,
    password: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let store = storage::load_store()?;

    if store.domains.contains_key(&name) {
        print_error(&format!("Domain '{}' already exists", name));
        anyhow::bail!("Domain '{}' already exists", name);
    }

    let expiry = parse_date(&expiry_date)?;

    if let Some(ref pwd) = password {
        storage::store_password(&name, pwd)?;
    }

    let now = Utc::now();
    let domain = Domain {
        name: name.clone(),
        expiry_date: expiry,
        registrar,
        password: None,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    let mut store = storage::load_store()?;
    storage::add_domain(&mut store, domain);
    storage::save_store(&store)?;

    print_success(&format!("✓ Domain '{}' added successfully", name.green()));

    if password.is_some() {
        println!("  {}", "Password stored securely in keychain".dimmed());
    }

    Ok(())
}

fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    let formats = [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%d-%m-%Y",
        "%d/%m/%Y",
    ];

    for format in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }

    if let Ok(naive) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap()));
    }

    if let Ok(naive) = chrono::NaiveDate::parse_from_str(date_str, "%d-%m-%Y") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap()));
    }

    if let Ok(naive) = chrono::NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
        return Ok(Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0).unwrap()));
    }

    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
