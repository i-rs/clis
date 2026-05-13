use crate::presentation::{print_error, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String) -> Result<()> {
    let store = storage::load_store()?;

    let remind = match storage::get_remind(&store, &name) {
        Some(r) => r,
        None => {
            print_error(&format!("Remind '{}' not found", name));
            anyhow::bail!("Remind '{}' not found", name);
        }
    };

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
        format!("{} days left", days)
    };
    println!("{:16} {}", "Status:".style(style), status);

    if !remind.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), remind.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !remind.content.is_empty() {
        println!("\n{}:", "Content".bold());
        for line in &remind.content {
            println!("  {}", line);
        }
    }

    println!("\n{:16} {}", "Created:".style(style), remind.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), remind.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
