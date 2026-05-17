use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-sheet Examples".bold().cyan());
    println!();

    println!("{}", "Record sheet change:".bold().green());
    println!("  i-rs-sheet add bedsheet");
    println!("  i-rs-sheet add pillowcase --tag bedroom");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-sheet list");
    println!("  i-rs-sheet list --tag bedroom");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-sheet get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-sheet delete abc12345");
    println!();
}
