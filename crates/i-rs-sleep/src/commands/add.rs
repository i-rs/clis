use crate::presentation::{print_error, print_success, print_header};
use crate::storage;
use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;

pub fn handle_add(bedtime_str: String, wake_time_str: String, quality: i32, tags: Vec<String>, remark: Vec<String>) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    let bedtime = parse_time(&bedtime_str)?;
    let wake_time = parse_time(&wake_time_str)?;

    if quality < 1 || quality > 5 {
        print_error("Quality must be between 1 and 5");
        anyhow::bail!("Quality must be between 1 and 5");
    }

    let record = storage::add_sleep(&mut store, bedtime, wake_time, quality, tags, remark)?;
    storage::save_store(&store)?;

    print_header("Sleep Record Created");
    println!("{} {}", "ID:".style(owo_colors::Style::new().bold()), record.id);
    println!("{} {}h", "Duration:".style(owo_colors::Style::new().bold()), format!("{:.1}", record.duration_hours()));
    println!("{} {}", "Quality:".style(owo_colors::Style::new().bold()), record.quality_label());
    print_success("Sleep record created successfully");

    Ok(())
}

fn parse_time(time_str: &str) -> anyhow::Result<DateTime<Utc>> {
    let now = Utc::now().date_naive();
    
    if let Ok(time) = chrono::NaiveTime::parse_from_str(time_str, "%H:%M") {
        let datetime = now.and_time(time).and_local_timezone(chrono::Local).unwrap().with_timezone(&Utc);
        Ok(datetime)
    } else {
        Err(anyhow::anyhow!("Invalid time format. Use HH:MM"))
    }
}