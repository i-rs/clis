use crate::models::Car;
use crate::presentation::{print_error, print_success, OutputFormat};
use crate::storage;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "Car name")]
    pub name: String,
    #[arg(short, long, help = "License plate number")]
    pub license_plate: String,
    #[arg(short, long, help = "Car brand")]
    pub brand: String,
    #[arg(short, long, help = "Car model")]
    pub model: String,
    #[arg(short, long, help = "Current mileage (km)")]
    pub mileage: f64,
    #[arg(short, long, help = "Tags (comma separated)")]
    pub tags: Option<String>,
    #[arg(short, long, help = "Remarks (multiple)")]
    pub remark: Vec<String>,
}

pub fn run(args: &Args, output_format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if store.cars.contains_key(&args.name) {
        print_error(&format!("Car '{}' already exists", args.name));
        anyhow::bail!("Car '{}' already exists", args.name);
    }

    let mut car = Car::new(
        args.name.clone(),
        args.license_plate.clone(),
        args.brand.clone(),
        args.model.clone(),
        args.mileage,
    );

    if let Some(tags_str) = &args.tags {
        car.tags = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    if !args.remark.is_empty() {
        car.remark = args.remark.clone();
    }

    storage::save_store(&store)?;

    if output_format == OutputFormat::Json {
        println!("{{\"success\": true, \"data\": {{\"name\": \"{}\"}}}}", args.name);
    } else {
        print_success(&format!("Car '{}' created successfully", args.name));
    }

    Ok(())
}
