use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_set(target: f64) -> Result<()> {
    let mut store = storage::load_store()?;

    store.set_target(target);
    storage::save_store(&store)?;

    print_success(&format!("✓ Target height set to {} cm", target.green()));

    Ok(())
}

pub fn handle_target() -> Result<()> {
    let store = storage::load_store()?;

    match store.get_target() {
        Some(target) => {
            println!();
            println!("{}", "Target Height:".bold().cyan());
            println!("  {} cm", target.green());
        }
        None => {
            print_error("No target height set. Use 'set <height>' to set a target.");
        }
    }

    Ok(())
}
