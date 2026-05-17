use crate::models::MaintenanceRecord;
use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Car name")]
    pub car: String,
    #[arg(short, long, help = "Maintenance date (YYYY-MM-DD, default: today)")]
    pub date: Option<String>,
    #[arg(short, long, help = "Current mileage (km)")]
    pub mileage: f64,
    #[arg(
        short,
        long,
        help = "Maintenance type (e.g., oil_change, tire, brake, inspection)"
    )]
    pub maintenance_type: String,
    #[arg(short, long, help = "Maintenance cost")]
    pub cost: f64,
    #[arg(short, long, help = "Description")]
    pub description: Option<String>,
    #[arg(short, long, help = "Shop name")]
    pub shop: Option<String>,
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
        anyhow::bail!("Maintenance record mileage cannot be less than car's current mileage");
    }

    let record = MaintenanceRecord::new(
        args.car.clone(),
        date,
        args.mileage,
        args.maintenance_type.clone(),
        args.cost,
        args.description.clone(),
        args.shop.clone(),
        args.note.clone(),
    );

    store.add_maintenance_record(record);
    storage::save_store(&store)?;

    if output_format == OutputFormat::Json {
        println!(
            "{{\"success\": true, \"data\": {{\"car\": \"{}\", \"type\": \"{}\", \"cost\": {:.2}}}}}",
            args.car, args.maintenance_type, args.cost
        );
    } else {
        print_success(&format!(
            "Maintenance record added for car '{}' ({}: {:.2})",
            args.car, args.maintenance_type, args.cost
        ));
    }

    Ok(())
}
