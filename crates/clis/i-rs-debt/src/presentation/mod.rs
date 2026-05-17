use crate::models::{Debt, DebtDetail, DebtRow, Stats};
pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{OutputFormat, print_success};
use owo_colors::OwoColorize;
pub fn format_debt_table(debts: &[&Debt]) -> String {
    let rows: Vec<DebtRow> = debts.iter().map(|d| DebtRow::from_debt(d)).collect();
    if rows.is_empty() {
        return String::new();
    }
    i_rs_core::render_table(&rows)
}
pub fn format_debt_detail(debt: &Debt) -> String {
    let detail = DebtDetail::from(debt);
    let mut lines = Vec::new();
    lines.push(format!("{}: {}", "Name".cyan().bold(), detail.name));
    lines.push(format!("{}: {}", "Type".cyan().bold(), detail.debt_type));
    lines.push(format!(
        "{}: {:.2}",
        "Total Amount".cyan().bold(),
        detail.total_amount
    ));
    lines.push(format!(
        "{}: {:.2}",
        "Paid Amount".cyan().bold(),
        detail.paid_amount
    ));
    lines.push(format!(
        "{}: {:.2}",
        "Remaining".cyan().bold(),
        detail.remaining
    ));

    if let Some(rate) = detail.interest_rate {
        lines.push(format!("{}: {:.2}%", "Interest Rate".cyan().bold(), rate));
    }
    if let Some(due) = detail.due_date {
        lines.push(format!("{}: {}", "Due Date".cyan().bold(), due));
    }
    lines.push(format!(
        "{}: {:.1}%",
        "Progress".cyan().bold(),
        detail.progress
    ));
    if detail.is_overdue
        && let Some(days) = detail.days_overdue
    {
        lines.push(format!("{}: {} days", "Overdue".red().bold(), days));
    }
    lines.push(format!(
        "{}: {}",
        "Payments".cyan().bold(),
        detail.payment_count
    ));
    if !detail.tags.is_empty() {
        lines.push(format!(
            "{}: {}",
            "Tags".cyan().bold(),
            detail.tags.join(", ")
        ));
    }
    if !detail.remark.is_empty() {
        lines.push(format!("{}:", "Remarks".cyan().bold()));
        for r in &detail.remark {
            lines.push(format!("  - {r}"));
        }
    }
    lines.push(format!(
        "{}: {}",
        "Created".cyan().bold(),
        detail.created_at
    ));
    lines.push(format!(
        "{}: {}",
        "Updated".cyan().bold(),
        detail.updated_at
    ));
    lines.join("\n")
}
pub fn format_stats(stats: &Stats) -> String {
    let mut lines = Vec::new();
    lines.push("=== Debt Statistics ===".cyan().bold().to_string());
    lines.push(format!(
        "{}: {}",
        "Total Debts".cyan().bold(),
        stats.total_debts
    ));
    lines.push(format!(
        "{}: {:.2}",
        "Total Amount".cyan().bold(),
        stats.total_amount
    ));
    lines.push(format!(
        "{}: {:.2}",
        "Total Paid".cyan().bold(),
        stats.total_paid
    ));
    lines.push(format!(
        "{}: {:.2}",
        "Total Remaining".cyan().bold(),
        stats.total_remaining
    ));
    lines.push(format!(
        "{}: {}",
        "Overdue Count".cyan().bold(),
        stats.overdue_count
    ));
    if !stats.by_type.is_empty() {
        lines.push(String::new());
        lines.push("=== By Type ===".cyan().bold().to_string());
        for (type_name, type_stats) in &stats.by_type {
            lines.push(String::new());
            lines.push(
                format!("{} ({})", type_name, type_stats.count)
                    .cyan()
                    .bold()
                    .to_string(),
            );
            lines.push(format!("  Total: {:.2}", type_stats.total));
            lines.push(format!("  Paid: {:.2}", type_stats.paid));
            lines.push(format!("  Remaining: {:.2}", type_stats.remaining));
        }
    }
    lines.join("\n")
}
pub fn print_debt_count(count: usize) {
    println!("\n{} {} debts", "Total:".dimmed(), count.to_string().cyan());
}
