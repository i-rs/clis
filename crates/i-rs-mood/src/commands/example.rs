use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-mood Examples".bold().cyan());
    println!();

    println!("{}", "Add Mood Record:".bold().green());
    println!("  i-rs-mood add 2025-06-14 happy --tag work --content \"Great day!\"");
    println!("  i-rs-mood add 2025-06-15 neutral --tag personal");
    println!();

    println!("{}", "List Records:".bold().green());
    println!("  i-rs-mood list");
    println!("  i-rs-mood list --days 7");
    println!();

    println!("{}", "List with Calendar:".bold().green());
    println!("  i-rs-mood list --calendar");
    println!("  i-rs-mood list --days 14 --calendar");
    println!();

    println!("{}", "Update Record:".bold().green());
    println!("  i-rs-mood update 2025-06-14 --mood excited --tag work --content \"Even better!\"");
    println!();

    println!("{}", "Delete Record:".bold().green());
    println!("  i-rs-mood delete 2025-06-15");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-mood list --json");
    println!();

    println!("{}", "Mood Values:".bold().yellow());
    println!("  happy, good, neutral, bad, terrible");
    println!();
}
