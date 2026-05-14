use crate::presentation::{output_error, output_item, print_error, print_header, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct DeleteArgs {
    #[arg(help = "Book name to delete")]
    pub name: String,
}

pub fn delete(args: DeleteArgs, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.books.contains_key(&args.name) {
        let msg = format!("Book '{}' not found", args.name);
        print_error(&msg);
        if matches!(output_format, OutputFormat::Json) {
            println!("{}", output_error(&msg, "NOT_FOUND", output_format));
        }
        anyhow::bail!(msg);
    }

    let book = storage::delete_book(&args.name, &mut store);
    storage::save_store(&store)?;

    match output_format {
        OutputFormat::Json => {
            if let Some(b) = book {
                println!("{}", output_item(&b, output_format));
            }
        }
        OutputFormat::Table | OutputFormat::Default => {
            print_header("Book Deleted");
            print_success(&format!("Book '{}' has been deleted", args.name));
        }
    }

    Ok(())
}
