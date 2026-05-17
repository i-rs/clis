use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-toothbrush Examples".bold().cyan());
    println!();

    println!("{}", "Record toothbrush replacement:".bold().green());
    println!("  i-rs-toothbrush add \"Electric\"");
    println!("  i-rs-toothbrush add \"Manual\" --tag travel");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-toothbrush list");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-toothbrush get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-toothbrush delete abc12345");
    println!();
}
