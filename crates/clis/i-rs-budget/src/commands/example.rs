use crate::presentation::print_header;
use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    print_header("i-rs-budget Examples");
    println!();

    println!("{}", "Add a monthly budget:".bold().cyan());
    println!("  i-rs-budget add food 500");
    println!("  i-rs-budget add food 500 --period monthly");
    println!();

    println!("{}", "Add budget with different periods:".bold().cyan());
    println!("  i-rs-budget add groceries 300 --period weekly");
    println!("  i-rs-budget add rent 2000 --period monthly");
    println!("  i-rs-budget add vacation 5000 --period yearly");
    println!();

    println!("{}", "Add expenses:".bold().cyan());
    println!("  i-rs-budget expense food 25.50 --description \"Lunch\"");
    println!("  i-rs-budget expense food 15.00 --description \"Coffee\"");
    println!("  i-rs-budget expense groceries 120.30 --date 2024-01-15");
    println!();

    println!("{}", "List budgets and expenses:".bold().cyan());
    println!("  i-rs-budget list budgets");
    println!("  i-rs-budget list expenses");
    println!("  i-rs-budget list expenses --category food");
    println!();

    println!("{}", "View budget statistics:".bold().cyan());
    println!("  i-rs-budget stats");
    println!("  i-rs-budget stats --period monthly");
    println!("  i-rs-budget stats --category food");
    println!();

    println!("{}", "Update budgets:".bold().cyan());
    println!("  i-rs-budget update food --amount 600");
    println!("  i-rs-budget update food --period weekly --tags weekly-expenses");
    println!();

    println!("{}", "Delete budgets and expenses:".bold().cyan());
    println!("  i-rs-budget delete --category food");
    println!("  i-rs-budget delete --expense-id abc12345");
    println!();

    println!("{}", "Get budget or expense details:".bold().cyan());
    println!("  i-rs-budget get --category food");
    println!("  i-rs-budget get --expense-id abc12345");
    println!();

    println!("{}", "JSON output:".bold().cyan());
    println!("  i-rs-budget list budgets --json");
    println!("  i-rs-budget stats --json");
    println!("  i-rs-budget get --category food --json");
    println!();
}
