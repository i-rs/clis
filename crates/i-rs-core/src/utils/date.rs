use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

const DATE_FORMATS: &[&str] = &[
    "%Y-%m-%d",
    "%Y/%m/%d",
    "%d-%m-%Y",
    "%d/%m/%Y",
];

const DATETIME_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%Y/%m/%d %H:%M:%S",
    "%Y/%m/%d %H:%M",
];

pub fn parse_date(date_str: &str) -> anyhow::Result<NaiveDate> {
    for format in DATE_FORMATS {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    Err(anyhow::anyhow!(
        "Invalid date format: {}. Use YYYY-MM-DD",
        date_str
    ))
}

/// Parse a date string that may optionally include time. Returns DateTime<Utc>.
/// Supports formats: "YYYY-MM-DD[ HH:MM[:SS]]", "YYYY/MM/DD[ HH:MM[:SS]]",
/// "DD-MM-YYYY", "DD/MM/YYYY"
/// If no time is provided, defaults to 00:00:00 UTC.
pub fn parse_datetime(date_str: &str) -> anyhow::Result<DateTime<Utc>> {
    // Try datetime formats first (e.g. "2024-01-15 14:30:00")
    for format in DATETIME_FORMATS {
        if let Ok(naive) = NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }

    // Fallback: date-only formats with midnight default
    for format in DATE_FORMATS {
        if let Ok(naive) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(Utc.from_utc_datetime(
                &naive.and_hms_opt(0, 0, 0).expect("0:00:00 is always valid"),
            ));
        }
    }

    Err(anyhow::anyhow!(
        "Invalid date format: {}. Use YYYY-MM-DD or YYYY-MM-DD HH:MM",
        date_str
    ))
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}