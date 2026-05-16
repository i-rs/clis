use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-feedpet Examples".bold().cyan());
    println!();

    println!("{}", "Record pet feeding:".bold().green());
    println!("  i-rs-feedpet add \"Cat\" dry-food \"50g\"");
    println!("  i-rs-feedpet add \"Dog\" wet-food \"200g\" --tag morning");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-feedpet list");
    println!("  i-rs-feedpet list --tag morning");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-feedpet get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-feedpet delete abc12345");
    println!();
}
