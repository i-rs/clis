use crate::presentation::print_warning;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_stats() -> Result<()> {
    let store = storage::load_store()?;

    let total = store.appliances_count();
    let expired = store.expired_count();
    let soon = store.needs_replacement_count();

    if total == 0 {
        print_warning("No appliances to show statistics.");
        return Ok(());
    }

    println!();
    println!("{}", "Appliance Statistics".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    println!("  {:12} {}", "Total:".dimmed(), total.to_string().cyan());
    println!(
        "  {:12} {} (past lifespan)",
        "Expired:".dimmed(),
        expired.to_string().red()
    );
    println!(
        "  {:12} {} (within 90 days)",
        "Replace Soon:".dimmed(),
        soon.to_string().yellow()
    );

    let healthy = total - expired - soon;
    println!(
        "  {:12} {} (good condition)",
        "Healthy:".dimmed(),
        healthy.to_string().green()
    );

    if total > 0 {
        let healthy_pct = (healthy as f64 / total as f64) * 100.0;
        let expired_pct = (expired as f64 / total as f64) * 100.0;
        let soon_pct = (soon as f64 / total as f64) * 100.0;

        println!();
        println!("{}", "Breakdown:".bold().cyan());
        println!("  {:12} {:.1}%", "Healthy:".dimmed(), healthy_pct);
        println!("  {:12} {:.1}%", "Expired:".dimmed(), expired_pct);
        println!("  {:12} {:.1}%", "Replace Soon:".dimmed(), soon_pct);
    }

    Ok(())
}
