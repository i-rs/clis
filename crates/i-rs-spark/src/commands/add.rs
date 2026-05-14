use crate::models::SparkEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    content: String,
    source: Option<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = SparkEntry::new(content.clone(), source, tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    let preview = if content.len() > 30 { format!("{}...", &content[..30]) } else { content };
    print_success(&format!("✓ Spark recorded: {}", preview.green()));

    Ok(())
}
