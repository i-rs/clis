use crate::presentation::{print_header, print_success, OutputFormat};
use owo_colors::OwoColorize;

pub fn handle_checkin(name: String, format: OutputFormat) -> anyhow::Result<()> {
    let mut store = crate::storage::load_store()?;
    let habit = crate::service::checkin_habit(&mut store, &name)?;
    crate::storage::save_store(&store)?;

    if format.is_json() {
        let item = crate::models::ListItem::from(&habit);
        println!("{}", crate::presentation::output_item(&item, format));
        return Ok(());
    }

    print_header("Habit Checkin");
    println!(
        "{} {}",
        "Name:".style(owo_colors::Style::new().bold()),
        name
    );
    println!(
        "{} {}",
        "Checkins:".style(owo_colors::Style::new().bold()),
        habit.checkins.len()
    );
    print_success(&format!("Checked in for habit '{name}'!"));

    Ok(())
}
