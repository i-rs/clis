use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-spark Examples".bold().cyan());
    println!();

    println!("{}", "Record a spark:".bold().green());
    println!("  i-rs-spark add \"New app idea: AI-powered meal planner\" --source \"Dream\"");
    println!("  i-rs-spark add \"Write a Rust CLI tool for tracking habits\" --source \"Reddit\" --tag coding");
    println!();

    println!("{}", "List sparks:".bold().green());
    println!("  i-rs-spark list");
    println!("  i-rs-spark list --tag coding");
    println!();

    println!("{}", "Get spark:".bold().green());
    println!("  i-rs-spark get abc12345");
    println!();

    println!("{}", "Delete spark:".bold().green());
    println!("  i-rs-spark delete abc12345");
    println!();
}
