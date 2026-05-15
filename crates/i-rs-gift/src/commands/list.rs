use crate::presentation::{format_table, print_gift_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, gift_type: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let gifts: Vec<&crate::models::Gift> = if let Some(ref t) = gift_type {
        storage::filter_by_type(&store, Some(&t.to_lowercase()))
    } else if tag.is_some() {
        storage::filter_by_tag(&store, tag.as_deref())
    } else {
        storage::filter_by_tag(&store, None)
    };

    if gifts.is_empty() {
        if format.is_json() {
            let filter = tag.or(gift_type);
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format));
        } else {
            print_warning("No gifts found.");
        }
        return Ok(());
    }

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            gift_type: String,
            recipient: String,
            occasion: String,
            value: f64,
            date: String,
            tags: Vec<String>,
            remark: Vec<String>,
        }

        let items: Vec<ListItem> = gifts.iter().map(|g| ListItem {
            name: g.name.clone(),
            gift_type: g.gift_type.to_string(),
            recipient: g.recipient.clone(),
            occasion: g.occasion.clone(),
            value: g.value,
            date: g.date.format("%Y-%m-%d").to_string(),
            tags: g.tags.clone(),
            remark: g.remark.clone(),
        }).collect();

        let filter = tag.or(gift_type);
        println!("{}", output_list(&items, items.len(), filter.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&gifts);
    println!("\n{table}");

    print_gift_count(gifts.len());

    Ok(())
}
