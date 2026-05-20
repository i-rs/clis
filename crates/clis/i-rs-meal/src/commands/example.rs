use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-meal Examples".bold().cyan());
    println!();

    println!("{}", "Add meals:".bold().green());
    println!("  i-rs-meal add breakfast --food \"oatmeal, milk\" --calories 300");
    println!("  i-rs-meal add lunch --food \"rice, chicken\" --tag work");
    println!("  i-rs-meal add dinner --food \"salad, fish\"");
    println!();

    println!("{}", "List meals:".bold().green());
    println!("  i-rs-meal list");
    println!("  i-rs-meal list --date 2024-01-15");
    println!();

    println!("{}", "Get meal:".bold().green());
    println!("  i-rs-meal get abc12345");
    println!();

    println!("{}", "Update meal:".bold().green());
    println!("  i-rs-meal update abc12345 --food \"New food\"");
    println!("  i-rs-meal update abc12345 --calories 500");
    println!();

    println!("{}", "Delete meal:".bold().green());
    println!("  i-rs-meal delete abc12345");
    println!();
}
