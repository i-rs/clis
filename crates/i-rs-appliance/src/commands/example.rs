use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-appliance Examples".bold().cyan());
    println!();

    println!("{}", "Add Appliance:".bold().green());
    println!(
        "  i-rs-appliance add \"Refrigerator\" Samsung \"RF28R7551\" 2020-01-15 10 --tag kitchen"
    );
    println!("  i-rs-appliance add \"Washing Machine\" LG \"WM4000\" 2021-06-20 8 --tag laundry");
    println!();

    println!("{}", "List Appliances:".bold().green());
    println!("  i-rs-appliance list");
    println!("  i-rs-appliance list --tag kitchen");
    println!();

    println!("{}", "Get Appliance Details:".bold().green());
    println!("  i-rs-appliance get \"Refrigerator\"");
    println!();

    println!("{}", "Update Appliance:".bold().green());
    println!("  i-rs-appliance update \"Refrigerator\" --lifespan 12");
    println!("  i-rs-appliance update \"Refrigerator\" --add-maintenance \"Cleaned coils\"");
    println!();

    println!("{}", "Add Maintenance Record:".bold().green());
    println!(
        "  i-rs-appliance update \"Refrigerator\" --add-maintenance \"Replaced water filter\" --maintenance-date 2024-03-01"
    );
    println!();

    println!("{}", "Delete Appliance:".bold().green());
    println!("  i-rs-appliance delete \"Old Microwave\"");
    println!();

    println!("{}", "Statistics:".bold().green());
    println!("  i-rs-appliance stats");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-appliance list --json");
    println!("  i-rs-appliance get \"Refrigerator\" --json");
    println!();
}
