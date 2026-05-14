use crate::models::{CarStats, Stats};
use crate::presentation::{format_stats, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, help = "Show statistics for specific car")]
    pub car: Option<String>,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if let Some(car_name) = &args.car {
        if !store.cars.contains_key(car_name) {
            anyhow::bail!("Car '{}' not found", car_name);
        }

        let fuel_count = store.get_fuel_records(Some(car_name)).len();
        let maintenance_count = store.get_maintenance_records(Some(car_name)).len();
        let total_fuel_cost = store.total_fuel_cost(Some(car_name));
        let total_maintenance_cost = store.total_maintenance_cost(Some(car_name));
        let latest_mileage = store.get_car(car_name).map(|c| c.mileage).unwrap_or(0.0);

        let car_stats = CarStats {
            fuel_count,
            maintenance_count,
            total_fuel_cost,
            total_maintenance_cost,
            latest_mileage,
        };

        if output_format == OutputFormat::Json {
            println!("{}", output_item(&serde_json::json!({
                "car": car_name,
                "fuel_count": car_stats.fuel_count,
                "maintenance_count": car_stats.maintenance_count,
                "total_fuel_cost": car_stats.total_fuel_cost,
                "total_maintenance_cost": car_stats.total_maintenance_cost,
                "total_cost": car_stats.total_fuel_cost + car_stats.total_maintenance_cost,
                "latest_mileage": car_stats.latest_mileage
            }), output_format));
        } else {
            println!("=== {} Statistics ===", car_name);
            println!("Mileage: {:.0} km", car_stats.latest_mileage);
            println!("Fuel Records: {}", car_stats.fuel_count);
            println!("Total Fuel Cost: {:.2}", car_stats.total_fuel_cost);
            println!("Maintenance Records: {}", car_stats.maintenance_count);
            println!("Total Maintenance Cost: {:.2}", car_stats.total_maintenance_cost);
            println!("Total Cost: {:.2}", car_stats.total_fuel_cost + car_stats.total_maintenance_cost);
        }

        return Ok(());
    }

    let total_fuel_cost = store.total_fuel_cost(None);
    let total_maintenance_cost = store.total_maintenance_cost(None);

    let mut by_car: HashMap<String, CarStats> = HashMap::new();

    for car in store.list_cars() {
        let car_name = &car.name;
        let fuel_records = store.get_fuel_records(Some(car_name));
        let maintenance_records = store.get_maintenance_records(Some(car_name));

        by_car.insert(car_name.clone(), CarStats {
            fuel_count: fuel_records.len(),
            maintenance_count: maintenance_records.len(),
            total_fuel_cost: store.total_fuel_cost(Some(car_name)),
            total_maintenance_cost: store.total_maintenance_cost(Some(car_name)),
            latest_mileage: car.mileage,
        });
    }

    let stats = Stats {
        total_cars: store.cars.len(),
        total_fuel_records: store.fuel_records.len(),
        total_maintenance_records: store.maintenance_records.len(),
        total_fuel_cost,
        total_maintenance_cost,
        by_car,
    };

    if output_format == OutputFormat::Json {
        let by_car_json: HashMap<String, serde_json::Value> = stats.by_car
            .iter()
            .map(|(k, v)| {
                (k.clone(), serde_json::json!({
                    "fuel_count": v.fuel_count,
                    "maintenance_count": v.maintenance_count,
                    "total_fuel_cost": v.total_fuel_cost,
                    "total_maintenance_cost": v.total_maintenance_cost,
                    "total_cost": v.total_fuel_cost + v.total_maintenance_cost,
                    "latest_mileage": v.latest_mileage
                }))
            })
            .collect();

        println!("{}", output_item(&serde_json::json!({
            "total_cars": stats.total_cars,
            "total_fuel_records": stats.total_fuel_records,
            "total_maintenance_records": stats.total_maintenance_records,
            "total_fuel_cost": stats.total_fuel_cost,
            "total_maintenance_cost": stats.total_maintenance_cost,
            "total_cost": stats.total_fuel_cost + stats.total_maintenance_cost,
            "by_car": by_car_json
        }), output_format));
    } else {
        println!("{}", format_stats(&stats));
    }

    Ok(())
}
