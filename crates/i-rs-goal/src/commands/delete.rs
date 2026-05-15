use crate::presentation::{output_error, print_success, OutputFormat};
use crate::storage;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct DeleteArgs {
    #[arg(help = "Goal name")]
    pub name: String,
}

pub fn delete(args: DeleteArgs, output_format: OutputFormat) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;
    
    if storage::remove_entry(&mut store, &args.name)? {
        match output_format {
            OutputFormat::Json => {
                println!("{}", output_error(&format!("Goal '{}' deleted successfully", args.name), "goal_deleted", output_format));
            }
            OutputFormat::Table | OutputFormat::Default => {
                print_success(&format!("Goal '{}' deleted successfully", args.name));
            }
        }
    } else {
        anyhow::bail!("Goal '{}' not found", args.name);
    }
    
    Ok(())
}
