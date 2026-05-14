use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-dose Examples".bold().cyan());
    println!();

    println!("{}", "Record medication:".bold().green());
    println!("  i-rs-dose add \"Vitamin D\" \"1000\" \"IU\" --tag vitamins");
    println!("  i-rs-dose add \"Aspirin\" \"500\" \"mg\" --tag pain");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-dose list");
    println!("  i-rs-dose list --tag vitamins");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-dose get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-dose delete abc12345");
    println!();
}
