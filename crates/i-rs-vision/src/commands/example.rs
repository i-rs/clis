use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-vision Examples".bold().cyan());
    println!();

    println!("{}", "Add Vision Record:".bold().green());
    println!("  i-rs-vision add 2025-06-14 --left-sphere -3.50 --right-sphere -4.00");
    println!("  i-rs-vision add 2025-06-14 -ls -3.50 -rs -4.00 -lc -0.50 -rc -0.75 -la 180 -ra 5");
    println!("  i-rs-vision add 2025-06-14 -ls -3.50 -rs -4.00 --tag myopia --remark \"Annual checkup\"");
    println!();

    println!("{}", "List Records:".bold().green());
    println!("  i-rs-vision list");
    println!("  i-rs-vision list --days 365");
    println!();

    println!("{}", "Get Record:".bold().green());
    println!("  i-rs-vision get 2025-06-14");
    println!();

    println!("{}", "Delete Record:".bold().green());
    println!("  i-rs-vision delete 2025-06-14");
    println!();

    println!("{}", "Statistics:".bold().green());
    println!("  i-rs-vision stats");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-vision list --json");
    println!("  i-rs-vision get 2025-06-14 --json");
    println!("  i-rs-vision stats --json");
    println!();

    println!("{}", "Options:".bold().yellow());
    println!("  -ls, --left-sphere     Left eye sphere (D)");
    println!("  -rs, --right-sphere    Right eye sphere (D)");
    println!("  -lc, --left-cylinder   Left eye cylinder/astigmatism (D)");
    println!("  -rc, --right-cylinder  Right eye cylinder/astigmatism (D)");
    println!("  -la, --left-axis       Left eye axis (degrees)");
    println!("  -ra, --right-axis      Right eye axis (degrees)");
    println!("  -t, --tag              Tags (repeatable)");
    println!("  -r, --remark           Remarks (repeatable)");
    println!();

    println!("{}", "Notes:".bold().yellow());
    println!("  - Sphere values: negative = myopia (nearsighted), positive = hyperopia (farsighted)");
    println!("  - Cylinder values: astigmatism correction, typically 0 to -2.00");
    println!("  - Axis values: 0-180 degrees for astigmatism orientation");
}
