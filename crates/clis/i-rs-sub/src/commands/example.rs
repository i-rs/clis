use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-sub Examples".bold().cyan());
    println!();

    println!("{}", "Add subscription:".bold().green());
    println!("  i-rs-sub add \"Netflix\" 15.99 USD monthly 2024-01-15 --url https://netflix.com");
    println!("  i-rs-sub add \"Spotify\" 10.99 USD monthly 2024-01-01 --tag music");
    println!();

    println!("{}", "List subscriptions:".bold().green());
    println!("  i-rs-sub list");
    println!("  i-rs-sub list --tag music");
    println!();

    println!("{}", "Get subscription:".bold().green());
    println!("  i-rs-sub get Netflix");
    println!();

    println!("{}", "Update subscription:".bold().green());
    println!("  i-rs-sub update Netflix --amount 19.99");
    println!();

    println!("{}", "Delete subscription:".bold().green());
    println!("  i-rs-sub delete \"Old Service\"");
    println!();

    println!("{}", "Billing Cycles:".bold().yellow());
    println!("  daily, weekly, monthly, quarterly, yearly");
    println!();
}
