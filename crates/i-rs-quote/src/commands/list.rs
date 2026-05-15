use crate::presentation::{format_table, print_quote_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, author: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let quotes: Vec<&crate::models::Quote> = if let Some(ref author) = author {
        storage::filter_by_author(&store, Some(author))
    } else if let Some(ref tag) = tag {
        storage::filter_by_tag(&store, Some(tag))
    } else {
        storage::get_all_quotes(&store)
    };

    if quotes.is_empty() {
        if format.is_json() {
            let filter_str = if let Some(ref a) = author {
                format!("author:{a}")
            } else if let Some(ref t) = tag {
                format!("tag:{t}")
            } else {
                "all".to_string()
            };
            println!("{}", output_list::<serde_json::Value>(&[], 0, Some(filter_str.as_str()), format));
        } else {
            print_warning("No quotes found.");
        }
        return Ok(());
    }

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            id: String,
            content: String,
            author: Option<String>,
            source: Option<String>,
            tags: Vec<String>,
            created_at: String,
        }

        let items: Vec<ListItem> = quotes.iter().map(|q| ListItem {
            id: q.id.clone(),
            content: q.content.clone(),
            author: q.author.clone(),
            source: q.source.clone(),
            tags: q.tags.clone(),
            created_at: q.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        let filter_str: Option<String> = author.as_ref()
            .map(|a| format!("author:{a}"))
            .or_else(|| tag.clone());

        println!("{}", output_list(&items, items.len(), filter_str.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&quotes);
    println!("\n{table}");

    print_quote_count(quotes.len());

    Ok(())
}
