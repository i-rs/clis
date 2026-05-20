use crate::models::{ListItem, Priority};
use crate::presentation::{OutputFormat, output_item, print_header};
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let todo = crate::service::get_todo(&store, &name)?;

    if format.is_json() {
        let output = ListItem::from(&todo);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

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
            println!("  {line}");
        }
    }

    println!(
        "\n{:16} {}",
        "Created:".style(style),
        todo.created_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        todo.updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    Ok(())
}
