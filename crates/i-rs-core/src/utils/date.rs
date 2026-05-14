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
        "Invalid date format: {date_str}. Use YYYY-MM-DD"
    ))
}

/// Parse a date string that may optionally include time. Returns `DateTime`<Utc>.
///
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
        "Invalid date format: {date_str}. Use YYYY-MM-DD or YYYY-MM-DD HH:MM"
    ))
}

#[must_use] 
pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use chrono::Timelike;

    #[test]
    fn test_parse_date_standard() {
        assert!(parse_date("2024-01-15").is_ok());
        assert!(parse_date("2024/01/15").is_ok());
        assert!(parse_date("15-01-2024").is_ok());
        assert!(parse_date("15/01/2024").is_ok());
    }

    #[test]
    fn test_parse_date_invalid() {
        assert!(parse_date("not-a-date").is_err());
        assert!(parse_date("").is_err());
        assert!(parse_date("2024-13-01").is_err());
    }

    #[test]
    fn test_parse_datetime_with_time() {
        let dt = parse_datetime("2024-01-15 14:30:00").expect("valid datetime");
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_datetime_date_only() {
        let dt = parse_datetime("2024-01-15").expect("valid date");
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_parse_datetime_slash_formats() {
        assert!(parse_datetime("2024/01/15 14:30:00").is_ok());
        assert!(parse_datetime("2024/01/15 14:30").is_ok());
        assert!(parse_datetime("2024/01/15").is_ok());
    }

    #[test]
    fn test_parse_datetime_invalid() {
        assert!(parse_datetime("not-a-date").is_err());
        assert!(parse_datetime("").is_err());
    }

    #[test]
    fn test_format_date() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).expect("2024-01-15 is valid");
        assert_eq!(format_date(date), "2024-01-15");
    }
}