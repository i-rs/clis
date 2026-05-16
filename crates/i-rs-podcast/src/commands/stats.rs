use crate::presentation::{OutputFormat, print_stats};
use crate::storage;
use anyhow::Result;

pub fn handle_stats(output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let stats = store.podcast_stats();

    if matches!(output_format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct StatsOutput {
            total: usize,
            not_started: usize,
            in_progress: usize,
            completed: usize,
            total_duration_secs: i64,
            total_listened_secs: i64,
            total_progress_percent: f64,
        }

        let output = StatsOutput {
            total: stats.total,
            not_started: stats.not_started,
            in_progress: stats.in_progress,
            completed: stats.completed,
            total_duration_secs: stats.total_duration_secs,
            total_listened_secs: stats.total_listened_secs,
            total_progress_percent: stats.total_progress_percent(),
        };

        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "data": output
            })
        );
    } else {
        print_stats(&stats);
    }

    Ok(())
}
