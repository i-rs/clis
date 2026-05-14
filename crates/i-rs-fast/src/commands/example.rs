use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-fast Examples".bold().cyan());
    println!();

    println!("{}", "Start fasting:".bold().green());
    println!("  i-rs-fast add 16");
    println!("  i-rs-fast add 24 --tag omad");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-fast list");
    println!("  i-rs-fast list --tag omad");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-fast get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-fast delete abc12345");
    println!();
}