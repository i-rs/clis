use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-habit Examples".bold().cyan());
    println!();

    println!("{}", "Create a habit:".bold().green());
    println!(
        "  i-rs-habit add daily_walk --description \"Walk 30 minutes\" --frequency daily --tag health"
    );
    println!("  i-rs-habit add meditation --description \"Morning meditation\" --frequency daily");
    println!();

    println!("{}", "Checkin for a habit:".bold().green());
    println!("  i-rs-habit checkin daily_walk");
    println!("  i-rs-habit checkin meditation");
    println!();

    println!("{}", "List habits:".bold().green());
    println!("  i-rs-habit list");
    println!("  i-rs-habit list --tag health");
    println!();

    println!("{}", "Get habit details:".bold().green());
    println!("  i-rs-habit get daily_walk");
    println!();

    println!("{}", "Update habit:".bold().green());
    println!("  i-rs-habit update daily_walk --frequency weekly");
    println!();

    println!("{}", "Delete habit:".bold().green());
    println!("  i-rs-habit delete old_habit");
    println!();

    println!("{}", "Frequencies:".bold().yellow());
    println!("  daily, weekly, monthly, yearly, custom");
    println!();
}
