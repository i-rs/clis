use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-step Examples".bold().cyan());
    println!();

    println!("{}", "Add steps:".bold().green());
    println!("  i-rs-step add 10000 2024-01-15");
    println!("  i-rs-step add 8000 2024-01-15 --distance 6.5");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-step list");
    println!();

    println!("{}", "Get record:".bold().green());
    println!("  i-rs-step get 2024-01-15");
    println!();

    println!("{}", "Update record:".bold().green());
    println!("  i-rs-step update 2024-01-15 --steps 12000");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-step delete 2024-01-15");
    println!();
}
