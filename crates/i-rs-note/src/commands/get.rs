use crate::presentation::{print_error, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String) -> Result<()> {
    let store = storage::load_store()?;

    let note = match storage::get_note(&store, &name) {
        Some(n) => n,
        None => {
            print_error(&format!("Note '{}' not found", name));
            anyhow::bail!("Note '{}' not found", name);
        }
    };

    print_header(&format!("Note: {}", note.name.green()));
    println!();

    let style = OwoStyle::new().bold();

    if let Some(ref title) = note.title {
        println!("{:16} {}", "Title:".style(style), title.cyan());
    }

    if !note.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), note.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !note.content.is_empty() {
        println!("\n{}:", "Content".bold());
        for line in &note.content {
            println!("  {}", line);
        }
    }

    println!("\n{:16} {}", "Created:".style(style), note.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), note.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
