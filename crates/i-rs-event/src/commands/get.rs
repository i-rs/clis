use crate::presentation::{print_error, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;
use i_rs_core::presentation::output::output_item;
use owo_colors::OwoColorize;

#[derive(Parser, Debug, Clone)]
pub struct GetArgs {
    #[arg(help = "Event name")]
    pub name: String,
}

pub fn run(args: &GetArgs, json: bool) -> Result<()> {
    let store = storage::load_store()?;
    let format = if json { OutputFormat::Json } else { OutputFormat::Table };

    if let Some(event) = store.events.get(&args.name) {
        if json {
            println!("{}", output_item(event, format));
        } else {
            print_header("Event Details");
            println!("{}", "Name:".cyan().bold());
            println!("  {}", event.name);
            println!("{}", "Date:".cyan().bold());
            println!("  {}", event.date.format("%Y-%m-%d %H:%M"));
            println!("{}", "Type:".cyan().bold());
            println!("  {}", event.event_type);
            println!("{}", "Location:".cyan().bold());
            println!("  {}", if event.location.is_empty() { "N/A" } else { &event.location });
            println!("{}", "Participants:".cyan().bold());
            if event.participants.is_empty() {
                println!("  N/A");
            } else {
                for p in &event.participants {
                    println!("  - {}", p);
                }
            }
            println!("{}", "Tags:".cyan().bold());
            if event.tags.is_empty() {
                println!("  N/A");
            } else {
                println!("  {}", event.tags.join(", "));
            }
            println!("{}", "Remarks:".cyan().bold());
            if event.remark.is_empty() {
                println!("  N/A");
            } else {
                for r in &event.remark {
                    println!("  - {}", r);
                }
            }
            println!("{}", "Created:".cyan().bold());
            println!("  {}", event.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("{}", "Updated:".cyan().bold());
            println!("  {}", event.updated_at.format("%Y-%m-%d %H:%M:%S"));
        }
    } else {
        print_error(&format!("Event '{}' not found", args.name));
        if json {
            println!(
                r#"{{"success": false, "error": {{"code": "NOT_FOUND", "message": "Event '{}' not found"}}}}"#,
                args.name
            );
        }
        anyhow::bail!("Event '{}' not found", args.name);
    }

    Ok(())
}
