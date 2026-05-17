use crate::models::PodcastStatus;
use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;

pub fn handle_update(
    name: String,
    author: Option<String>,
    duration: Option<i64>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
    notes: Option<Vec<String>>,
    output_format: OutputFormat,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let podcast = match store.podcasts.get_mut(&name) {
        Some(p) => p,
        None => {
            anyhow::bail!("Podcast '{name}' not found");
        }
    };

    if let Some(a) = author {
        podcast.author = Some(a);
    }
    if let Some(d) = duration {
        podcast.duration_secs = Some(d);
    }
    i_rs_core::update_field!(podcast.tags, tag);
    i_rs_core::update_field!(podcast.remark, remark);
    i_rs_core::update_field!(podcast.notes, notes);

    if let (Some(current), Some(total)) = (podcast.current_position_secs, podcast.duration_secs) {
        if current >= total && total > 0 {
            podcast.status = PodcastStatus::Completed;
        } else if current > 0 {
            podcast.status = PodcastStatus::InProgress;
        }
    }

    podcast.updated_at = Utc::now();

    storage::save_store(&store)?;

    if matches!(output_format, OutputFormat::Json) {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "message": format!("Podcast '{}' updated successfully", name)
            })
        );
    } else {
        print_success(&format!("✓ Podcast '{name}' updated"));
    }

    Ok(())
}
