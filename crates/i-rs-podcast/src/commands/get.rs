use crate::presentation::{print_error, output_error, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let podcast = match store.podcasts.get(&name) {
        Some(p) => p,
        None => {
            if matches!(output_format, OutputFormat::Json) {
                println!("{}", output_error(&format!("Podcast '{}' not found", name), "NOT_FOUND", output_format));
            } else {
                print_error(&format!("Podcast '{}' not found", name));
            }
            anyhow::bail!("Podcast '{}' not found", name);
        }
    };

    if matches!(output_format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct PodcastOutput<'a> {
            name: &'a str,
            author: Option<&'a str>,
            duration_secs: Option<i64>,
            current_position_secs: Option<i64>,
            status: String,
            notes: Vec<&'a str>,
            tags: Vec<&'a str>,
            remark: Vec<&'a str>,
            created_at: i64,
            updated_at: i64,
        }

        let output = PodcastOutput {
            name: &podcast.name,
            author: podcast.author.as_deref(),
            duration_secs: podcast.duration_secs,
            current_position_secs: podcast.current_position_secs,
            status: podcast.status.to_string(),
            notes: podcast.notes.iter().map(|s| s.as_str()).collect(),
            tags: podcast.tags.iter().map(|s| s.as_str()).collect(),
            remark: podcast.remark.iter().map(|s| s.as_str()).collect(),
            created_at: podcast.created_at.timestamp(),
            updated_at: podcast.updated_at.timestamp(),
        };

        println!("{}", output_item(&output, output_format));
        return Ok(());
    }

    let progress = if let (Some(current), Some(total)) = (podcast.current_position_secs, podcast.duration_secs) {
        let percent = if total > 0 {
            (current as f64 / total as f64 * 100.0) as i32
        } else {
            0
        };
        let current_str = format_duration(current);
        let total_str = format_duration(total);
        format!("{} / {} ({}%)", current_str, total_str, percent)
    } else if let Some(current) = podcast.current_position_secs {
        format!("{} / -", format_duration(current))
    } else {
        "-".to_string()
    };

    let status_icon = match podcast.status {
        crate::models::PodcastStatus::NotStarted => "○",
        crate::models::PodcastStatus::InProgress => "◐",
        crate::models::PodcastStatus::Completed => "●",
    };

    println!();
    println!("{}", "┌──────────────────────────────────────────────".dimmed());
    println!("{} {}", "│".dimmed(), podcast.name.bold().cyan());
    println!("{}", "├──────────────────────────────────────────────".dimmed());

    if let Some(ref author) = podcast.author {
        println!("{} {:12} {}", "│".dimmed(), "Author:".dimmed(), author);
    }

    println!("{} {:12} {} {}", "│".dimmed(), "Status:".dimmed(), status_icon, podcast.status);
    println!("{} {:12} {}", "│".dimmed(), "Progress:".dimmed(), progress);

    if !podcast.tags.is_empty() {
        println!(
            "{} {:12} {}",
            "│".dimmed(),
            "Tags:".dimmed(),
            podcast.tags.join(", ")
        );
    }

    if !podcast.notes.is_empty() {
        println!("{} {:12}", "│".dimmed(), "Notes:".dimmed());
        for line in &podcast.notes {
            println!("{}  {}", "│".dimmed(), line);
        }
    }

    if !podcast.remark.is_empty() {
        println!("{} {:12}", "│".dimmed(), "Remark:".dimmed());
        for line in &podcast.remark {
            println!("{}  {}", "│".dimmed(), line);
        }
    }

    println!(
        "{} {:12} {}",
        "│".dimmed(),
        "Added:".dimmed(),
        podcast.created_at.format("%Y-%m-%d %H:%M")
    );

    println!("{}", "└──────────────────────────────────────────────".dimmed());

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
