use crate::models::{Article, ReadStatus};
use crate::presentation::{format_table, print_article_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, status: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let status_filter = status.as_ref().map(|s| ReadStatus::from(s.as_str()));

    let articles: Vec<&Article> = storage::filter_by_tag_and_status(&store, tag.as_deref(), status_filter);

    if articles.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No articles found.");
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            title: String,
            url: String,
            source: String,
            status: String,
            tags: Vec<String>,
            created_at: String,
        }

        let items: Vec<ListItem> = articles.iter().map(|a| ListItem {
            name: a.name.clone(),
            title: a.title.clone(),
            url: a.url.clone(),
            source: a.source.clone(),
            status: a.status.to_string(),
            tags: a.tags.clone(),
            created_at: a.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&articles);
    println!("\n{table}");

    print_article_count(articles.len());

    Ok(())
}
