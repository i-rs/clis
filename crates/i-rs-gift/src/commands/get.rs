use crate::presentation::{print_header, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let gift = if let Some(g) = storage::get_entry(&store, &name) { g } else {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_item(&serde_json::json!({
                "error": "not_found",
                "message": format!("Gift '{}' not found", name)
            }), format));
        } 
        anyhow::bail!("Gift '{name}' not found");
    };

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct GiftDetail {
            name: String,
            gift_type: String,
            recipient: String,
            occasion: String,
            value: f64,
            date: String,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let detail = GiftDetail {
            name: gift.name.clone(),
            gift_type: gift.gift_type.to_string(),
            recipient: gift.recipient.clone(),
            occasion: gift.occasion.clone(),
            value: gift.value,
            date: gift.date.format("%Y-%m-%d").to_string(),
            tags: gift.tags.clone(),
            remark: gift.remark.clone(),
            created_at: gift.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: gift.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&detail, format));
        return Ok(());
    }

    print_header("Gift Details");
    println!("  {:12} {}", "Name:".cyan(), gift.name);
    println!("  {:12} {}", "Type:".cyan(), gift.gift_type);
    println!("  {:12} {}", "Recipient:".cyan(), gift.recipient);
    println!("  {:12} {}", "Occasion:".cyan(), gift.occasion);
    println!("  {:12} {:.2}", "Value:".cyan(), gift.value);
    println!("  {:12} {}", "Date:".cyan(), gift.date.format("%Y-%m-%d"));
    println!("  {:12} {}", "Tags:".cyan(), if gift.tags.is_empty() { "-".to_string() } else { gift.tags.join(", ") });
    println!("  {:12} {}", "Remark:".cyan(), if gift.remark.is_empty() { "-".to_string() } else { gift.remark.join(", ") });
    println!("  {:12} {}", "Created:".cyan(), gift.created_at.format("%Y-%m-%d %H:%M"));
    println!("  {:12} {}", "Updated:".cyan(), gift.updated_at.format("%Y-%m-%d %H:%M"));

    Ok(())
}
