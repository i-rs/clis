use crate::models::{HabitRow, ListItem};
use crate::presentation::{format_table, print_habit_count, OutputFormat, output_list};
use owo_colors::OwoColorize;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> anyhow::Result<()> {
    let store = crate::storage::load_store()?;
    let habits = crate::service::list_habits(&store, tag.clone())?;

    if format == OutputFormat::Json {
        let items: Vec<ListItem> = habits.iter().map(|h| h.into()).collect();
        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
    } else {
        if habits.is_empty() {
            println!("{}", "No habits found.".cyan());
            return Ok(());
        }

        let rows: Vec<HabitRow> = habits
            .iter()
            .map(|h| HabitRow::from_habit(h))
            .collect();

        println!("{}", format_table(&rows));
        print_habit_count(rows.len());
    }

    Ok(())
}
