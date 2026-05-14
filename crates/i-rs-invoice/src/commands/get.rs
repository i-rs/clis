use crate::presentation::{print_error, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;

#[derive(Args)]
pub struct GetArgs {
    #[arg(help = "Invoice ID")]
    pub id: String,

    #[arg(long, help = "Output format")]
    pub format: Option<OutputFormat>,
}

pub fn run_get(args: GetArgs) -> Result<()> {
    let store = storage::load_store()?;

    let invoice = storage::get_entry(&store, &args.id);

    let format = args.format.unwrap_or(OutputFormat::Default);

    match invoice {
        Some(inv) => {
            match format {
                OutputFormat::Json => {
                    println!("{}", crate::presentation::output_item(inv, OutputFormat::Json));
                }
                _ => {
                    print_header("Invoice Details");
                    println!("{}: {}", "ID".cyan(), inv.id);
                    println!("{}: {}", "Name".cyan(), inv.name);
                    println!("{}: {:.2}", "Amount".cyan(), inv.amount);
                    println!("{}: {}", "Date".cyan(), inv.date.format("%Y-%m-%d"));
                    println!("{}: {}", "Type".cyan(), inv.invoice_type);
                    println!("{}: {}", "Reimbursed".cyan(), if inv.reimbursed { "Yes" } else { "No" });
                    println!("{}: {}", "Tags".cyan(), inv.tags.join(", "));
                    if !inv.remark.is_empty() {
                        println!("{}: {}", "Remark".cyan(), inv.remark.join(", "));
                    }
                    println!("{}: {}", "Created".cyan(), inv.created_at.format("%Y-%m-%d %H:%M:%S"));
                    println!("{}: {}", "Updated".cyan(), inv.updated_at.format("%Y-%m-%d %H:%M:%S"));
                }
            }
        }
        None => {
            print_error(&format!("Invoice '{}' not found", args.id));
            anyhow::bail!("Invoice '{}' not found", args.id);
        }
    }

    Ok(())
}
