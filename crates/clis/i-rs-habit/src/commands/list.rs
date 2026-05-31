use crate::models::{HabitRow, ListItem};
use crate::presentation::{OutputFormat, format_table, output_list, print_habit_count};
use owo_colors::OwoColorize;

pub fn handle_list(
    tag: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let store = crate::storage::load_store()?;
    let mut habits = crate::service::list_habits(&store, tag.clone())?;
    let total = habits.len();

    if limit.is_some() || offset.is_some() {
        let start = offset.unwrap_or(0).min(total);
        let end = limit.map(|l| (start + l).min(total)).unwrap_or(total);
        habits = habits[start..end].to_vec();
    }
    let shown = habits.len();

    if format == OutputFormat::Json {
        let items: Vec<ListItem> = habits.iter().map(std::convert::Into::into).collect();
        println!(
            "{}",
            output_list(&items, total, tag.as_deref(), format)
        );
    } else {
        if habits.is_empty() {
            println!("{}", "No habits found.".cyan());
            return Ok(());
        }
        let rows: Vec<HabitRow> = habits.iter().map(HabitRow::from_habit).collect();
        println!("{}", format_table(&rows));
        if shown < total {
            println!(
                "  {} {}-{} / {}",
                "Showing:".dimmed(),
                offset.unwrap_or(0) + 1,
                offset.unwrap_or(0) + shown,
                total
            );
        }
        print_habit_count(shown);
    }

    Ok(())
}
