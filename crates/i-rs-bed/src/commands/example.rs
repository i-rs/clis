use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-bed Examples".bold().cyan());
    println!();

    println!("{}", "Record bed item replacement:".bold().green());
    println!("  i-rs-bed add mattress");
    println!("  i-rs-bed add pillow --tag bedroom");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-bed list");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-bed get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-bed delete abc12345");
    println!();
}