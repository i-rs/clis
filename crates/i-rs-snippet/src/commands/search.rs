use crate::presentation::{format_table, print_snippet_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_search(query: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let snippets: Vec<&crate::models::Snippet> = storage::search_snippets(&store, &query);

    if snippets.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, Some(&query), format));
        } else {
            print_warning(&format!("No snippets found matching '{}'", query));
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            language: String,
            code: Vec<String>,
            description: Vec<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<ListItem> = snippets.iter().map(|s| ListItem {
            name: s.name.clone(),
            language: s.language.clone(),
            code: s.code.clone(),
            description: s.description.clone(),
            tags: s.tags.clone(),
            remark: s.remark.clone(),
            created_at: s.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: s.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), Some(&query), format));
        return Ok(());
    }

    let table = format_table(&snippets);
    println!("\n{}", table);

    print_snippet_count(snippets.len());

    Ok(())
}
