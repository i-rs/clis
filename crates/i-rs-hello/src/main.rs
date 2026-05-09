use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "i-rs-hello")]
#[command(about = "A simple hello world CLI", long_about = None)]
struct Args {
    #[arg(short, long)]
    name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let name = args.name.unwrap_or_else(|| "World".to_string());
    println!("Hello, {}!", name);

    Ok(())
}
