use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-exercise Examples".bold().cyan());
    println!();

    println!("{}", "Add Exercise:".bold().green());
    println!("  i-rs-exercise add \"Morning Run\" running 30 200 --tags cardio --notes \"Park trail\"");
    println!("  i-rs-exercise add \"Gym Workout\" strength 60 300 --tags gym --tags strength");
    println!("  i-rs-exercise add \"Swimming\" swimming 45 400");
    println!();

    println!("{}", "List Exercises:".bold().green());
    println!("  i-rs-exercise list");
    println!("  i-rs-exercise list --tag cardio");
    println!("  i-rs-exercise list --type running");
    println!();

    println!("{}", "Get Exercise Details:".bold().green());
    println!("  i-rs-exercise get \"Morning Run\"");
    println!();

    println!("{}", "Update Exercise:".bold().green());
    println!("  i-rs-exercise update \"Morning Run\" --duration 45 --calories 250");
    println!("  i-rs-exercise update \"Morning Run\" --notes \"Updated notes\"");
    println!();

    println!("{}", "Delete Exercise:".bold().green());
    println!("  i-rs-exercise delete \"Morning Run\"");
    println!();

    println!("{}", "View Statistics:".bold().green());
    println!("  i-rs-exercise stats");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-exercise list --json");
    println!("  i-rs-exercise get \"Morning Run\" --json");
    println!("  i-rs-exercise stats --json");
    println!();
}
