use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-podcast Examples".bold().cyan());
    println!();

    println!("{}", "Add Podcast:".bold().green());
    println!("  i-rs-podcast add \"The Daily\" --author \"New York Times\" --duration 3600");
    println!("  i-rs-podcast add \"Rust Course\" --author \"Ferris\" --duration 7200 --tag rust --tag programming");
    println!();

    println!("{}", "List Podcasts:".bold().green());
    println!("  i-rs-podcast list");
    println!("  i-rs-podcast list --status not_started");
    println!("  i-rs-podcast list --status in_progress");
    println!("  i-rs-podcast list --status completed");
    println!("  i-rs-podcast list --tag programming");
    println!();

    println!("{}", "Get Podcast Details:".bold().green());
    println!("  i-rs-podcast get \"The Daily\"");
    println!();

    println!("{}", "Update Progress (listen):".bold().green());
    println!("  i-rs-podcast listen \"The Daily\" --position 1800");
    println!("  i-rs-podcast listen \"Rust Course\" --position 3600 --notes \"Key concept: ownership\"");
    println!();

    println!("{}", "Update Podcast:".bold().green());
    println!("  i-rs-podcast update \"The Daily\" --author \"NYT\"");
    println!("  i-rs-podcast update \"Rust Course\" --tag rust --tag tutorial");
    println!();

    println!("{}", "Delete Podcast:".bold().green());
    println!("  i-rs-podcast delete \"The Daily\"");
    println!();

    println!("{}", "Statistics:".bold().green());
    println!("  i-rs-podcast stats");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-podcast list --json");
    println!("  i-rs-podcast get \"The Daily\" --json");
    println!();

    println!("{}", "Options:".bold().yellow());
    println!("  --author, -a:       Author/host name");
    println!("  --duration, -d:    Total duration in seconds");
    println!("  --position, -p:    Current position in seconds (listen command)");
    println!("  --status, -s:      Status filter (not_started, in_progress, completed)");
    println!("  --tag, -t:         Tags (repeatable)");
    println!("  --remark:          Remark lines (repeatable)");
    println!("  --notes:           Note lines (repeatable)");
    println!();

    println!("{}", "Status Icons:".bold().yellow());
    println!("  ○  Not Started");
    println!("  ◐  In Progress");
    println!("  ●  Completed");
    println!();
}
