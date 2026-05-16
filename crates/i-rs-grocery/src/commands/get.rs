use crate::models::{GroceryRow, ListItem};
use crate::presentation::{OutputFormat, format_table, output_item, print_header};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let item = store
        .get_entry(&name)
        .ok_or_else(|| anyhow::anyhow!("Item '{name}' not found"))?;

    if format == OutputFormat::Json {
        let list_item: ListItem = item.into();
        output_item(&list_item, format);
    } else {
        print_header("Grocery Item Details");
        let style = owo_colors::Style::new().bold();

        let row = GroceryRow::from_item(item);
        println!("{}", format_table(&[row]));

        if !item.remark.is_empty() {
            println!("\n{}", "Remarks:".style(style));
            for (i, remark) in item.remark.iter().enumerate() {
                println!("  {}. {}", i + 1, remark);
            }
        }
    }

    Ok(())
}
