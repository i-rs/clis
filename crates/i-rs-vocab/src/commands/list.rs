use crate::models::{VocabStatus, VocabWord};
use crate::presentation::{format_table, output_list, print_stats, print_word_count, print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    status_filter: Option<String>,
    tag_filter: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let words: Vec<&VocabWord> = if let Some(ref status_str) = status_filter {
        if let Some(status) = VocabStatus::from_str(status_str) { store.filter_by_status(status) } else {
            print_warning(&format!("Invalid status '{status_str}'. Showing all words."));
            store.get_all_words()
        }
    } else if let Some(ref tag) = tag_filter {
        store.filter_by_tag(tag)
    } else {
        store.get_all_words()
    };

    if words.is_empty() {
        if format.is_json() {
            let filter = status_filter.or(tag_filter);
            println!("{}", output_list::<serde_json::Value>(&[], 0, filter.as_deref(), format));
        } else {
            print_warning("No vocabulary words found.");
        }
        return Ok(());
    }

    if format.is_json() {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            word: String,
            definition: String,
            status: String,
            status_label: String,
            review_count: u32,
            tags: Vec<String>,
        }

        let items: Vec<ListItem> = words.iter().map(|w| ListItem {
            word: w.word.clone(),
            definition: w.definition.clone(),
            status: w.status.to_string(),
            status_label: w.status.label().to_string(),
            review_count: w.review_count,
            tags: w.tags.clone(),
        }).collect();

        let filter = status_filter.or(tag_filter);
        println!("{}", output_list(&items, items.len(), filter.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&words);
    println!("\n{table}");

    print_word_count(words.len());

    if status_filter.is_none() && tag_filter.is_none() {
        let stats = store.get_stats();
        print_stats(&stats);
    }

    Ok(())
}
