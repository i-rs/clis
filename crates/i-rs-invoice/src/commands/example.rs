use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ExampleArgs {}

pub fn run_example(_args: ExampleArgs) -> Result<()> {
    println!(r#"
=== i-rs-invoice Examples ===

# Add an electronic invoice
i-rs-invoice add "Office Supplies" --amount 299.99 --type electronic --tags expense --tags office

# Add a paper invoice and mark as reimbursed
i-rs-invoice add "Travel Expense" --amount 1500.00 --date 2024-01-15 --type paper --reimbursed --tags travel

# List all invoices
i-rs-invoice list

# List unreimbursed invoices
i-rs-invoice list --unreimbursed

# List invoices by tag
i-rs-invoice list --tag expense

# Get invoice details
i-rs-invoice get <invoice-id>

# Update invoice
i-rs-invoice update <invoice-id> --name "Updated Name" --reimbursed
i-rs-invoice update <invoice-id> --add-tags business --remove-tags personal

# View statistics
i-rs-invoice stats
i-rs-invoice stats --tag expense
i-rs-invoice stats --reimbursed

# Delete invoice
i-rs-invoice delete <invoice-id>

# JSON output
i-rs-invoice list --json
i-rs-invoice get <invoice-id> --json
"#);

    Ok(())
}
