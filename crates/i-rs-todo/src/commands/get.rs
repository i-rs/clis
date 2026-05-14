use crate::models::Priority;
use crate::presentation::{print_error, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String) -> Result<()> {
    let store = storage::load_store()?;

    let todo = match store.get_todo(&name) {
        Some(t) => t,
        None => {
            print_error(&format!("Todo '{}' not found", name));
            anyhow::bail!("Todo '{}' not found", name);
        }
    };

    print_header(&format!("Todo: {}", todo.name.green()));
    println!();

    let style = OwoStyle::new().bold();

    if let Some(ref title) = todo.title {
        println!("{:16} {}", "Title:".style(style), title.cyan());
    }

    let priority_str = match todo.priority {
        Priority::High => format!("🔴 {}", todo.priority.label()),
        Priority::Medium => format!("🟡 {}", todo.priority.label()),
        Priority::Low => format!("🟢 {}", todo.priority.label()),
    };
    println!("{:16} {}", "Priority:".style(style), priority_str);

    let status = if todo.is_done {
        format!("✓ {}", "Done".green())
    } else {
        "○ Pending".to_string()
    };
    println!("{:16} {}", "Status:".style(style), status);

    if !todo.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            todo.tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !todo.content.is_empty() {
        println!("\n{}:", "Content".bold());
        for line in &todo.content {
            println!("  {}", line);
        }
    }

    println!(
        "\n{:16} {}",
        "Created:".style(style),
        todo.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        todo.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
    );

    Ok(())
}
