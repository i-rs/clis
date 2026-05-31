use crate::presentation::{print_header, print_success, OutputFormat};
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    description: Option<String>,
    frequency: String,
    tags: Vec<String>,
    remark: Vec<String>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let mut store = crate::storage::load_store()?;
    let habit = crate::service::add_habit(
        &mut store,
        name.clone(),
        description.unwrap_or_default(),
        frequency,
        tags,
        remark,
    )?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let item = crate::models::ListItem::from(&habit);
        println!("{}", crate::presentation::output_item(&item, format));
        return Ok(());
    }

    print_header("Habit Created");
    println!(
        "{} {}",
        "Name:".style(owo_colors::Style::new().bold()),
        habit.name
    );
    print_success(&format!("Habit '{}' created successfully", name));

    Ok(())
}
