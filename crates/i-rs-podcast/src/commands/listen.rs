use crate::models::PodcastStatus;
use crate::presentation::{print_error, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_listen(
    name: String,
    position: i64,
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

    if position < 0 {
        print_error("Position must be non-negative");
        anyhow::bail!("Position must be non-negative");
    }

    if let Some(total) = podcast.duration_secs {
        if position > total {
            print_error(&format!("Position {} exceeds total duration {}", position, total));
            anyhow::bail!("Position exceeds total duration");
        }
    }

    podcast.current_position_secs = Some(position);
    podcast.updated_at = Utc::now();

    if let Some(total) = podcast.duration_secs {
        if position >= total {
            podcast.status = PodcastStatus::Completed;
            if matches!(output_format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": true,
                    "message": format!("Podcast '{}' completed!", name)
                }));
            } else {
                print_success(&format!("✓ Podcast '{}' completed! 🎉", name.green()));
            }
        } else {
            let percent = if total > 0 {
                (position as f64 / total as f64 * 100.0) as i32
            } else {
                0
            };
            if matches!(output_format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": true,
                    "message": format!("Position updated to {} ({}%)", format_duration(position), percent)
                }));
            } else {
                print_success(&format!(
                    "✓ Position: {} ({}/{} - {}%)",
                    format_duration(position).cyan(),
                    format_duration(position),
                    format_duration(total),
                    percent.to_string().green()
                ));
            }
            if podcast.status == PodcastStatus::NotStarted {
                podcast.status = PodcastStatus::InProgress;
            }
        }
    } else {
        podcast.status = PodcastStatus::InProgress;
        if matches!(output_format, OutputFormat::Json) {
            println!("{}", serde_json::json!({
                "success": true,
                "message": format!("Position updated to {}", format_duration(position))
            }));
        } else {
            print_success(&format!("✓ Position: {}", format_duration(position).cyan()));
        }
    }

    if let Some(n) = notes {
        if podcast.notes.is_empty() {
            podcast.notes = n;
        } else {
            podcast.notes.push(String::new());
            podcast.notes.extend(n);
        }
    }

    storage::save_store(&store)?;

    Ok(())
}

fn format_duration(secs: i64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}
