use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-time Examples".bold().cyan());
    println!();

    println!("{}", "Start a timer:".bold().green());
    println!("  i-rs-time start \"Working on project\"");
    println!("  i-rs-time start \"Meeting\" --tag work");
    println!("  i-rs-time start \"Coding\" --tag development --remark \"Feature implementation\"");
    println!();

    println!("{}", "Stop the timer:".bold().green());
    println!("  i-rs-time stop");
    println!();

    println!("{}", "List time entries:".bold().green());
    println!("  i-rs-time list");
    println!("  i-rs-time list --tag work");
    println!();

    println!("{}", "View statistics:".bold().green());
    println!("  i-rs-time stats today");
    println!("  i-rs-time stats yesterday");
    println!("  i-rs-time stats week");
    println!();

    println!("{}", "Generate reports:".bold().green());
    println!("  i-rs-time report --days 7");
    println!("  i-rs-time report --start 2024-01-01 --end 2024-01-31");
    println!();

    println!("{}", "Manage entries:".bold().green());
    println!("  i-rs-time get <entry-id>");
    println!("  i-rs-time delete <entry-id>");
    println!();

    println!("{}", "JSON output:".bold().yellow());
    println!("  i-rs-time list --json");
    println!("  i-rs-time stats today --json");
    println!();
}
