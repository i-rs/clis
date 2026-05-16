use crate::presentation::{OutputFormat, print_stats};
use crate::storage;
use anyhow::Result;

pub fn handle_stats(output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let stats = store.movie_stats();

    if matches!(output_format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct StatsOutput {
            total: usize,
            watched: usize,
            unwatched: usize,
            avg_rating: Option<f32>,
        }

        let output = StatsOutput {
            total: stats.total,
            watched: stats.watched,
            unwatched: stats.unwatched,
            avg_rating: stats.avg_rating,
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
