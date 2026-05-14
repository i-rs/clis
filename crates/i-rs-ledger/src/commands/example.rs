use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-ledger Examples".bold().cyan());
    println!();

    println!("{}", "Add Income:".bold().green());
    println!("  i-rs-ledger add 2024-01-15 5000 CNY income salary --tag monthly");
    println!();

    println!("{}", "Add Expense:".bold().green());
    println!("  i-rs-ledger add 2024-01-15 150.50 CNY expense food --tag dining --remark \"Lunch with client\"");
    println!();

    println!("{}", "Add Transfer:".bold().green());
    println!("  i-rs-ledger add 2024-01-15 1000 CNY transfer savings --remark \"Monthly savings\"");
    println!();

    println!("{}", "List Entries:".bold().green());
    println!("  i-rs-ledger list");
    println!("  i-rs-ledger list --category expense");
    println!();

    println!("{}", "Get Entry:".bold().green());
    println!("  i-rs-ledger get abc12345");
    println!();

    println!("{}", "Update Entry:".bold().green());
    println!("  i-rs-ledger update abc12345 --amount 200 --remark \"Updated\"");
    println!();

    println!("{}", "Delete Entry:".bold().green());
    println!("  i-rs-ledger delete abc12345");
    println!();

    println!("{}", "Entry Types:".bold().yellow());
    println!("  income, expense, transfer");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-ledger list --json");
    println!();
}
