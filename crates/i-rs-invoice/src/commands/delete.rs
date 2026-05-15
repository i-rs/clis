use crate::presentation::{print_error, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct DeleteArgs {
    #[arg(help = "Invoice ID")]
    pub id: String,

    #[arg(long, help = "Output format")]
    pub format: Option<OutputFormat>,
}

pub fn run_delete(args: DeleteArgs) -> Result<()> {
    let mut store = storage::load_store()?;

    let format = args.format.unwrap_or(OutputFormat::Default);

    if store.get_entry(&args.id).is_none() {
        match format {
            OutputFormat::Json => {
                println!("{}", crate::presentation::output_error(&format!("Invoice '{}' not found", args.id), "NOT_FOUND", OutputFormat::Json));
            }
            _ => {
                print_error(&format!("Invoice '{}' not found", args.id));
            }
        }
        anyhow::bail!("Invoice '{}' not found", args.id);
    }

    store.remove_entry(&args.id);
    storage::save_store(&store)?;

    match format {
        OutputFormat::Json => {}
        _ => {
            print_success(&format!("Invoice '{}' deleted successfully", args.id));
        }
    }

    Ok(())
}
