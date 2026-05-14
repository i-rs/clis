use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-purify Examples".bold().cyan());
    println!();

    println!("{}", "Record filter replacement:".bold().green());
    println!("  i-rs-purify add \"RO Membrane\"");
    println!("  i-rs-purify add \"Carbon Filter\" --tag kitchen");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-purify list");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-purify get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-purify delete abc12345");
    println!();
}