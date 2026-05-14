use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-quote Examples".bold().cyan());
    println!();

    println!("{}", "Add Quote:".bold().green());
    println!("  i-rs-quote add --content \"The only way to do great work is to love what you do.\" --author \"Steve Jobs\"");
    println!("  i-rs-quote add --content \"Quote text\" --author \"Author\" --source \"Book Name\" --tag inspiration --tag life");
    println!("  i-rs-quote add --content \"Be the change\" --author \"Gandhi\" --source \"Speech\" --tag wisdom --remark \"Great reminder!\")");
    println!();

    println!("{}", "List Quotes:".bold().green());
    println!("  i-rs-quote list");
    println!("  i-rs-quote list --tag inspiration");
    println!("  i-rs-quote list --author \"Steve\"");
    println!();

    println!("{}", "Get Quote:".bold().green());
    println!("  i-rs-quote get <uuid>");
    println!();

    println!("{}", "Random Quote:".bold().green());
    println!("  i-rs-quote random");
    println!();

    println!("{}", "Delete Quote:".bold().green());
    println!("  i-rs-quote delete <uuid>");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-quote list --json");
    println!("  i-rs-quote get <uuid> --json");
    println!();
}
