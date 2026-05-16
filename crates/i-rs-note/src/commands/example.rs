use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-note Examples".bold().cyan());
    println!();

    println!("{}", "Add Note:".bold().green());
    println!(
        "  i-rs-note add meeting --title \"Team Meeting Notes\" --tag work --content \"Agenda items\" --content \"Discussion points\""
    );
    println!("  i-rs-note add idea --tag personal --content \"New app concept\"");
    println!();

    println!("{}", "List Notes:".bold().green());
    println!("  i-rs-note list");
    println!("  i-rs-note list --tag work");
    println!();

    println!("{}", "Get Note:".bold().green());
    println!("  i-rs-note get meeting");
    println!();

    println!("{}", "Update Note:".bold().green());
    println!("  i-rs-note update meeting --content \"New content\" --tag important");
    println!();

    println!("{}", "Delete Note:".bold().green());
    println!("  i-rs-note delete meeting");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-note list --json");
    println!("  i-rs-note get meeting --json");
    println!();
}
