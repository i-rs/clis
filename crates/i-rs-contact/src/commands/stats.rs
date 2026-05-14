use crate::models::{ContactFrequency, Stats};
use crate::presentation::{print_header, OutputFormat, output_item};
use crate::storage;
use owo_colors::OwoColorize;
use std::collections::HashMap;

pub fn handle_stats(format: OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let total_contacts = store.entries.len();
    let mut by_relationship: HashMap<String, usize> = HashMap::new();
    let mut by_tag: HashMap<String, usize> = HashMap::new();
    let mut recent_contacts: Vec<ContactFrequency> = Vec::new();
    let mut needs_reminder: Vec<String> = Vec::new();

    for contact in store.entries.values() {
        if !contact.relationship.is_empty() {
            *by_relationship.entry(contact.relationship.clone()).or_insert(0) += 1;
        }

        for tag in &contact.tags {
            *by_tag.entry(tag.clone()).or_insert(0) += 1;
        }

        let days_since = contact.days_since_last_contact().unwrap_or(i64::MAX);
        
        recent_contacts.push(ContactFrequency {
            name: contact.name.clone(),
            days_since_contact: days_since,
            contact_count: contact.contact_count,
        });

        if days_since > 30 {
            needs_reminder.push(contact.name.clone());
        }
    }

    recent_contacts.sort_by(|a, b| a.days_since_contact.cmp(&b.days_since_contact).reverse());

    let stats = Stats {
        total_contacts,
        by_relationship,
        by_tag,
        recent_contacts,
        needs_reminder,
    };

    if format == OutputFormat::Json {
        output_item(&stats, format);
    } else {
        print_header("Contact Statistics");
        
        println!("\n{} {}", "Total Contacts:".bold().cyan(), total_contacts);

        if !stats.by_relationship.is_empty() {
            println!("\n{}", "By Relationship:".bold().cyan());
            let mut relationships: Vec<_> = stats.by_relationship.iter().collect();
            relationships.sort_by(|a, b| b.1.cmp(a.1));
            for (rel, count) in relationships {
                println!("  {}: {}", rel, count);
            }
        }

        if !stats.by_tag.is_empty() {
            println!("\n{}", "By Tag:".bold().cyan());
            let mut tags: Vec<_> = stats.by_tag.iter().collect();
            tags.sort_by(|a, b| b.1.cmp(a.1));
            for (tag, count) in tags {
                println!("  {}: {}", tag, count);
            }
        }

        if !stats.needs_reminder.is_empty() {
            println!("\n{} {}", "Needs Reminder (30+ days):".bold().yellow(), stats.needs_reminder.len());
            for name in &stats.needs_reminder {
                println!("  - {}", name);
            }
        }
    }

    Ok(())
}
