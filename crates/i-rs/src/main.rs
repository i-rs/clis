use std::process::{Command, ExitCode};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    // i-rs → show tool list
    if args.len() == 1 {
        print_tools();
        return ExitCode::SUCCESS;
    }

    let first = &args[1];

    // i-rs --help / -h → show tool list
    if first == "--help" || first == "-h" {
        print_usage();
        return ExitCode::SUCCESS;
    }

    // i-rs --version / -V → show version
    if first == "--version" || first == "-V" {
        println!("i-rs {VERSION}");
        return ExitCode::SUCCESS;
    }

    // i-rs --<flag> → unknown flag error
    if first.starts_with('-') {
        eprintln!("error: unknown flag '{first}'");
        eprintln!("Usage: i-rs <TOOL> [ARGS]...");
        eprintln!("Run `i-rs --help` for available tools.");
        return ExitCode::FAILURE;
    }

    // i-rs <tool> [args...] → dispatch
    let tool = first;
    let tool_args: Vec<&str> = args[2..].iter().map(String::as_str).collect();
    let binary = format!("i-rs-{tool}");

    match Command::new(&binary).args(&tool_args).status() {
        Ok(status) => {
            if status.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(status.code().unwrap_or(1) as u8)
            }
        }
        Err(_) => {
            eprintln!("error: '{binary}' is not installed");
            eprintln!();
            eprintln!("  Install with cargo:");
            eprintln!("    cargo install -p {binary}");
            eprintln!();
            eprintln!("  Or with npm:");
            eprintln!("    npm install -g @i-rs/i-rs-{tool}");
            eprintln!();
            eprintln!("  Or build from source:");
            eprintln!("    git clone https://github.com/i-rs/clis.git");
            eprintln!("    cargo build -p {binary}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    println!("i-rs v{VERSION} — Unified CLI for all i-rs tools");
    println!();
    println!("Usage: i-rs <TOOL> [ARGS]...");
    println!();
    println!("Run any i-rs tracking tool as a subcommand.");
    println!("Each tool can also be used directly (e.g., i-rs-todo, i-rs-mood).");
    println!();
    println!("Examples:");
    println!("  i-rs todo list");
    println!("  i-rs mood add today happy");
    println!("  i-rs weight add 75");
    println!("  i-rs todo --help");
    println!();
    println!("Available tools:");
    print_tools();
    println!();
    println!("For tool-specific help: i-rs <TOOL> --help");
}

fn print_tools() {
    const TOOLS: &[(&str, &str)] = &[
        // Core & System
        ("server", "Server management"),
        ("password", "Password management"),
        ("keys", "API key management"),
        ("kv", "Key-value storage"),
        ("deploy", "Deployment tracking"),
        // Notes & Bookmarks
        ("note", "Note management"),
        ("bookmark", "Bookmark management"),
        ("article", "Article read-later"),
        ("read", "Reading progress tracking"),
        ("quote", "Quote collection"),
        ("spark", "Inspiration capture"),
        ("snippet", "Code snippet management"),
        ("vocab", "Vocabulary learning"),
        // Health & Fitness
        ("weight", "Weight tracking"),
        ("height", "Height tracking"),
        ("mood", "Mood tracking"),
        ("sleep", "Sleep tracking"),
        ("step", "Step counting"),
        ("water", "Water intake tracking"),
        ("cal", "Calorie tracking"),
        ("fast", "Fasting tracking"),
        ("exercise", "Exercise tracking"),
        ("run", "Running records"),
        ("cycling", "Cycling tracking"),
        ("dose", "Medicine dosage"),
        ("allergy", "Allergy tracking"),
        ("sit", "Sedentary reminder"),
        ("vision", "Vision tracking"),
        // Finance
        ("ledger", "Personal accounting"),
        ("budget", "Budget management"),
        ("recur", "Recurring expenses"),
        ("sub", "Subscription tracking"),
        ("invest", "Investment tracking"),
        ("debt", "Debt management"),
        ("invoice", "Invoice management"),
        ("tax", "Tax records"),
        ("goal", "Savings goals"),
        // Reminders & Expiry
        ("remind", "Event reminders"),
        ("domain", "Domain expiry tracking"),
        ("bestby", "Best-by date tracking"),
        ("tick", "Duration tracking"),
        ("time", "Time tracking"),
        ("event", "Event management"),
        ("birthday", "Birthday reminders"),
        // Home & Care
        ("sheet", "Bedsheet replacement"),
        ("toothbrush", "Toothbrush replacement"),
        ("towel", "Towel replacement"),
        ("bed", "Mattress/pillow replacement"),
        ("ac", "AC cleaning"),
        ("filter", "Filter cleaning"),
        ("purify", "Water purifier filter"),
        ("appliance", "Appliance lifecycle"),
        ("plant", "Plant care"),
        // Pets
        ("feedpet", "Pet feeding"),
        ("petbath", "Pet bathing"),
        ("walkdog", "Dog walking"),
        ("aqua", "Aquarium maintenance"),
        // Productivity & Tracking
        ("todo", "Todo tracking"),
        ("habit", "Habit tracking"),
        ("project", "Project management"),
        ("want", "Wish list"),
        ("gift", "Gift planning"),
        ("car", "Vehicle management"),
        ("meal", "Meal tracking"),
        ("pig", "Craving tracking"),
        ("grocery", "Grocery list"),
        ("contact", "Contact management"),
        ("movie", "Movie tracking"),
        ("podcast", "Podcast tracking"),
        ("cycle", "Menstrual cycle"),
    ];

    let pad = TOOLS.iter().map(|(n, _)| n.len()).max().unwrap_or(12);
    for (name, desc) in TOOLS {
        println!("  {name:>pad$}   {desc}");
    }
}
