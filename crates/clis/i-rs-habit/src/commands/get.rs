use crate::models::{HabitRow, ListItem};
use crate::presentation::{OutputFormat, format_table, output_item, print_header};
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> anyhow::Result<()> {
    let store = crate::storage::load_store()?;
    let habit = crate::service::get_habit(&store, &name)?;

    if format == OutputFormat::Json {
        let item: ListItem = (&habit).into();
        println!("{}", output_item(&item, format));
    } else {
        print_header("Habit Details");
        let style = owo_colors::Style::new().bold();

        let row = HabitRow::from_habit(&habit);
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
