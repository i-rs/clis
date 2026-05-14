use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-ac Examples".bold().cyan());
    println!();

    println!("{}", "Record AC cleaning:".bold().green());
    println!("  i-rs-ac add \"Living Room\"");
    println!("  i-rs-ac add \"Bedroom\" --tag summer");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-ac list");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-ac get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-ac delete abc12345");
    println!();
}