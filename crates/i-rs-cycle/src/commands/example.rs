use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-cycle Examples".bold().cyan());
    println!();

    println!("{}", "Record cycle event:".bold().green());
    println!("  i-rs-cycle add 2024-01-15 period");
    println!("  i-rs-cycle add 2024-01-15 spotting --symptom cramps");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-cycle list");
    println!("  i-rs-cycle list --tag health");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-cycle get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-cycle delete abc12345");
    println!();
}