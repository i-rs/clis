use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-aqua Examples".bold().cyan());
    println!();

    println!("{}", "Record water change:".bold().green());
    println!("  i-rs-aqua add");
    println!("  i-rs-aqua add --tank-size 100");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-aqua list");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-aqua get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-aqua delete abc12345");
    println!();
}