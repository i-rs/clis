use crate::presentation::{print_success, format_minutes, OutputFormat, output_item};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_stop(format: OutputFormat) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    let entry = storage::stop_timer(&mut store)?;
    storage::save_store(&store)?;

    if format == OutputFormat::Json {
        let item: crate::models::ListItem = (&entry).into();
        println!("{}", output_item(&item, format));
    } else {
        print_success(&format!("Timer stopped for '{}'", entry.name.cyan()));
        println!("  {} {}", "Duration:".cyan(), format_minutes(entry.duration_minutes).green());
        println!("  {} {}", "Started:".cyan(), entry.start_time.format("%Y-%m-%d %H:%M").green());
        if let Some(end) = entry.end_time {
            println!("  {} {}", "Ended:".cyan(), end.format("%Y-%m-%d %H:%M").green());
        }
    }

    Ok(())
}
