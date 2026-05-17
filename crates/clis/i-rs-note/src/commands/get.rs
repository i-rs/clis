use crate::presentation::{OutputFormat, output_error, output_item, print_header};
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let note = match crate::service::get_note(&store, &name) {
        Ok(n) => n,
        Err(e) => {
            if format.is_json() {
                println!("{}", output_error(&e.to_string(), "NOT_FOUND", format));
            }
            return Err(e);
        }
    };

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct GetOutput {
            name: String,
            title: Option<String>,
            tags: Vec<String>,
            content: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            name: note.name.clone(),
            title: note.title.clone(),
            tags: note.tags.clone(),
            content: note.content.clone(),
            created_at: note.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: note.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Note: {}", note.name.green()));
    println!();

    let style = OwoStyle::new().bold();

    if let Some(ref title) = note.title {
        println!("{:16} {}", "Title:".style(style), title.cyan());
    }

    if !note.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            note.tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !note.content.is_empty() {
        println!("\n{}:", "Content".bold());
        for line in &note.content {
            println!("  {line}");
        }
    }

    println!(
        "\n{:16} {}",
        "Created:".style(style),
        note.created_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        note.updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    Ok(())
}
