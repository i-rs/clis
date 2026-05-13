use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    expiry_date: Option<String>,
    registrar: Option<String>,
    password: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let domain = match storage::get_domain_mut(&mut store, &name) {
        Some(d) => d,
        None => {
            print_error(&format!("Domain '{}' not found", name));
            anyhow::bail!("Domain '{}' not found", name);
        }
    };

    if let Some(expiry) = expiry_date {
        domain.expiry_date = parse_date(&expiry)?;
    }
    if let Some(registrar) = registrar {
        domain.registrar = Some(registrar);
    }
    if let Some(password) = password {
        storage::store_password(&name, &password)?;
        println!("{}", "Password updated and stored securely in keychain".green());
    }
    if let Some(tag) = tag {
        domain.tags = tag;
    }
    if let Some(remark) = remark {
        domain.remark = remark;
    }

    domain.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Domain '{}' updated successfully", name.green()));

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
