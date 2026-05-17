use crate::presentation::{OutputFormat, print_success};
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Car name")]
    pub name: String,
    #[arg(short, long, help = "New car name")]
    pub rename: Option<String>,
    #[arg(short = 'l', long = "license-plate", help = "New license plate")]
    pub license_plate: Option<String>,
    #[arg(short, long, help = "New brand")]
    pub brand: Option<String>,
    #[arg(short, long, help = "New model")]
    pub model: Option<String>,
    #[arg(short, long, help = "New mileage")]
    pub mileage: Option<f64>,
    #[arg(short, long, help = "Add tags (comma separated)")]
    pub add_tags: Option<String>,
    #[arg(short, long, help = "Remove tags (comma separated)")]
    pub remove_tags: Option<String>,
    #[arg(short, long, help = "Add remarks")]
    pub add_remark: Vec<String>,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let mut store = storage::load_store()?;

    let mut new_name: Option<String> = None;

    if let Some(ref rename_str) = args.rename {
        if store.cars.contains_key(rename_str) {
            anyhow::bail!("Car '{rename_str}' already exists");
        }

        for record in &mut store.fuel_records {
            if record.car_name == args.name {
                record.car_name = rename_str.clone();
            }
        }
        for record in &mut store.maintenance_records {
            if record.car_name == args.name {
                record.car_name = rename_str.clone();
            }
        }
        store.remove_entry(&args.name);
        new_name = Some(rename_str.clone());
    }

    let final_name = if let Some(ref n) = new_name {
        n.clone()
    } else {
        args.name.clone()
    };

    let car = match store.get_entry_mut(&args.name) {
        Some(c) => c,
        None => {
            anyhow::bail!("Car '{}' not found", args.name);
        }
    };

    if let Some(ref plate) = args.license_plate {
        car.license_plate = plate.clone();
    }

    if let Some(ref b) = args.brand {
        car.brand = b.clone();
    }

    if let Some(ref m) = args.model {
        car.model = m.clone();
    }

    if let Some(mileage) = args.mileage {
        if mileage < car.mileage {
            anyhow::bail!("New mileage cannot be less than current mileage");
        }
        car.mileage = mileage;
    }

    if let Some(ref tags_str) = args.add_tags {
        let new_tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for tag in new_tags {
            if !car.tags.contains(&tag) {
                car.tags.push(tag);
            }
        }
    }

    if let Some(ref tags_str) = args.remove_tags {
        let remove_tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
        car.tags.retain(|t| !remove_tags.contains(t));
    }

    if !args.add_remark.is_empty() {
        car.remark.extend(args.add_remark.clone());
    }

    car.updated_at = Utc::now();

    storage::save_store(&store)?;

    if output_format == OutputFormat::Json {
        println!("{{\"success\": true, \"data\": {{\"name\": \"{final_name}\"}}}}");
    } else {
        print_success(&format!("Car '{final_name}' updated successfully"));
    }

    Ok(())
}
