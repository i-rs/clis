use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-recur Examples".bold().cyan());
    println!();

    println!("{}", "Add Recurring Expense:".bold().green());
    println!("  i-rs-recur add \"Netflix\" 15.99 USD monthly 2024-01-15 --tag entertainment");
    println!("  i-rs-recur add \"Rent\" 3000 CNY monthly 2024-01-01 --tag housing");
    println!("  i-rs-recur add \"Gym\" 200 CNY monthly 2024-02-01 --tag fitness");
    println!();

    println!("{}", "List Expenses:".bold().green());
    println!("  i-rs-recur list");
    println!("  i-rs-recur list --tag entertainment");
    println!();

    println!("{}", "Get Details:".bold().green());
    println!("  i-rs-recur get Netflix");
    println!();

    println!("{}", "Update Expense:".bold().green());
    println!("  i-rs-recur update Netflix --amount 19.99");
    println!();

    println!("{}", "Delete Expense:".bold().green());
    println!("  i-rs-recur delete \"Old Subscription\"");
    println!();

    println!("{}", "Frequencies:".bold().yellow());
    println!("  daily, weekly, monthly, quarterly, yearly");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-recur list --json");
    println!();
}
