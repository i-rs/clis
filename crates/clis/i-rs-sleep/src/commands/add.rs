use crate::models::ListItem;
use crate::presentation::{OutputFormat, output_item, print_success};
use crate::service;
use crate::storage;
use chrono::{DateTime, Utc};

pub fn handle_add(
    bedtime_str: String,
    wake_time_str: String,
    quality: i32,
    tags: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    let bedtime = parse_time(&bedtime_str)?;
    let wake_time = parse_time(&wake_time_str)?;

    if !(1..=5).contains(&quality) {
        anyhow::bail!("Quality must be between 1 and 5");
    }

    let record = service::add_sleep(&mut store, bedtime, wake_time, quality, tags, remark)?;
    storage::save_store(&store)?;

    if format.is_json() {
        let output = ListItem::from(&record);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_success("Sleep record created successfully");

    Ok(())
}

fn parse_time(time_str: &str) -> anyhow::Result<DateTime<Utc>> {
    let now = Utc::now().date_naive();

    if let Ok(time) = chrono::NaiveTime::parse_from_str(time_str, "%H:%M") {
        let datetime = now
            .and_time(time)
            .and_local_timezone(chrono::Local)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid time due to DST transition"))?
            .with_timezone(&Utc);
        Ok(datetime)
    } else {
        Err(anyhow::anyhow!("Invalid time format. Use HH:MM"))
    }
}
