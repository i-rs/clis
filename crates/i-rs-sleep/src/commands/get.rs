use crate::models::{SleepRow, ListItem};
use crate::presentation::{format_table, print_header, OutputFormat, output_item};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_get(id: String, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let record = store.get_entry(&id)
        .ok_or_else(|| anyhow::anyhow!("Sleep record '{}' not found", id))?;

    if format == OutputFormat::Json {
        let item: ListItem = record.into();
        output_item(&item, format);
    } else {
        print_header("Sleep Record Details");
        let style = owo_colors::Style::new().bold();
        
        let row = SleepRow::from_record(record);
        println!("{}", format_table(&[row]));
        
        if !record.remark.is_empty() {
            println!("\n{}", "Remarks:".style(style));
            for (i, remark) in record.remark.iter().enumerate() {
                println!("  {}. {}", i + 1, remark);
            }
        }
    }

    Ok(())
}