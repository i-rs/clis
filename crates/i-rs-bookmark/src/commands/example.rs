use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-bookmark Examples".bold().cyan());
    println!();

    println!("{}", "Add Bookmark:".bold().green());
    println!("  i-rs-bookmark add github https://github.com --account user@example.com --tag code --tag work");
    println!("  i-rs-bookmark add twitter https://twitter.com --tag social");
    println!();

    println!("{}", "List Bookmarks:".bold().green());
    println!("  i-rs-bookmark list");
    println!("  i-rs-bookmark list --tag work");
    println!();

    println!("{}", "Get Bookmark:".bold().green());
    println!("  i-rs-bookmark get github");
    println!("  i-rs-bookmark get github --show-password");
    println!();

    println!("{}", "Update Bookmark:".bold().green());
    println!("  i-rs-bookmark update github --remark \"My GitHub account\"");
    println!("  i-rs-bookmark update twitter --tag social --tag personal");
    println!();

    println!("{}", "Delete Bookmark:".bold().green());
    println!("  i-rs-bookmark delete twitter");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-bookmark list --json");
    println!("  i-rs-bookmark get github --json");
    println!();
}