use crate::presentation::{format_car_table, print_car_count, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, help = "Filter by car name")]
    pub car: Option<String>,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let cars: Vec<_> = store.list_entries().into_iter().cloned().collect();

    if let Some(car_name) = &args.car {
        let filtered: Vec<_> = cars.iter().filter(|c| &c.name == car_name).collect();
        let cars_refs: Vec<_> = filtered.clone();

        if output_format == OutputFormat::Json {
            let data: Vec<_> = filtered.iter().map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "license_plate": c.license_plate,
                    "brand": c.brand,
                    "model": c.model,
                    "mileage": c.mileage,
                    "tags": c.tags
                })
            }).collect();
            println!("{}", i_rs_core::presentation::output::output_list(&data, data.len(), None, output_format));
        } else {
            let table = format_car_table(&cars_refs);
            if !table.is_empty() {
                println!("{table}");
            }
            print_car_count(filtered.len());
        }
    } else {
        let cars_refs: Vec<_> = cars.iter().collect();

        if output_format == OutputFormat::Json {
            let data: Vec<_> = cars.iter().map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "license_plate": c.license_plate,
                    "brand": c.brand,
                    "model": c.model,
                    "mileage": c.mileage,
                    "tags": c.tags
                })
            }).collect();
            println!("{}", i_rs_core::presentation::output::output_list(&data, data.len(), None, output_format));
        } else {
            let table = format_car_table(&cars_refs);
            if !table.is_empty() {
                println!("{table}");
            }
            print_car_count(cars.len());
        }
    }

    Ok(())
}
