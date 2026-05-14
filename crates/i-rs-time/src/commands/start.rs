use crate::presentation::{print_success, OutputFormat, output_item};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_start(name: String, tag: Vec<String>, remark: Vec<String>, format: OutputFormat) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    let entry = storage::start_timer(&mut store, name.clone(), tag, remark)?;
    storage::save_store(&store)?;

    if format == OutputFormat::Json {
        let item: crate::models::ListItem = (&entry).into();
        println!("{}", output_item(&item, format));
    } else {
        print_success(&format!("Timer started for '{}'", name.cyan()));
        println!("  {} {}", "Started at:".cyan(), entry.start_time.format("%Y-%m-%d %H:%M").green());
        println!("  {} {}", "ID:".cyan(), entry.id[..8].to_string().green());
    }

    Ok(())
}
