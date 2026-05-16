use crate::presentation::{OutputFormat, output_error, output_item, print_header};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entity = if let Some(e) = store.get_entry(&name) {
        e
    } else {
        let msg = format!("Item '{name}' not found");
        if format.is_json() {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        }
        anyhow::bail!("{msg}");
    };

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct GetOutput {
            name: String,
            purchase_date: String,
            cycle_days: Option<i64>,
            replace_in: Option<i64>,
            status: String,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let status = if let Some(expired) = entity.is_expired() {
            if expired {
                "EXPIRED".to_string()
            } else if entity.is_soon().unwrap_or(false) {
                "SOON".to_string()
            } else {
                "OK".to_string()
            }
        } else {
            "NO_CYCLE".to_string()
        };

        let output = GetOutput {
            name: entity.name.clone(),
            purchase_date: entity.purchase_date.format("%Y-%m-%d").to_string(),
            cycle_days: entity.cycle_days,
            replace_in: entity.days_until_replace(),
            status,
            tags: entity.tags.clone(),
            remark: entity.remark.clone(),
            created_at: entity.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entity.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Item: {}", entity.name.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Name:".style(style), entity.name.cyan());
    println!(
        "{:16} {}",
        "Purchase Date:".style(style),
        entity.purchase_date.format("%Y-%m-%d").to_string().cyan()
    );

    if let Some(cycle) = entity.cycle_days {
        let days = entity.days_until_replace().unwrap_or(0);
        let status = if days < 0 {
            format!("{} (expired {} days ago)", "EXPIRED".red(), days.abs())
        } else if days <= 7 {
            format!("{} ({} days left)", "REPLACE SOON".yellow().bold(), days)
        } else {
            format!("{days} days left")
        };
        println!(
            "{:16} {} ({})",
            "Cycle:".style(style),
            format!("{cycle} days").cyan(),
            status
        );
    } else {
        println!("{:16} {}", "Cycle:".style(style), "(not set)".dimmed());
        println!(
            "  {}",
            "Use 'update' command to set replacement cycle".dimmed()
        );
    }

    if !entity.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            entity
                .tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !entity.remark.is_empty() {
        println!(
            "{:16} {}",
            "Remark:".style(style),
            entity.remark.join("; ").dimmed()
        );
    }

    println!(
        "{:16} {}",
        "Created:".style(style),
        entity
            .created_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        entity
            .updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    Ok(())
}
