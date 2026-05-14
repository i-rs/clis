use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-cal Examples".bold().cyan());
    println!();

    println!("{}", "Record calorie intake:".bold().green());
    println!("  i-rs-cal add \"Apple\" 95");
    println!("  i-rs-cal add \"Pizza\" 285 --tag lunch");
    println!("  i-rs-cal add \"Burger\" 350 --date 2024-01-15");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-cal list");
    println!("  i-rs-cal list --tag lunch");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-cal get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-cal delete abc12345");
    println!();
}