use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, show_value: bool, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let entry = match storage::get_entry(&store, &name) {
        Some(e) => e,
        None => {
            let msg = format!("Key '{}' not found", name);
            if matches!(format, OutputFormat::Json) {
                println!("{}", output_error(&msg, "NOT_FOUND", format));
            } else {
            }
            anyhow::bail!("{}", msg);
        }
    };

    let key_value = storage::get_key(&name)?;

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct GetOutput {
            name: String,
            key_type: String,
            value: Option<String>,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            name: entry.name.clone(),
            key_type: entry.key_type.clone(),
            value: if show_value { key_value } else { None },
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Key: {}", entry.name.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!("{:16} {}", "Name:".style(style), entry.name.cyan());
    println!("{:16} {}", "Type:".style(style), entry.key_type.yellow());

    match key_value {
        Some(value) => {
            if show_value {
                println!("{:16} {}", "Value:".style(style), value.red());
            } else {
                println!("{:16} {}", "Value:".style(style), "(use --show-value to reveal)".dimmed().to_string());
            }
        }
        None => {
            println!("{:16} {}", "Value:".style(style), "(not set)".dimmed().to_string());
        }
    }

    if !entry.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), entry.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !entry.remark.is_empty() {
        println!("{:16} {}", "Remark:".style(style), entry.remark.join("; ").dimmed());
    }

    println!("{:16} {}", "Created:".style(style), entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
