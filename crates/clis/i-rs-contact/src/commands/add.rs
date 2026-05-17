use crate::presentation::{print_header, print_success};
use crate::storage;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;

pub fn handle_add(
    name: String,
    phone: Option<String>,
    email: Option<String>,
    relationship: Option<String>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> anyhow::Result<()> {
    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    let mut store = storage::load_store()?;

    if store.get_entry(&name).is_some() {
        anyhow::bail!("Contact '{name}' already exists");
    }

    let contact = storage::add_contact(
        &mut store,
        name.clone(),
        phone.unwrap_or_default(),
        email.unwrap_or_default(),
        relationship.unwrap_or_default(),
        tags,
        remark,
    )?;
    storage::save_store(&store)?;

    print_header("Contact Created");
    println!(
        "{} {}",
        "Name:".style(owo_colors::Style::new().bold()),
        name
    );
    if !contact.phone.is_empty() {
        println!(
            "{} {}",
            "Phone:".style(owo_colors::Style::new().bold()),
            contact.phone
        );
    }
    if !contact.email.is_empty() {
        println!(
            "{} {}",
            "Email:".style(owo_colors::Style::new().bold()),
            contact.email
        );
    }
    if !contact.relationship.is_empty() {
        println!(
            "{} {}",
            "Relationship:".style(owo_colors::Style::new().bold()),
            contact.relationship
        );
    }
    print_success(&format!("Contact '{name}' created successfully"));

    Ok(())
}
