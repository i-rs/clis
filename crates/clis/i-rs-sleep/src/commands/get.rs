use crate::models::{ListItem, SleepRow};
use crate::presentation::{OutputFormat, format_table, output_item, print_header};
use crate::service;
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_get(id: String, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let record = service::get_sleep(&store, &id)?;

    if format.is_json() {
        let item = ListItem::from(&record);
        println!("{}", output_item(&item, format));
        return Ok(());
    }

    print_header("Sleep Record Details");
    let style = owo_colors::Style::new().bold();

    let row = SleepRow::from_record(&record);
    println!("{}", format_table(&[row]));

    if !record.remark.is_empty() {
        println!("\n{}", "Remarks:".style(style));
        for (i, remark) in record.remark.iter().enumerate() {
            println!("  {}. {}", i + 1, remark);
        }
    }

    Ok(())
}
