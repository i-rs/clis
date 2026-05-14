use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {}

pub fn run(_args: &Args) -> Result<()> {
    println!(r#"
=== i-rs-car Examples ===

# Add a new car
i-rs-car add "My Car" --license-plate "ABC123" --brand "Toyota" --model "Camry" --mileage 50000 --tags family,daily

# List all cars
i-rs-car list

# Get car details
i-rs-car get "My Car"

# Add fuel record
i-rs-car fuel "My Car" --mileage 51000 --fuel-amount 45 --price 8.5 --fuel-type 95 --station "Shell Station"

# Add maintenance record
i-rs-car maintain "My Car" --mileage 52000 --maintenance-type oil_change --cost 300 --shop "Toyota 4S"

# Show car fuel records
i-rs-car get "My Car" --fuel

# Show car maintenance records
i-rs-car get "My Car" --maintain

# View statistics
i-rs-car stats

# View statistics for specific car
i-rs-car stats --car "My Car"

# Update car mileage
i-rs-car update "My Car" --mileage 55000

# Update car tags
i-rs-car update "My Car" --add-tags business

# Delete a car
i-rs-car delete "My Car" --force

# JSON output
i-rs-car list --json
i-rs-car get "My Car" --json
i-rs-car stats --json
"#);

    Ok(())
}
