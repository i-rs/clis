use clap::Subcommand;
use std::io::Read;

#[derive(Subcommand, Debug, Clone)]
pub enum DataCommand {
    #[command(about = "Export all data as JSON")]
    Export,
    #[command(about = "Import data from JSON file or stdin")]
    Import {
        #[arg(value_name = "FILE")]
        file: Option<String>,
    },
    #[command(about = "Clear all data")]
    Clear,
}

pub fn handle(command: &DataCommand) -> anyhow::Result<()> {
    match command {
        DataCommand::Export => {
            let exported = crate::storage::export_data()?;
            println!("{}", exported);
            Ok(())
        }
        DataCommand::Import { file } => {
            let input = if let Some(path) = &file {
                ::std::fs::read_to_string(path)?
            } else {
                let mut buf = String::new();
                ::std::io::stdin().read_to_string(&mut buf)?;
                buf
            };
            crate::storage::import_data(&input)?;
            println!("Data imported successfully");
            Ok(())
        }
        DataCommand::Clear => {
            crate::storage::clear_data()?;
            println!("Data cleared successfully");
            Ok(())
        }
    }
}
