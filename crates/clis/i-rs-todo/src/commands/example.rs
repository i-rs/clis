use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-todo Examples".bold().cyan());
    println!();

    println!("{}", "Add Todo:".bold().green());
    println!("  i-rs-todo add buy-milk --title \"Buy milk\" --priority high --tag shopping");
    println!(
        "  i-rs-todo add read-book --title \"Read Rust book\" --priority medium --tag learning --content \"Chapter 5\""
    );
    println!();

    println!("{}", "List Todos:".bold().green());
    println!("  i-rs-todo list");
    println!("  i-rs-todo list --pending");
    println!("  i-rs-todo list --done");
    println!("  i-rs-todo list --tag shopping");
    println!();

    println!("{}", "Get Todo:".bold().green());
    println!("  i-rs-todo get buy-milk");
    println!();

    println!("{}", "Mark as Done:".bold().green());
    println!("  i-rs-todo done buy-milk");
    println!();

    println!("{}", "Update Todo:".bold().green());
    println!("  i-rs-todo update buy-milk --priority medium --tag shopping --tag groceries");
    println!();

    println!("{}", "Delete Todo:".bold().green());
    println!("  i-rs-todo delete buy-milk");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-todo list --json");
    println!("  i-rs-todo get buy-milk --json");
    println!();

    println!("{}", "Priority Values:".bold().yellow());
    println!("  high, medium, low");
    println!();
}
