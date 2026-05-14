use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-remind Examples".bold().cyan());
    println!();

    println!("{}", "Add Remind:".bold().green());
    println!("  i-rs-remind add meeting 2025-06-15 --title \"Team Meeting\" --tag work --content \"Discuss project进展\"");
    println!("  i-rs-remind add birthday 2025-07-20 --title \"Friend Birthday\" --tag personal");
    println!();

    println!("{}", "List Reminds:".bold().green());
    println!("  i-rs-remind list");
    println!("  i-rs-remind list --tag work");
    println!();

    println!("{}", "Get Remind:".bold().green());
    println!("  i-rs-remind get meeting");
    println!();

    println!("{}", "Mark as Done:".bold().green());
    println!("  i-rs-remind done meeting");
    println!();

    println!("{}", "Update Remind:".bold().green());
    println!("  i-rs-remind update meeting --event-date 2025-06-20 --content \"New agenda\"");
    println!();

    println!("{}", "Delete Remind:".bold().green());
    println!("  i-rs-remind delete meeting");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-remind list --json");
    println!("  i-rs-remind get meeting --json");
    println!();
}