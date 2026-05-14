use crate::models::{CarDetail, FuelRecord};
use crate::presentation::{format_car_detail, format_fuel_table, format_maintenance_table, output_item, print_fuel_count, print_maintenance_count, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Car name")]
    pub name: String,
    #[arg(short, long, help = "Show fuel records")]
    pub fuel: bool,
    #[arg(short, long, help = "Show maintenance records")]
    pub maintain: bool,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let car = match store.get_car(&args.name) {
        Some(c) => c,
        None => {
            anyhow::bail!("Car '{}' not found", args.name);
        }
    };

    if args.fuel {
        let fuel_records: Vec<&FuelRecord> = store.get_fuel_records(Some(&args.name));
        
        let records_with_prev: Vec<(&FuelRecord, Option<f64>)> = fuel_records.iter().map(|r| {
            let prev_mileage = store.get_last_fuel_record(&args.name)
                .and_then(|prev| if prev.date < r.date { Some(prev.mileage) } else { None });
            (*r, prev_mileage)
        }).collect();

        if output_format == OutputFormat::Json {
            let data: Vec<_> = records_with_prev.iter().map(|(r, prev)| {
                let efficiency = if let Some(p) = prev {
                    r.fuel_efficiency(*p).map(|e| format!("{e:.1}"))
                } else {
                    None
                };
                serde_json::json!({
                    "id": r.id,
                    "date": r.date.format("%Y-%m-%d").to_string(),
                    "mileage": r.mileage,
                    "fuel_amount": r.fuel_amount,
                    "price_per_liter": r.price_per_liter,
                    "total_cost": r.total_cost,
                    "efficiency": efficiency,
                    "station": r.station,
                    "note": r.note
                })
            }).collect();
            println!("{}", output_item(&data, output_format));
        } else {
            let table = format_fuel_table(&records_with_prev);
            if !table.is_empty() {
                println!("{table}");
            }
            print_fuel_count(fuel_records.len());
        }
    } else if args.maintain {
        let maintenance_records: Vec<_> = store.get_maintenance_records(Some(&args.name)).into_iter().collect();

        if output_format == OutputFormat::Json {
            let data: Vec<_> = maintenance_records.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "date": r.date.format("%Y-%m-%d").to_string(),
                    "mileage": r.mileage,
                    "maintenance_type": r.maintenance_type,
                    "cost": r.cost,
                    "description": r.description,
                    "shop": r.shop,
                    "note": r.note
                })
            }).collect();
            println!("{}", output_item(&data, output_format));
        } else {
            let table = format_maintenance_table(&maintenance_records);
            if !table.is_empty() {
                println!("{table}");
            }
            print_maintenance_count(maintenance_records.len());
        }
    } else {
        let fuel_count = store.get_fuel_records(Some(&args.name)).len();
        let maintenance_count = store.get_maintenance_records(Some(&args.name)).len();
        let total_fuel_cost = store.total_fuel_cost(Some(&args.name));
        let total_maintenance_cost = store.total_maintenance_cost(Some(&args.name));

        let detail = CarDetail {
            name: car.name.clone(),
            license_plate: car.license_plate.clone(),
            brand: car.brand.clone(),
            model: car.model.clone(),
            mileage: car.mileage,
            fuel_count,
            maintenance_count,
            total_fuel_cost,
            total_maintenance_cost,
            tags: car.tags.clone(),
            remark: car.remark.clone(),
            created_at: car.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: car.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        if output_format == OutputFormat::Json {
            println!("{}", output_item(&detail, output_format));
        } else {
            println!("{}", format_car_detail(&detail));
        }
    }

    Ok(())
}
