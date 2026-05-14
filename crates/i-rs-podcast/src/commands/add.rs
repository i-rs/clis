use crate::models::{Podcast, PodcastStatus};
use crate::presentation::{print_error, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    author: Option<String>,
    duration: Option<i64>,
    tag: Vec<String>,
    remark: Vec<String>,
    notes: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    if let Err(e) = validate_name(&name) {
        print_error(&e.message);
        anyhow::bail!("{}", e.message);
    }

    let mut store = storage::load_store()?;

    if store.podcasts.contains_key(&name) {
        print_error(&format!("Podcast '{}' already exists. Use update command instead.", name));
        anyhow::bail!("Podcast '{}' already exists", name);
    }

    let now = Utc::now();
    let podcast = Podcast {
        name: name.clone(),
        author,
        duration_secs: duration,
        current_position_secs: Some(0),
        status: PodcastStatus::NotStarted,
        notes,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_podcast(podcast);
    storage::save_store(&store)?;

    if matches!(output_format, OutputFormat::Json) {
        println!("{}", serde_json::json!({
            "success": true,
            "message": format!("Podcast '{}' added successfully", name)
        }));
    } else {
        print_success(&format!("✓ Podcast added: {}", name.green()));
    }

    Ok(())
}
