use crate::presentation::{output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let investment = if let Some(inv) = store.investments.get(&name) { inv } else {
        println!("{}", format!("Investment '{}' not found", name.red()));
        anyhow::bail!("Investment '{name}' not found");
    };

    match format {
        OutputFormat::Json => {
            let json_output = output_item(investment, format);
            println!("{json_output}");
        }
        OutputFormat::Table | OutputFormat::Default => {
            println!("{}", "Investment Details".cyan().bold());
            println!("{}", "─".repeat(50));
            println!("  {}: {}", "Name".dimmed(), investment.name);
            println!("  {}: {}", "Symbol".dimmed(), investment.symbol);
            println!("  {}: {}", "Type".dimmed(), investment.asset_type.to_string().to_uppercase());
            println!("  {}: {:.4}", "Quantity".dimmed(), investment.quantity);
            println!("  {}: {:.2}", "Buy Price".dimmed(), investment.buy_price);
            println!("  {}: {}", "Buy Date".dimmed(), investment.buy_date.format("%Y-%m-%d"));
            println!("  {}: {:.2}", "Total Cost".dimmed(), investment.total_cost());

            if let Some(current) = investment.current_price {
                println!("  {}: {:.2}", "Current Price".dimmed(), current);
                if let Some(profit_loss) = investment.profit_loss() {
                    let profit_loss_str = if profit_loss >= 0.0 {
                        format!("{:.2} (+{:.2}%)", profit_loss, investment.profit_loss_percentage().expect("current_price is_some checked above"))
                    } else {
                        format!("{:.2} ({:.2}%)", profit_loss, investment.profit_loss_percentage().expect("current_price is_some checked above"))
                    };
                    println!("  {}: {}", "Profit/Loss".dimmed(), profit_loss_str);
                }
                println!("  {}: {:.2}", "Current Value".dimmed(), investment.current_value().expect("current_price is_some checked above"));
            } else {
                println!("  {}: {}", "Current Price".dimmed(), "N/A".dimmed());
                println!("  {}: {}", "Profit/Loss".dimmed(), "N/A".dimmed());
            }

            if !investment.tags.is_empty() {
                println!("  {}: {}", "Tags".dimmed(), investment.tags.join(", "));
            }

            if !investment.remark.is_empty() {
                println!("  {}: {}", "Remark".dimmed(), investment.remark.join(", "));
            }

            println!("  {}: {}", "Created".dimmed(), investment.created_at.format("%Y-%m-%d %H:%M"));
            println!("  {}: {}", "Updated".dimmed(), investment.updated_at.format("%Y-%m-%d %H:%M"));
        }
    }

    Ok(())
}
