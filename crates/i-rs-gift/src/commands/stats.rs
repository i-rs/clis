use crate::models::GiftType;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_stats() -> Result<()> {
    let store = storage::load_store()?;

    let gifts: Vec<&crate::models::Gift> = store.gifts.values().collect();

    if gifts.is_empty() {
        println!("{}", "No gifts found.".yellow());
        return Ok(());
    }

    let sent_count = gifts.iter().filter(|g| g.gift_type == GiftType::Sent).count();
    let received_count = gifts.iter().filter(|g| g.gift_type == GiftType::Received).count();

    let sent_value: f64 = gifts.iter()
        .filter(|g| g.gift_type == GiftType::Sent)
        .map(|g| g.value)
        .sum();
    let received_value: f64 = gifts.iter()
        .filter(|g| g.gift_type == GiftType::Received)
        .map(|g| g.value)
        .sum();

    let mut occasion_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for gift in &gifts {
        *occasion_counts.entry(gift.occasion.clone()).or_insert(0) += 1;
    }

    let mut recipient_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for gift in &gifts {
        *recipient_counts.entry(gift.recipient.clone()).or_insert(0) += 1;
    }

    println!();
    println!("{}", "Gift Statistics".bold().cyan());
    println!();

    println!("{}", "Overview:".bold().green());
    println!("  {:12} {}", "Total:".dimmed(), gifts.len());
    println!("  {:12} {}", "Sent:".dimmed(), sent_count);
    println!("  {:12} {}", "Received:".dimmed(), received_count);
    println!();

    println!("{}", "Value Summary:".bold().green());
    println!("  {:12} {:.2}", "Total Sent:".dimmed(), sent_value);
    println!("  {:12} {:.2}", "Total Received:".dimmed(), received_value);
    if sent_count > 0 {
        println!("  {:12} {:.2}", "Avg Sent:".dimmed(), sent_value / sent_count as f64);
    }
    if received_count > 0 {
        println!("  {:12} {:.2}", "Avg Received:".dimmed(), received_value / received_count as f64);
    }
    let balance = received_value - sent_value;
    println!("  {:12} {:.2}", "Balance:".dimmed(), balance);
    println!();

    println!("{}", "By Occasion:".bold().green());
    let mut occasions: Vec<_> = occasion_counts.iter().collect();
    occasions.sort_by(|a, b| b.1.cmp(a.1));
    for (occasion, count) in occasions.iter().take(5) {
        println!("  {:12} {} ({})", occasion.dimmed(), count, get_occasion_emoji(occasion));
    }
    println!();

    println!("{}", "Top Recipients:".bold().green());
    let mut recipients: Vec<_> = recipient_counts.iter().collect();
    recipients.sort_by(|a, b| b.1.cmp(a.1));
    for (recipient, count) in recipients.iter().take(5) {
        println!("  {:12} ({} gifts)", recipient.dimmed(), count);
    }

    Ok(())
}

fn get_occasion_emoji(occasion: &str) -> &'static str {
    match occasion.to_lowercase().as_str() {
        "birthday" => "🎂",
        "christmas" => "🎄",
        "wedding" => "💒",
        "anniversary" => "💕",
        "valentine" => "❤️",
        "new year" => "🎉",
        "graduation" => "🎓",
        _ => "🎁",
    }
}
