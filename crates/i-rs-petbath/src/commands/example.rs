use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-petbath Examples".bold().cyan());
    println!();

    println!("{}", "Record pet bath:".bold().green());
    println!("  i-rs-petbath add \"Cat\"");
    println!("  i-rs-petbath add \"Dog\" --tag summer");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-petbath list");
    println!("  i-rs-petbath list --tag summer");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-petbath get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-petbath delete abc12345");
    println!();
}