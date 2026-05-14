use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-want Examples".bold().cyan());
    println!();

    println!("{}", "Add items:".bold().green());
    println!("  i-rs-want add \"iPhone 15\" --price 799 --currency USD --priority high --url https://apple.com");
    println!("  i-rs-want add \"Mechanical Keyboard\" --price 200 --currency CNY --priority medium");
    println!();

    println!("{}", "List items:".bold().green());
    println!("  i-rs-want list");
    println!("  i-rs-want list --tag electronics");
    println!();

    println!("{}", "Get item:".bold().green());
    println!("  i-rs-want get \"iPhone 15\"");
    println!();

    println!("{}", "Mark as done:".bold().green());
    println!("  i-rs-want update \"iPhone 15\" --done");
    println!();

    println!("{}", "Delete item:".bold().green());
    println!("  i-rs-want delete \"Old Item\"");
    println!();
}
