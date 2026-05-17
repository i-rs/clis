use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-password Examples".bold().cyan());
    println!();

    println!("{}", "Add Password Entry:".bold().green());
    println!("  i-rs-password add github https://github.com --account user@example.com --tag work");
    println!(
        "  i-rs-password add gmail https://gmail.com --account my@gmail.com --password --tag personal"
    );
    println!();

    println!("{}", "List Password Entries:".bold().green());
    println!("  i-rs-password list");
    println!("  i-rs-password list --tag work");
    println!();

    println!("{}", "Get Password Entry:".bold().green());
    println!("  i-rs-password get github");
    println!("  i-rs-password get github --show-password");
    println!();

    println!("{}", "Update Password Entry:".bold().green());
    println!("  i-rs-password update github --account new@email.com");
    println!("  i-rs-password update gmail --tag work --remark \"Important account\"");
    println!();

    println!("{}", "Delete Password Entry:".bold().green());
    println!("  i-rs-password delete gmail");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-password list --json");
    println!("  i-rs-password get github --json");
    println!();
}
