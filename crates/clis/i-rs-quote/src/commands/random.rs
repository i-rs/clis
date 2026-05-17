use crate::presentation::print_warning;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_random() -> Result<()> {
    let store = storage::load_store()?;
    let quotes = storage::get_all_quotes(&store);

    if quotes.is_empty() {
        print_warning("No quotes found.");
        return Ok(());
    }

    use std::collections::hash_map::RandomState;
    use std::hash::BuildHasher;

    let random_index = {
        let rs = RandomState::new();

        (rs.hash_one(std::time::SystemTime::now()) as usize) % quotes.len()
    };

    let quote = quotes[random_index];

    let style = OwoStyle::new().bold();

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    println!("  {}", quote.content.cyan());
    println!();

    if let Some(ref author) = quote.author {
        println!("  — {}", author.green());
    }

    if let Some(ref source) = quote.source {
        println!("  ({})", source.yellow());
    }

    if !quote.tags.is_empty() {
        println!("  ");
        println!(
            "  {}",
            quote
                .tags
                .iter()
                .map(|t| format!("[{}]", t.magenta()))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    if !quote.remark.is_empty() {
        println!("{}", "Remarks:".style(style));
        for line in &quote.remark {
            println!("  {}", line.dimmed());
        }
        println!();
    }

    println!("  {}", format!("[{}]", quote.id).dimmed());
    println!();

    Ok(())
}
