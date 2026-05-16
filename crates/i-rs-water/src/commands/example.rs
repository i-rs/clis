use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-water Examples".bold().cyan());
    println!();

    println!("{}", "Record water intake:".bold().green());
    println!("  i-rs-water add 250");
    println!("  i-rs-water add 500 --tag morning");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-water list");
    println!("  i-rs-water list --tag morning");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-water get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-water delete abc12345");
    println!();
}
