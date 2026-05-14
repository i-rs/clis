use crate::presentation::print_header;
use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;

#[derive(Parser, Debug, Clone)]
pub struct ExampleArgs {}

pub fn run(_args: &ExampleArgs, _json: bool) -> Result<()> {
    print_header("i-rs-event Examples");

    println!("\n{} {}", "# Add an event".cyan().bold(), "# Add a meeting".dimmed());
    println!("i-rs-event add \"Team Meeting\" --date 2024-03-15 --type meeting --location \"Conference Room A\" -p \"Alice,Bob\" -t work,important");

    println!("\n{} {}", "# Add a gathering".cyan().bold(), "# Add a social event".dimmed());
    println!("i-rs-event add \"Birthday Party\" --date 2024-04-20 --type gathering --location \"Home\" -p \"Charlie,Dave,Eve\" -t personal,celebration");

    println!("\n{} {}", "# Add a course".cyan().bold(), "# Add a learning event".dimmed());
    println!("i-rs-event add \"Rust Workshop\" --date 2024-05-10 --type course --location \"Online\" -t learning,tech");

    println!("\n{} {}", "# List all events".cyan().bold(), "# Show all events".dimmed());
    println!("i-rs-event list");

    println!("\n{} {}", "# List events by tag".cyan().bold(), "# Filter by tag".dimmed());
    println!("i-rs-event list --tag work");

    println!("\n{} {}", "# List events by type".cyan().bold(), "# Filter by type".dimmed());
    println!("i-rs-event list --type meeting");

    println!("\n{} {}", "# Get event details".cyan().bold(), "# Show specific event".dimmed());
    println!("i-rs-event get \"Team Meeting\"");

    println!("\n{} {}", "# Delete an event".cyan().bold(), "# Remove an event".dimmed());
    println!("i-rs-event delete \"Team Meeting\"");

    println!("\n{} {}", "# View yearly statistics".cyan().bold(), "# Show statistics".dimmed());
    println!("i-rs-event stats");
    println!("i-rs-event stats --year 2024");

    println!("\n{} {}", "# JSON output".cyan().bold(), "# Get JSON format".dimmed());
    println!("i-rs-event list --json");
    println!("i-rs-event get \"Team Meeting\" --json");

    println!("\n{} {}", "# Show examples".cyan().bold(), "# Display this help".dimmed());
    println!("i-rs-event example");

    println!("\n{} {}", "# Show skill".cyan().bold(), "# View AI skill doc".dimmed());
    println!("i-rs-event skill");

    Ok(())
}
