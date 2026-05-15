use crate::presentation::{print_success, print_header};
use owo_colors::OwoColorize;

pub fn handle_checkin(name: String) -> anyhow::Result<()> {
    let habit = crate::service::checkin_habit(&name)?;

    print_header("Habit Checkin");
    println!("{} {}", "Name:".style(owo_colors::Style::new().bold()), name);
    println!("{} {}", "Checkins:".style(owo_colors::Style::new().bold()), habit.checkins.len());
    print_success(&format!("Checked in for habit '{name}'!"));

    Ok(())
}
