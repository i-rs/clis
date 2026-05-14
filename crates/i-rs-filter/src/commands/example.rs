use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-filter Examples".bold().cyan());
    println!();

    println!("{}", "Record filter cleaning:".bold().green());
    println!("  i-rs-filter add \"Air Purifier\" HEPA");
    println!("  i-rs-filter add \"Vacuum\" dust --tag living-room");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-filter list");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-filter get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-filter delete abc12345");
    println!();
}