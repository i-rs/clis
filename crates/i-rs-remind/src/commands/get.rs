use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let remind = if let Some(r) = storage::get_remind(&store, &name) { r } else {
        let msg = format!("Remind '{name}' not found");
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        } 
        anyhow::bail!("{msg}");
    };

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct GetOutput {
            name: String,
            title: Option<String>,
            event_date: String,
            days_until_event: i64,
            is_done: bool,
            is_past: bool,
            tags: Vec<String>,
            content: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            name: remind.name.clone(),
            title: remind.title.clone(),
            event_date: remind.event_date.format("%Y-%m-%d %H:%M").to_string(),
            days_until_event: remind.days_until_event(),
            is_done: remind.is_done,
            is_past: remind.is_past(),
            tags: remind.tags.clone(),
            content: remind.content.clone(),
            created_at: remind.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: remind.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Remind: {}", remind.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    let days = remind.days_until_event();

    if let Some(ref title) = remind.title {
        println!("{:16} {}", "Title:".style(style), title.cyan());
    }

    println!("{:16} {}", "Date:".style(style), remind.event_date.format("%Y-%m-%d %H:%M").to_string().cyan());

    let status = if remind.is_done {
        format!("{}", "DONE".green().bold())
    } else if remind.is_past() {
        format!("{} ({} days ago)", "PAST".dimmed(), days.abs())
    } else if remind.is_today() {
        format!("{}", "TODAY!".red().bold())
    } else if days <= 7 {
        format!("{} ({} days left)", "SOON".yellow().bold(), days)
    } else {
        format!("{days} days left")
    };
    println!("{:16} {}", "Status:".style(style), status);

    if !remind.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), remind.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !remind.content.is_empty() {
        println!("\n{}:", "Content".bold());
        for line in &remind.content {
            println!("  {line}");
        }
    }

    println!("\n{:16} {}", "Created:".style(style), remind.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), remind.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}