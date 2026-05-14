use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-meal Examples".bold().cyan());
    println!();

    println!("{}", "Add meals:".bold().green());
    println!("  i-rs-meal add breakfast \" oatmeal, milk\" 2024-01-15 --calories 300");
    println!("  i-rs-meal add lunch \" rice, chicken\" 2024-01-15 --tag work");
    println!("  i-rs-meal add dinner \" salad, fish\" 2024-01-15");
    println!();

    println!("{}", "List meals:".bold().green());
    println!("  i-rs-meal list");
    println!("  i-rs-meal list --date 2024-01-15");
    println!();

    println!("{}", "Get meal:".bold().green());
    println!("  i-rs-meal get abc12345");
    println!();

    println!("{}", "Delete meal:".bold().green());
    println!("  i-rs-meal delete abc12345");
    println!();
}
