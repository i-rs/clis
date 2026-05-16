use crate::presentation::print_header;
use crate::storage;
use chrono::Utc;
use owo_colors::OwoColorize;

pub fn handle_remind(days: Option<i64>) -> anyhow::Result<()> {
    let store = storage::load_store()?;
    let threshold = days.unwrap_or(30);
    let now = Utc::now();

    let mut needs_contact: Vec<(&String, i64)> = Vec::new();

    for (name, contact) in &store.entries {
        if let Some(last) = contact.last_contact {
            let days_since = (now - last).num_days();
            if days_since >= threshold {
                needs_contact.push((name, days_since));
            }
        } else {
            needs_contact.push((name, i64::MAX));
        }
    }

    needs_contact.sort_by_key(|a| std::cmp::Reverse(a.1));

    if needs_contact.is_empty() {
        println!("{}", "All contacts are up to date!".green());
        return Ok(());
    }

    print_header(&format!("Contacts to Reach Out ({threshold} days+)"));
    println!();

    for (name, days_since) in &needs_contact {
        let days_str = if *days_since == i64::MAX {
            "Never".to_string()
        } else {
            format!("{days_since} days")
        };

        let contact = store
            .get_entry(name)
            .expect("entry comes from store keys, must exist");

        println!(
            "{} {}",
            name.bold().cyan(),
            format!("({days_str})").dimmed()
        );

        if !contact.phone.is_empty() {
            println!("  Phone: {}", contact.phone);
        }
        if !contact.email.is_empty() {
            println!("  Email: {}", contact.email);
        }

        if !contact.relationship.is_empty() {
            println!("  Relationship: {}", contact.relationship);
        }

        println!();
    }

    let total = needs_contact.len();
    println!(
        "{} {} contacts need attention",
        "Total:".dimmed(),
        total.to_string().yellow()
    );

    Ok(())
}
