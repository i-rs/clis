use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-bestby Examples".bold().cyan());
    println!();

    println!("{}", "Add Item:".bold().green());
    println!(
        "  i-rs-bestby add \"Toothbrush\" 2024-01-15 --tag bathroom --remark \"Electric toothbrush\""
    );
    println!("  i-rs-bestby add \"Pillow\" 2023-06-01 --tag bedroom");
    println!();

    println!("{}", "Set Replacement Cycle:".bold().green());
    println!("  i-rs-bestby update \"Toothbrush\" --cycle-days 90");
    println!("  i-rs-bestby update \"Pillow\" --cycle-days 365");
    println!();

    println!("{}", "List Items:".bold().green());
    println!("  i-rs-bestby list");
    println!("  i-rs-bestby list --tag bathroom");
    println!();

    println!("{}", "Get Item:".bold().green());
    println!("  i-rs-bestby get \"Toothbrush\"");
    println!();

    println!("{}", "Update Item:".bold().green());
    println!("  i-rs-bestby update \"Toothbrush\" --cycle-days 60 --remark \"New brush head\"");
    println!();

    println!("{}", "Delete Item:".bold().green());
    println!("  i-rs-bestby delete \"Old Item\"");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-bestby list --json");
    println!("  i-rs-bestby get \"Toothbrush\" --json");
    println!();

    println!("{}", "Status:".bold().yellow());
    println!("  EXPIRED - Item past replacement date");
    println!("  SOON    - Within 7 days of replacement");
    println!("  OK      - Replacement not yet needed");
    println!("  NO_CYCLE - No cycle set (use update to set)");
    println!();
}
