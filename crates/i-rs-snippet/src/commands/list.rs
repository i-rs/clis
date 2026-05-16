use crate::presentation::{OutputFormat, format_table, output_list, print_snippet_count};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let snippets: Vec<&crate::models::Snippet> = storage::filter_by_tag(&store, tag.as_deref());

    i_rs_core::handle_empty!(snippets, format, tag.as_deref(), "No snippets found.");

    if format.is_json() {
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

        let items: Vec<ListItem> = snippets
            .iter()
            .map(|s| ListItem {
                name: s.name.clone(),
                language: s.language.clone(),
                code: s.code.clone(),
                description: s.description.clone(),
                tags: s.tags.clone(),
                remark: s.remark.clone(),
                created_at: s.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: s.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        println!(
            "{}",
            output_list(&items, items.len(), tag.as_deref(), format)
        );
        return Ok(());
    }

    let table = format_table(&snippets);
    println!("\n{table}");

    print_snippet_count(snippets.len());

    Ok(())
}
