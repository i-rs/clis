use crate::models::{ContactRow, ListItem};
use crate::presentation::{format_table, print_header, OutputFormat, output_item};
use crate::storage;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let contact = store.get_entry(&name)
        .ok_or_else(|| anyhow::anyhow!("Contact '{name}' not found"))?;

    if format == OutputFormat::Json {
        let item: ListItem = contact.into();
        output_item(&item, format);
    } else {
        print_header("Contact Details");
        let style = owo_colors::Style::new().bold();
        
        let row = ContactRow::from_contact(contact);
        println!("{}", format_table(&[row]));
        
        println!("\n{} {}", "Contact Count:".style(style), contact.contact_count);
        if let Some(last) = contact.last_contact {
            println!("{} {}", "Last Contact:".style(style), last.format("%Y-%m-%d %H:%M"));
        }
        
        if !contact.remark.is_empty() {
            println!("\n{}", "Remarks:".style(style));
            for (i, remark) in contact.remark.iter().enumerate() {
                println!("  {}. {}", i + 1, remark);
            }
        }
    }

    Ok(())
}
