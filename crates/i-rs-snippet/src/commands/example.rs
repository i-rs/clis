use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-snippet Examples".bold().cyan());
    println!();

    println!("{}", "Add Snippet:".bold().green());
    println!(
        "  i-rs-snippet add hello --language rust --code 'fn main() {{ println!(\"Hello!\"); }}' --tag rust --tag hello"
    );
    println!(
        "  i-rs-snippet add py-hello --language python --code 'print(\"Hello\")' --tag python"
    );
    println!(
        "  i-rs-snippet add func --language js --code 'function test() {{}}' --description 'Test function' --tag js"
    );
    println!();

    println!("{}", "List Snippets:".bold().green());
    println!("  i-rs-snippet list");
    println!("  i-rs-snippet list --tag rust");
    println!();

    println!("{}", "Search Snippets:".bold().green());
    println!("  i-rs-snippet search hello");
    println!("  i-rs-snippet search print");
    println!();

    println!("{}", "Get Snippet:".bold().green());
    println!("  i-rs-snippet get hello");
    println!();

    println!("{}", "Copy to Clipboard:".bold().green());
    println!("  i-rs-snippet copy hello");
    println!();

    println!("{}", "Update Snippet:".bold().green());
    println!("  i-rs-snippet update hello --code 'println!(\"Updated!\");' --tag updated");
    println!();

    println!("{}", "Delete Snippet:".bold().green());
    println!("  i-rs-snippet delete hello");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-snippet list --json");
    println!("  i-rs-snippet get hello --json");
    println!("  i-rs-snippet search hello --json");
    println!();
}
