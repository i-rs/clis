use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-vocab Examples".bold().cyan());
    println!();

    println!("{}", "Add Word:".bold().green());
    println!("  i-rs-vocab add hello \"greeting; hello world\" --tag basic --example \"Hello, how are you?\"");
    println!("  i-rs-vocab add \"ephemeral\" \"lasting for a very short time\" --tag advanced");
    println!();

    println!("{}", "List Words:".bold().green());
    println!("  i-rs-vocab list");
    println!("  i-rs-vocab list --status new");
    println!("  i-rs-vocab list --status learning");
    println!("  i-rs-vocab list --tag basic");
    println!();

    println!("{}", "Get Word Details:".bold().green());
    println!("  i-rs-vocab get hello");
    println!();

    println!("{}", "Update Word:".bold().green());
    println!("  i-rs-vocab update hello --definition \"a greeting\"");
    println!("  i-rs-vocab update hello --status learning");
    println!();

    println!("{}", "Review/Quiz:".bold().green());
    println!("  i-rs-vocab quiz");
    println!("  i-rs-vocab quiz --count 10");
    println!();

    println!("{}", "Statistics:".bold().green());
    println!("  i-rs-vocab stats");
    println!();

    println!("{}", "Delete Word:".bold().green());
    println!("  i-rs-vocab delete hello");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-vocab list --json");
    println!("  i-rs-vocab get hello --json");
    println!();

    println!("{}", "Status Values:".bold().yellow());
    println!("  new, learning, mastered");
    println!();
}
