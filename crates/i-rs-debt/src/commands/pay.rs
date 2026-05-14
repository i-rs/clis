use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Debt name")]
    pub name: String,
    #[arg(short, long, help = "Payment amount")]
    pub amount: f64,
    #[arg(short, long, help = "Payment note")]
    pub note: Option<String>,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    let is_paid_off_before = {
        let debt = match storage::get_debt_mut(&mut store, &args.name) {
            Some(d) => d,
            None => {
                anyhow::bail!("Debt '{}' not found", args.name);
            }
        };

        if args.amount <= 0.0 {
            anyhow::bail!("Payment amount must be greater than 0");
        }

        if args.amount > debt.remaining {
            println!(
                "Note: Payment amount ({:.2}) exceeds remaining debt ({:.2})",
                args.amount, debt.remaining
            );
        }

        let is_paid_off = debt.remaining == 0.0;
        debt.add_payment(args.amount, args.note.clone());
        is_paid_off
    };

    storage::save_store(&store)?;

    if output_format == OutputFormat::Json {
        println!(
            "{{\"success\": true, \"data\": {{\"name\": \"{}\", \"paid\": {:.2}}}}}",
            args.name, args.amount
        );
    } else {
        print_success(&format!(
            "Payment of {:.2} recorded for '{}'",
            args.amount, args.name
        ));

        if is_paid_off_before {
            println!("{}", "Congratulations! This debt is fully paid off!".green().bold());
        }
    }

    Ok(())
}

use owo_colors::OwoColorize;
