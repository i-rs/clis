use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-contact Examples".bold().cyan());
    println!();

    println!("{}", "Add a contact:".bold().green());
    println!(
        "  i-rs-contact add John --phone 13800138000 --email john@example.com --relationship friend"
    );
    println!("  i-rs-contact add Alice --phone 13900139000 --tag family --tag important");
    println!();

    println!("{}", "List contacts:".bold().green());
    println!("  i-rs-contact list");
    println!("  i-rs-contact list --tag family");
    println!();

    println!("{}", "Get contact details:".bold().green());
    println!("  i-rs-contact get John");
    println!();

    println!("{}", "Update contact:".bold().green());
    println!("  i-rs-contact update John --phone 13800138001 --relationship colleague");
    println!();

    println!("{}", "Delete contact:".bold().green());
    println!("  i-rs-contact delete old_contact");
    println!();

    println!("{}", "Contact statistics:".bold().green());
    println!("  i-rs-contact stats");
    println!();

    println!("{}", "Remind to contact:".bold().green());
    println!("  i-rs-contact remind");
    println!("  i-rs-contact remind --days 7");
    println!();

    println!("{}", "Relationships:".bold().yellow());
    println!("  family, friend, colleague, client, other");
    println!();
}
