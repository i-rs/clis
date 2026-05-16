use crate::presentation::OutputFormat;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_stats(format: OutputFormat) -> Result<()> {
    let store = crate::storage::load_store()?;
    let stats = crate::service::stats_kv(&store)?;

    if format.is_json() {
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }

    let style = owo_colors::Style::new().bold();

    println!();
    println!("{}", "KV Store Statistics".bold().cyan());
    println!();
    println!(
        "{:20} {}",
        "Total Entries:".style(style),
        stats.total_entries.to_string().green()
    );
    println!(
        "{:20} {}",
        "Total Tags:".style(style),
        stats.total_tags.to_string().green()
    );
    println!(
        "{:20} {}",
        "Total Remarks:".style(style),
        stats.total_remarks.to_string().green()
    );
    println!(
        "{:20} {} bytes",
        "Total Value Size:".style(style),
        stats.total_value_bytes.to_string().yellow()
    );
    println!(
        "{:20} {:.1} bytes",
        "Avg Value Size:".style(style),
        stats.avg_value_bytes
    );
    if let Some(ref oldest) = stats.oldest_entry {
        println!("{:20} {}", "Oldest Entry:".style(style), oldest.dimmed());
    }
    if let Some(ref newest) = stats.newest_entry {
        println!("{:20} {}", "Newest Entry:".style(style), newest.dimmed());
    }
    println!();

    Ok(())
}
