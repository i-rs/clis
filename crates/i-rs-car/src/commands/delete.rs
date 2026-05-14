use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Car name")]
    pub name: String,
    #[arg(short, long, help = "Skip confirmation")]
    pub force: bool,
}

pub fn run(args: &Args) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.cars.contains_key(&args.name) {
        anyhow::bail!("Car '{}' not found", args.name);
    }

    if !args.force {
        print!("Are you sure you want to delete car '{}'? (y/N) ", args.name);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    store.delete_car(&args.name);
    store.fuel_records.retain(|r| r.car_name != args.name);
    store.maintenance_records.retain(|r| r.car_name != args.name);
    storage::save_store(&store)?;

    print_success(&format!("Car '{}' deleted successfully", args.name));

    Ok(())
}
