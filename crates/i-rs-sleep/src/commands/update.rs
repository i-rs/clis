use crate::presentation::{print_error, print_success};
use crate::storage;
use chrono::{DateTime, Utc};

pub fn handle_update(id: String, bedtime: Option<String>, wake_time: Option<String>, quality: Option<i32>, tags: Option<Vec<String>>, remark: Option<Vec<String>>) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    if let Some(q) = quality {
        if q < 1 || q > 5 {
            print_error("Quality must be between 1 and 5");
            anyhow::bail!("Quality must be between 1 and 5");
        }
    }

    let bedtime_dt = bedtime.as_ref().map(|b| parse_time(b)).transpose()?;
    let wake_time_dt = wake_time.as_ref().map(|w| parse_time(w)).transpose()?;

    storage::update_sleep(&mut store, &id, bedtime_dt, wake_time_dt, quality, tags, remark)?;
    storage::save_store(&store)?;

    print_success(&format!("Sleep record '{}' updated successfully", id));

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