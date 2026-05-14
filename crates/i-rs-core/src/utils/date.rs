use chrono::NaiveDate;

pub fn parse_date(date_str: &str) -> anyhow::Result<NaiveDate> {
    let formats = [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%d-%m-%Y",
        "%d/%m/%Y",
    ];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    Err(anyhow::anyhow!(
        "Invalid date format: {}. Use YYYY-MM-DD",
        date_str
    ))
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}