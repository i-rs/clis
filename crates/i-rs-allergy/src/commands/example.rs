use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-allergy Examples".bold().cyan());
    println!();

    println!("{}", "Record allergy reaction:".bold().green());
    println!("  i-rs-allergy add \"Peanuts\" mild --symptom \"hives\" --symptom \"itching\"");
    println!(
        "  i-rs-allergy add \"Pollen\" severe --symptom \"sneezing\" --symptom \"watery eyes\""
    );
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-allergy list");
    println!("  i-rs-allergy list --tag food");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-allergy get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-allergy delete abc12345");
    println!();
}
