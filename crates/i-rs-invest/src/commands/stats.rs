use crate::models::AssetType;
use crate::presentation::print_header;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_stats() -> Result<()> {
    let store = storage::load_store()?;
    let investments: Vec<_> = store.investments.values().collect();

    if investments.is_empty() {
        println!("No investments found.");
        return Ok(());
    }

    print_header("Investment Portfolio Statistics");
    println!();

    let total_cost: f64 = investments.iter().map(|i| i.total_cost()).sum();
    println!("  {}: ${:.2}", "Total Cost".dimmed(), total_cost);

    let total_value: f64 = investments.iter().filter_map(|i| i.current_value()).sum();

    if total_value > 0.0 {
        let total_profit_loss = total_value - total_cost;
        let total_profit_loss_pct = (total_profit_loss / total_cost) * 100.0;

        println!("  {}: ${:.2}", "Total Value".dimmed(), total_value);
        println!(
            "  {}: ${:.2} ({:+.2}%)",
            "Total Profit/Loss".dimmed(),
            total_profit_loss,
            total_profit_loss_pct
        );
    } else {
        println!("  {}: {}", "Total Value".dimmed(), "N/A".dimmed());
        println!("  {}: {}", "Total Profit/Loss".dimmed(), "N/A".dimmed());
    }

    println!();
    print_header("By Asset Type");
    println!();

    let stocks: Vec<_> = investments
        .iter()
        .filter(|i| i.asset_type == AssetType::Stock)
        .collect();
    let funds: Vec<_> = investments
        .iter()
        .filter(|i| i.asset_type == AssetType::Fund)
        .collect();
    let cryptos: Vec<_> = investments
        .iter()
        .filter(|i| i.asset_type == AssetType::Crypto)
        .collect();

    if !stocks.is_empty() {
        let stock_cost: f64 = stocks.iter().map(|i| i.total_cost()).sum();
        let stock_value: f64 = stocks.iter().filter_map(|i| i.current_value()).sum();
        println!("  {} ({}):", "Stocks".cyan(), stocks.len());
        println!("    {}: ${:.2}", "Cost".dimmed(), stock_cost);
        if stock_value > 0.0 {
            let stock_pl = stock_value - stock_cost;
            let stock_pl_pct = (stock_pl / stock_cost) * 100.0;
            println!(
                "    {}: ${:.2} ({:+.2}%)",
                "Value".dimmed(),
                stock_value,
                stock_pl_pct
            );
        } else {
            println!("    {}: {}", "Value".dimmed(), "N/A".dimmed());
        }
    }

    if !funds.is_empty() {
        let fund_cost: f64 = funds.iter().map(|i| i.total_cost()).sum();
        let fund_value: f64 = funds.iter().filter_map(|i| i.current_value()).sum();
        println!("  {} ({}):", "Funds".cyan(), funds.len());
        println!("    {}: ${:.2}", "Cost".dimmed(), fund_cost);
        if fund_value > 0.0 {
            let fund_pl = fund_value - fund_cost;
            let fund_pl_pct = (fund_pl / fund_cost) * 100.0;
            println!(
                "    {}: ${:.2} ({:+.2}%)",
                "Value".dimmed(),
                fund_value,
                fund_pl_pct
            );
        } else {
            println!("    {}: {}", "Value".dimmed(), "N/A".dimmed());
        }
    }

    if !cryptos.is_empty() {
        let crypto_cost: f64 = cryptos.iter().map(|i| i.total_cost()).sum();
        let crypto_value: f64 = cryptos.iter().filter_map(|i| i.current_value()).sum();
        println!("  {} ({}):", "Cryptos".cyan(), cryptos.len());
        println!("    {}: ${:.2}", "Cost".dimmed(), crypto_cost);
        if crypto_value > 0.0 {
            let crypto_pl = crypto_value - crypto_cost;
            let crypto_pl_pct = (crypto_pl / crypto_cost) * 100.0;
            println!(
                "    {}: ${:.2} ({:+.2}%)",
                "Value".dimmed(),
                crypto_value,
                crypto_pl_pct
            );
        } else {
            println!("    {}: {}", "Value".dimmed(), "N/A".dimmed());
        }
    }

    println!();
    print_header("Individual Performance");
    println!();

    let mut with_prices: Vec<_> = investments
        .iter()
        .filter(|i| i.current_price.is_some())
        .collect();

    with_prices.sort_by(|a, b| {
        let pl_a = a.profit_loss_percentage().unwrap_or(0.0);
        let pl_b = b.profit_loss_percentage().unwrap_or(0.0);
        pl_b.partial_cmp(&pl_a)
            .expect("profit_loss_percentage returns finite f64")
    });

    for (i, inv) in with_prices.iter().enumerate().take(10) {
        let pl = inv
            .profit_loss_percentage()
            .expect("filtered to investments with current_price");

        print!("  {}. {} [{}]: ", i + 1, inv.name, inv.symbol);
        if pl >= 0.0 {
            println!("{}", format!("{pl:+.2}%").green());
        } else {
            println!("{}", format!("{pl:+.2}%").red());
        }
    }

    println!();
    println!(
        "  {} {} investments tracked",
        "Total:".dimmed(),
        investments.len()
    );

    Ok(())
}
