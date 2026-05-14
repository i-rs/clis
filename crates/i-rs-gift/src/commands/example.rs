use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-gift Examples".bold().cyan());
    println!();

    println!("{}", "Add Gift (Sent):".bold().green());
    println!("  i-rs-gift add \"Birthday Watch\" sent \"Mom\" birthday 500 2024-12-25 --tag family --remark \"Swiss brand\"");
    println!();

    println!("{}", "Add Gift (Received):".bold().green());
    println!("  i-rs-gift add \"AirPods\" received \"Boss\" christmas 1200 2024-12-25 --tag work");
    println!();

    println!("{}", "List Gifts:".bold().green());
    println!("  i-rs-gift list");
    println!("  i-rs-gift list --type sent");
    println!("  i-rs-gift list --type received");
    println!("  i-rs-gift list --tag family");
    println!();

    println!("{}", "Get Gift Details:".bold().green());
    println!("  i-rs-gift get \"Birthday Watch\"");
    println!("  i-rs-gift get \"Birthday Watch\" --json");
    println!();

    println!("{}", "Delete Gift:".bold().green());
    println!("  i-rs-gift delete \"Birthday Watch\"");
    println!();

    println!("{}", "Statistics:".bold().green());
    println!("  i-rs-gift stats");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-gift list --json");
    println!("  i-rs-gift get \"Birthday Watch\" --json");
    println!();
}
