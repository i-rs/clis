use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-tick Examples".bold().cyan());
    println!();

    println!("{}", "Record duration:".bold().green());
    println!("  i-rs-tick add \"Deep Work\" 1500 --tag focus");
    println!("  i-rs-tick add \"Coding\" 7200 --tag work");
    println!("  i-rs-tick add \"Reading\" 3600 --tag learning");
    println!();

    println!("{}", "With start time:".bold().green());
    println!("  i-rs-tick add \"Meeting\" 3600 --started-at \"2024-01-15 14:00\"");
    println!();

    println!("{}", "List entries:".bold().green());
    println!("  i-rs-tick list");
    println!("  i-rs-tick list --tag work");
    println!();

    println!("{}", "Get entry:".bold().green());
    println!("  i-rs-tick get abc12345");
    println!();

    println!("{}", "Delete entry:".bold().green());
    println!("  i-rs-tick delete abc12345");
    println!();

    println!("{}", "Duration format:".bold().yellow());
    println!("  Seconds: 3600 = 1 hour");
    println!("  Common: 1500 (25min), 1800 (30min), 3600 (1h), 7200 (2h)");
    println!();
}
