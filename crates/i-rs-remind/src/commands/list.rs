use crate::presentation::{format_table, print_remind_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let reminds: Vec<&crate::models::Remind> = storage::filter_by_tag(&store, tag.as_deref());

    if reminds.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No reminds found.");
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            title: Option<String>,
            event_date: String,
            days_until_event: i64,
            is_done: bool,
            is_past: bool,
            tags: Vec<String>,
            content: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<ListItem> = reminds.iter().map(|r| ListItem {
            name: r.name.clone(),
            title: r.title.clone(),
            event_date: r.event_date.format("%Y-%m-%d %H:%M").to_string(),
            days_until_event: r.days_until_event(),
            is_done: r.is_done,
            is_past: r.is_past(),
            tags: r.tags.clone(),
            content: r.content.clone(),
            created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: r.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&reminds);
    println!("\n{}", table);

    print_remind_count(reminds.len());

    Ok(())
}