use crate::models::{HabitRow, ListItem};
use crate::presentation::{format_table, print_header, OutputFormat, output_item};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let habit = store.get_entry(&name)
        .ok_or_else(|| anyhow::anyhow!("Habit '{}' not found", name))?;

    if format == OutputFormat::Json {
        let item: ListItem = habit.into();
        output_item(&item, format);
    } else {
        print_header("Habit Details");
        let style = owo_colors::Style::new().bold();
        
        let row = HabitRow::from_habit(habit);
        println!("{}", format_table(&[row]));
        
        if !habit.remark.is_empty() {
            println!("\n{}", "Remarks:".style(style));
            for (i, remark) in habit.remark.iter().enumerate() {
                println!("  {}. {}", i + 1, remark);
            }
        }
    }

    Ok(())
}