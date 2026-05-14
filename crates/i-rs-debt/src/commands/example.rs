use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {}

pub fn run(_args: &Args) -> Result<()> {
    println!(r#"
=== i-rs-debt Examples ===

# Add a credit card debt
i-rs-debt add "Credit Card A" --debt-type credit_card --amount 10000 --interest-rate 15.0 --due-date 2026-06-01

# Add a loan
i-rs-debt add "Car Loan" --debt-type loan --amount 50000 --interest-rate 5.5 --tags car,vehicle

# Add a borrowed debt
i-rs-debt add "Friend's Money" --debt-type borrowed --amount 2000 --remark "Borrowed from Zhang San"

# List all debts
i-rs-debt list

# List debts filtered by tag
i-rs-debt list --tag car

# Show overdue debts only
i-rs-debt list --overdue

# Show paid off debts only
i-rs-debt list --paid-off

# Get debt details
i-rs-debt get "Credit Card A"

# Get debt details with payment history
i-rs-debt get "Credit Card A" --payments

# Record a payment
i-rs-debt pay "Credit Card A" --amount 500 --note "Monthly payment"

# Update debt information
i-rs-debt update "Credit Card A" --interest-rate 12.0 --add-tags important

# Update debt due date
i-rs-debt update "Credit Card A" --due-date 2026-07-01

# View statistics
i-rs-debt stats

# View statistics by type
i-rs-debt stats --by-type

# Delete a debt
i-rs-debt delete "Old Debt" --force

# JSON output
i-rs-debt list --json
i-rs-debt get "Credit Card A" --json
i-rs-debt stats --json
"#);

    Ok(())
}
