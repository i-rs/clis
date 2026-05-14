use crate::models::ListItem;
use crate::presentation::{print_error, print_header, print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(id: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let record = storage::get_entry(&store, &id)
        .or_else(|| store.entries.values().find(|r| r.id.starts_with(&id)));

    match record {
        Some(r) => {
            if matches!(format, OutputFormat::Json) {
                let item = ListItem::from(r);
                println!("{}", serde_json::to_string(&item).unwrap_or_default());
                return Ok(());
            }

            print_header("Deploy Record");
            println!("{} {}", "ID:".cyan(), r.id);
            println!("{} {}", "Project:".cyan(), r.project);
            println!("{} {}", "Environment:".cyan(), r.environment);
            println!("{} {}", "Version:".cyan(), r.version);
            println!("{} {}", "Status:".cyan(), r.status);
            println!("{} {}", "Deployed At:".cyan(), r.deployed_at.format("%Y-%m-%d %H:%M:%S"));
            if let Some(ref rollback_from) = r.rollback_from {
                println!("{} {}", "Rollback From:".cyan(), rollback_from);
            }
            if !r.tags.is_empty() {
                println!("{} {}", "Tags:".cyan(), r.tags.join(", "));
            }
            if !r.remark.is_empty() {
                println!("{} {}", "Remark:".cyan(), r.remark.join("; "));
            }
            println!("{} {}", "Created:".cyan(), r.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("{} {}", "Updated:".cyan(), r.updated_at.format("%Y-%m-%d %H:%M:%S"));
        }
        None => {
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": false,
                    "error": { "code": "NOT_FOUND", "message": format!("Deploy record '{}' not found", id) }
                }));
            } else {
                print_error(&format!("Deploy record '{}' not found", id));
                print_warning("Use 'i-rs-deploy list' to see all records.");
            }
        }
    }

    Ok(())
}
