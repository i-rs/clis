use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-cycling Examples".bold().cyan());
    println!();

    println!("{}", "Add Cycling Record:".bold().green());
    println!("  i-rs-cycling add 2025-06-14 25.5 60 --elevation 300");
    println!("  i-rs-cycling add 2025-06-15 30.2 75 --route \"Mountain Trail\" --tag mountain");
    println!("  i-rs-cycling add 2025-06-16 15.0 30 --remark \"Morning ride\"");
    println!();

    println!("{}", "List Records:".bold().green());
    println!("  i-rs-cycling list");
    println!("  i-rs-cycling list --tag mountain");
    println!();

    println!("{}", "View Record Details:".bold().green());
    println!("  i-rs-cycling get <uuid>");
    println!("  i-rs-cycling get 2025-06-14");
    println!();

    println!("{}", "Update Record:".bold().green());
    println!("  i-rs-cycling update <uuid> --distance 26.0");
    println!("  i-rs-cycling update 2025-06-14 --add-tag favorite");
    println!("  i-rs-cycling update <uuid> --route \"New Route\"");
    println!();

    println!("{}", "Delete Record:".bold().green());
    println!("  i-rs-cycling delete <uuid>");
    println!("  i-rs-cycling delete 2025-06-14");
    println!();

    println!("{}", "View Statistics:".bold().green());
    println!("  i-rs-cycling stats");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-cycling list --json");
    println!("  i-rs-cycling get <uuid> --json");
    println!();
}
