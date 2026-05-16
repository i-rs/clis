use crate::models::{AssetType, Investment};
use crate::presentation::{format_table, output_list, print_investment_count, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    asset_type: Option<String>,
    tag: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let investments: Vec<&Investment> = if let Some(type_str) = asset_type {
        let parsed_type: AssetType = type_str.parse().map_err(|e| anyhow::anyhow!("{e}"))?;
        storage::filter_by_type(&store, Some(&parsed_type))
    } else if let Some(ref tag) = tag {
        storage::filter_by_tag(&store, Some(tag))
    } else {
        store.investments.values().collect()
    };

    i_rs_core::handle_empty!(investments, format, None::<&str>, "No investments found.");

    match format {
        OutputFormat::Json => {
            let json_output = output_list(&investments, investments.len(), tag.as_deref(), format);
            println!("{json_output}");
        }
        OutputFormat::Table | OutputFormat::Default => {
            let table = format_table(&investments);
            println!("{table}");
            print_investment_count(investments.len());
        }
    }

    Ok(())
}
