use crate::presentation::{OutputFormat, output_item, output_list, print_header};
use crate::service;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(id: String, date: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(date_str) = date {
        let parsed_date = parse_date(&date_str)?;
        let entries = service::list_meals(&store, Some(parsed_date))?;

        if entries.is_empty() {
            if format.is_json() {
                println!(
                    "{}",
                    output_list::<serde_json::Value>(&[], 0, Some(&date_str), format)
                );
            }
            anyhow::bail!("No meals on {date_str}");
        }

        if format.is_json() {
            let items: Vec<crate::models::ListItem> = entries
                .iter()
                .map(|e| crate::models::ListItem::from(e))
                .collect();
            println!(
                "{}",
                output_list(&items, items.len(), Some(&date_str), format)
            );
            return Ok(());
        }

        print_header(&format!(
            "Meals on {}",
            parsed_date.format("%Y-%m-%d").green()
        ));
        for entry in &entries {
            println!();
            let style = OwoStyle::new().bold();
            println!("{:16} {}", "Type:".style(style), entry.meal_type.cyan());
            println!("{:16} {}", "Food:".style(style), entry.food_items.yellow());
            if let Some(cal) = entry.calories {
                println!("{:16} {}", "Calories:".style(style), cal);
            }
            if !entry.tags.is_empty() {
                println!(
                    "{:16} {}",
                    "Tags:".style(style),
                    entry
                        .tags
                        .iter()
                        .map(|t| t.magenta().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        return Ok(());
    }

    let entry = service::get_meal(&store, &id)?;

    if format.is_json() {
        let output = crate::models::ListItem::from(&entry);
        println!("{}", output_item(&output, format));
        return Ok(());
    }

    let display_id = entry.id[..8].to_string();
    print_header(&format!("Meal: {}", display_id.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Type:".style(style), entry.meal_type.cyan());
    println!("{:16} {}", "Food:".style(style), entry.food_items.yellow());
    if let Some(cal) = entry.calories {
        println!("{:16} {}", "Calories:".style(style), cal);
    }
    if !entry.tags.is_empty() {
        println!(
            "{:16} {}",
            "Tags:".style(style),
            entry
                .tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !entry.remark.is_empty() {
        println!(
            "{:16} {}",
            "Remark:".style(style),
            entry.remark.join("; ").dimmed()
        );
    }
    println!(
        "{:16} {}",
        "Date:".style(style),
        entry.date.format("%Y-%m-%d").to_string().dimmed()
    );

    Ok(())
}
