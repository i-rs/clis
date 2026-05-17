use std::process::{Command, ExitCode};

include!(concat!(env!("OUT_DIR"), "/tools_gen.rs"));

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
    let tools = generated_tools();

    let pad = tools.iter().map(|(n, _)| n.len()).max().unwrap_or(12);
    for (name, desc) in tools {
        println!("  {name:>pad$}   {desc}");
    }
}
