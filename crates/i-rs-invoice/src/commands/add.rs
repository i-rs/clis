use crate::models::{Invoice, InvoiceType};
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use clap::Args;
use i_rs_core::utils::validation::validate_amount;
use owo_colors::OwoColorize;
use uuid::Uuid;

#[derive(Args)]
pub struct AddArgs {
    #[arg(help = "Invoice name")]
    pub name: String,

    #[arg(help = "Invoice amount")]
    pub amount: f64,

    #[arg(short, long, help = "Invoice date (YYYY-MM-DD), defaults to today")]
    pub date: Option<String>,

    #[arg(short, long, default_value = "electronic", help = "Invoice type: electronic or paper")]
    pub invoice_type: String,

    #[arg(short, long, action, help = "Mark as reimbursed")]
    pub reimbursed: bool,

    #[arg(short, long, help = "Tags (can be specified multiple times)")]
    pub tags: Vec<String>,

    #[arg(short, long, help = "Remarks (can be specified multiple times)")]
    pub remark: Vec<String>,

    #[arg(long, help = "Output format")]
    pub format: Option<crate::presentation::OutputFormat>,
}

pub fn run_add(args: AddArgs) -> Result<()> {
    if let Err(e) = validate_amount(args.amount) {
        eprintln!("{}", format!("Error: {}", e.message).red());
        anyhow::bail!("{}", e.message);
    }

    let mut store = storage::load_store()?;

    let invoice_type: InvoiceType = match args.invoice_type.parse() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", format!("Error: {}", e).red());
            anyhow::bail!("{}", e);
        }
    };

    let date = if let Some(date_str) = args.date {
        crate::models::parse_date(&date_str)?
    } else {
        Utc::now()
    };

    let now = Utc::now();
    let invoice = Invoice {
        id: Uuid::new_v4().to_string(),
        name: args.name,
        amount: args.amount,
        date,
        invoice_type,
        reimbursed: args.reimbursed,
        tags: args.tags,
        remark: args.remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, invoice);
    storage::save_store(&store)?;

    print_success("Invoice added successfully");

    Ok(())
}
