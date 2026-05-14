use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Debt name to delete")]
    pub name: String,
    #[arg(short, long, help = "Force delete without confirmation")]
    pub force: bool,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.debts.contains_key(&args.name) {
        anyhow::bail!("Debt '{}' not found", args.name);
    }

    if !args.force {
        println!("Are you sure you want to delete debt '{}'? (y/N)", args.name);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    storage::delete_debt(&mut store, &args.name);
    storage::save_store(&store)?;

    if output_format == OutputFormat::Json {
        println!("{{\"success\": true, \"message\": \"Debt '{}' deleted\"}}", args.name);
    } else {
        print_success(&format!("Debt '{}' deleted successfully", args.name));
    }

    Ok(())
}
