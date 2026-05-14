use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-article Examples".bold().cyan());
    println!();

    println!("{}", "Add Article:".bold().green());
    println!("  i-rs-article add rust-blog https://blog.rust-lang.org \"Rust Blog\" --source rust --tag programming --tag tutorial");
    println!("  i-rs-article add python-article https://python.org \"Python Guide\" --source python.org --tag python");
    println!();

    println!("{}", "List Articles:".bold().green());
    println!("  i-rs-article list");
    println!("  i-rs-article list --tag programming");
    println!("  i-rs-article list --status unread");
    println!("  i-rs-article list --tag rust --status unread");
    println!();

    println!("{}", "Get Article:".bold().green());
    println!("  i-rs-article get rust-blog");
    println!();

    println!("{}", "Read Article:".bold().green());
    println!("  i-rs-article read rust-blog");
    println!();

    println!("{}", "Update Article:".bold().green());
    println!("  i-rs-article update rust-blog --remark \"Important article\"");
    println!("  i-rs-article update rust-blog --tag rust --tag updated");
    println!("  i-rs-article update rust-blog --status reading");
    println!("  i-rs-article update rust-blog --notes \"Key point 1\" \"Key point 2\"");
    println!();

    println!("{}", "Delete Article:".bold().green());
    println!("  i-rs-article delete python-article");
    println!();

    println!("{}", "Stats:".bold().green());
    println!("  i-rs-article stats");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-article list --json");
    println!("  i-rs-article get rust-blog --json");
    println!();
}
