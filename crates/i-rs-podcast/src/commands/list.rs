use crate::models::PodcastStatus;
use crate::presentation::{format_table, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    status_filter: Option<String>,
    tag: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let podcasts: Vec<_> = if let Some(ref s) = status_filter {
        let status = PodcastStatus::from(s.as_str());
        store.get_by_status(status)
    } else if let Some(ref t) = tag {
        store.filter_by_tag(t)
    } else {
        store.get_all_podcasts()
    };

    if podcasts.is_empty() {
        if matches!(output_format, OutputFormat::Json) {
            let filter = status_filter
                .clone()
                .or_else(|| tag.map(|t| format!("tag:{}", t)));
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), output_format));
        } else {
            print_warning("No podcasts found.");
        }
        return Ok(());
    }

    if matches!(output_format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            author: Option<String>,
            duration_secs: Option<i64>,
            current_position_secs: Option<i64>,
            status: String,
            tags: Vec<String>,
        }

        let items: Vec<ListItem> = podcasts
            .iter()
            .map(|p| ListItem {
                name: p.name.clone(),
                author: p.author.clone(),
                duration_secs: p.duration_secs,
                current_position_secs: p.current_position_secs,
                status: p.status.to_string(),
                tags: p.tags.clone(),
            })
            .collect();

        let filter = status_filter
            .clone()
            .or_else(|| tag.map(|t| format!("tag:{}", t)));

        println!("{}", output_list(&items, items.len(), filter.as_deref(), output_format));
        return Ok(());
    }

    let table = format_table(&podcasts);
    println!("\n{}", table);

    let total = podcasts.len();
    let not_started = podcasts
        .iter()
        .filter(|p| p.status == PodcastStatus::NotStarted)
        .count();
    let in_progress = podcasts
        .iter()
        .filter(|p| p.status == PodcastStatus::InProgress)
        .count();
    let completed = podcasts
        .iter()
        .filter(|p| p.status == PodcastStatus::Completed)
        .count();

    println!(
        "\nTotal: {} podcasts (○ {} ◐ {} ● {})",
        total, not_started, in_progress, completed
    );

    Ok(())
}
