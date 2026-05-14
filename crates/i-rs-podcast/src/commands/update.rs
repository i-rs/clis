use crate::models::PodcastStatus;
use crate::presentation::{print_error, print_success, OutputFormat};
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
            print_error(&format!("Podcast '{}' not found", name));
            anyhow::bail!("Podcast '{}' not found", name);
        }
    };

    if let Some(a) = author {
        podcast.author = Some(a);
    }
    if let Some(d) = duration {
        podcast.duration_secs = Some(d);
    }
    if let Some(t) = tag {
        podcast.tags = t;
    }
    if let Some(r) = remark {
        podcast.remark = r;
    }
    if let Some(n) = notes {
        podcast.notes = n;
    }

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
        println!("{}", serde_json::json!({
            "success": true,
            "message": format!("Podcast '{}' updated successfully", name)
        }));
    } else {
        print_success(&format!("✓ Podcast '{}' updated", name));
    }

    Ok(())
}
