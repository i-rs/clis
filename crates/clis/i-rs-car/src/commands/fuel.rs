use crate::models::FuelRecord;
use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Car name")]
    pub car: String,
    #[arg(short, long, help = "Fuel date (YYYY-MM-DD, default: today)")]
    pub date: Option<String>,
    #[arg(short, long, help = "Current mileage (km)")]
    pub mileage: f64,
    #[arg(short, long, help = "Fuel amount (liters)")]
    pub fuel_amount: f64,
    #[arg(short, long, help = "Price per liter")]
    pub price: f64,
    #[arg(short, long, help = "Fuel type (e.g., 92, 95, 98, diesel)")]
    pub fuel_type: Option<String>,
    #[arg(short, long, help = "Gas station name")]
    pub station: Option<String>,
    #[arg(short, long, help = "Note")]
    pub note: Option<String>,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    if !store.cars.contains_key(&args.car) {
        anyhow::bail!("Car '{}' not found", args.car);
    }

    let date = match &args.date {
        Some(d) => NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid date format, use YYYY-MM-DD"))?,
        None => chrono::Utc::now().date_naive(),
    };

    if let Some(car) = store.get_entry(&args.car)
        && args.mileage < car.mileage
    {
        anyhow::bail!("Fuel record mileage cannot be less than car's current mileage");
    }

    let record = FuelRecord::new(
        args.car.clone(),
        date,
        args.mileage,
        args.fuel_amount,
        args.price,
        args.fuel_type.clone(),
        args.station.clone(),
        args.note.clone(),
    );

    store.add_fuel_record(record);
    storage::save_store(&store)?;

    if output_format == OutputFormat::Json {
        println!(
            "{{\"success\": true, \"data\": {{\"car\": \"{}\", \"cost\": {:.2}}}}}",
            args.car,
            args.fuel_amount * args.price
        );
    } else {
        print_success(&format!(
            "Fuel record added for car '{}' (cost: {:.2})",
            args.car,
            args.fuel_amount * args.price
        ));
    }

    Ok(())
}
