use crate::presentation::{print_error, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use clap::Args;

#[derive(Args)]
pub struct UpdateArgs {
    #[arg(help = "Invoice ID")]
    pub id: String,

    #[arg(short, long, help = "New name")]
    pub name: Option<String>,

    #[arg(short, long, help = "New amount")]
    pub amount: Option<f64>,

    #[arg(short, long, help = "New date (YYYY-MM-DD)")]
    pub date: Option<String>,

    #[arg(short, long, help = "New invoice type: electronic or paper")]
    pub invoice_type: Option<String>,

    #[arg(short, long, action, help = "Mark as reimbursed")]
    pub reimbursed: Option<bool>,

    #[arg(short, long, help = "Add tags (can be specified multiple times)")]
    pub add_tags: Vec<String>,

    #[arg(short, long, help = "Remove tags (can be specified multiple times)")]
    pub remove_tags: Vec<String>,

    #[arg(short, long, help = "Add remarks (can be specified multiple times)")]
    pub add_remark: Vec<String>,

    #[arg(long, help = "Output format")]
    pub format: Option<OutputFormat>,
}

pub fn run_update(args: UpdateArgs) -> Result<()> {
    let mut store = storage::load_store()?;

    let invoice = storage::get_entry_mut(&mut store, &args.id);

    match invoice {
        Some(inv) => {
            if let Some(name) = args.name {
                inv.name = name;
            }

            if let Some(amount) = args.amount {
                inv.amount = amount;
            }

            if let Some(date_str) = args.date {
                inv.date = crate::models::parse_date(&date_str)?;
            }

            if let Some(type_str) = args.invoice_type {
                match type_str.parse() {
                    Ok(t) => inv.invoice_type = t,
                    Err(e) => {
                        print_error(&e);
                        anyhow::bail!("{}", e);
                    }
                };
            }

            if let Some(reimbursed) = args.reimbursed {
                inv.reimbursed = reimbursed;
            }

            for tag in args.add_tags {
                if !inv.tags.contains(&tag) {
                    inv.tags.push(tag);
                }
            }

            inv.tags.retain(|t| !args.remove_tags.contains(t));

            for remark in args.add_remark {
                if !inv.remark.contains(&remark) {
                    inv.remark.push(remark);
                }
            }

            inv.updated_at = Utc::now();

            storage::save_store(&store)?;

            print_success(&format!("Invoice '{}' updated successfully", args.id));
        }
        None => {
            print_error(&format!("Invoice '{}' not found", args.id));
            anyhow::bail!("Invoice '{}' not found", args.id);
        }
    }

    Ok(())
}
