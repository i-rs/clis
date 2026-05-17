use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct DeleteArgs {
    #[arg(help = "Event name to delete")]
    pub name: String,
}

pub fn run(args: &DeleteArgs, json: bool) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.events.contains_key(&args.name) {
        print_error(&format!("Event '{}' not found", args.name));
        if json {
            println!(
                r#"{{"success": false, "error": {{"code": "NOT_FOUND", "message": "Event '{}' not found"}}}}"#,
                args.name
            );
        }
        anyhow::bail!("Event '{}' not found", args.name);
    }

    store.remove_entry(&args.name);
    storage::save_store(&store)?;

    if json {
        println!(
            r#"{{"success": true, "data": {{"deleted": "{}"}}}}"#,
            args.name
        );
    } else {
        print_success(&format!("Event '{}' deleted successfully", args.name));
    }

    Ok(())
}
