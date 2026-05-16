use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-walkdog Examples".bold().cyan());
    println!();

    println!("{}", "Record dog walk:".bold().green());
    println!("  i-rs-walkdog add \"Buddy\" 30");
    println!("  i-rs-walkdog add \"Max\" 45 --tag morning");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-walkdog list");
    println!("  i-rs-walkdog list --tag morning");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-walkdog get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-walkdog delete abc12345");
    println!();
}
