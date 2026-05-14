use crate::presentation::print_header;
use owo_colors::OwoColorize;

pub fn handle_example() {
    print_header("i-rs-project Examples");
    
    println!();
    println!("{}", "Add a new project:".bold().cyan());
    println!("  i-rs-project add my-project -d \"Project description\"");
    println!("  i-rs-project add api-redesign --description \"API redesign\" --priority high --tag work");
    
    println!();
    println!("{}", "List projects:".bold().cyan());
    println!("  i-rs-project list");
    println!("  i-rs-project list --tag work");
    println!("  i-rs-project list --status active");
    
    println!();
    println!("{}", "Get project details:".bold().cyan());
    println!("  i-rs-project get my-project");
    
    println!();
    println!("{}", "Update project:".bold().cyan());
    println!("  i-rs-project update my-project --status completed");
    println!("  i-rs-project update my-project --priority urgent --tag important");
    
    println!();
    println!("{}", "Delete project:".bold().cyan());
    println!("  i-rs-project delete my-project");
    
    println!();
    println!("{}", "Manage milestones:".bold().cyan());
    println!("  i-rs-project milestone add my-project \"v1.0\" --due-date 2026-06-01");
    println!("  i-rs-project milestone add my-project \"Beta Release\" --description \"Public beta\"");
    println!("  i-rs-project milestone complete my-project \"v1.0\"");
    
    println!();
    println!("{}", "Manage tasks:".bold().cyan());
    println!("  i-rs-project task add my-project \"Write docs\" --tag docs");
    println!("  i-rs-project task add my-project \"Review PR\" --description \"Review pull request\"");
    println!("  i-rs-project task complete my-project \"Write docs\"");
    
    println!();
    println!("{}", "Show statistics:".bold().cyan());
    println!("  i-rs-project stats");
    
    println!();
    println!("{}", "JSON output:".bold().cyan());
    println!("  i-rs-project list --json");
    println!("  i-rs-project get my-project --json");
}
