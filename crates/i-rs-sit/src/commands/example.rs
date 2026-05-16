use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-sit Examples".bold().cyan());
    println!();

    println!("{}", "Record sitting duration:".bold().green());
    println!("  i-rs-sit add 60");
    println!("  i-rs-sit add 120 --tag work");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-sit list");
    println!("  i-rs-sit list --tag work");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-sit get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-sit delete abc12345");
    println!();
}
