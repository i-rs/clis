use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-towel Examples".bold().cyan());
    println!();

    println!("{}", "Record towel replacement:".bold().green());
    println!("  i-rs-towel add bath");
    println!("  i-rs-towel add face --tag bedroom");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-towel list");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-towel get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-towel delete abc12345");
    println!();
}