use crate::models::SleepStats;
use crate::presentation::{OutputFormat, print_stats};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_stats(format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let records: Vec<&crate::models::SleepRecord> = store.entries.values().collect();

    if records.is_empty() {
        println!("{}", "No sleep records found.".cyan());
        return Ok(());
    }

    let stats = SleepStats::from_records(&records);

    if format == OutputFormat::Json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "data": stats
            })
        );
    } else {
        print_stats(&stats);
    }

    Ok(())
}
