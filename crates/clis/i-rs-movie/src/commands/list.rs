use crate::presentation::{OutputFormat, format_table, output_list, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    watched: Option<bool>,
    tag: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let movies: Vec<_> = if let Some(w) = watched {
        if w {
            store.get_watched_movies()
        } else {
            store.get_unwatched_movies()
        }
    } else if let Some(ref t) = tag {
        store.filter_by_tag(t)
    } else {
        store.get_all_movies()
    };

    if movies.is_empty() {
        if matches!(output_format, OutputFormat::Json) {
            let filter = if let Some(w) = watched {
                Some(if w { "watched" } else { "unwatched" }.to_string())
            } else {
                tag.as_ref().map(|t| format!("tag:{t}"))
            };
            println!(
                "{}",
                output_list::<serde_json::Value>(&[], 0, filter.as_deref(), output_format)
            );
        } else {
            print_warning("No movies found.");
        }
        return Ok(());
    }

    if matches!(output_format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem<'a> {
            name: &'a str,
            year: Option<i32>,
            director: Option<&'a str>,
            watched: bool,
            rating: Option<f32>,
            tags: Vec<&'a str>,
        }

        let items: Vec<ListItem> = movies
            .iter()
            .map(|m| ListItem {
                name: &m.name,
                year: m.year,
                director: m.director.as_deref(),
                watched: m.watched,
                rating: m.rating,
                tags: m.tags.iter().map(std::string::String::as_str).collect(),
            })
            .collect();

        let filter = if let Some(w) = watched {
            Some(if w { "watched" } else { "unwatched" }.to_string())
        } else {
            tag.as_ref().map(|t| format!("tag:{t}"))
        };

        println!(
            "{}",
            output_list(&items, items.len(), filter.as_deref(), output_format)
        );
        return Ok(());
    }

    let table = format_table(&movies);
    println!("\n{table}");

    let total = movies.len();
    let watched_count = movies.iter().filter(|m| m.watched).count();
    println!(
        "\nTotal: {} movies ({} watched, {} unwatched)",
        total,
        watched_count,
        total - watched_count
    );

    Ok(())
}
