use crate::models::{Budget, BudgetRow, BudgetStatsRow, Expense, ExpenseRow};
use owo_colors::OwoColorize;

pub use i_rs_core::presentation::output::{output_item, output_list};
pub use i_rs_core::presentation::{
    OutputFormat, print_error, print_header, print_success, print_warning,
};

pub fn format_budget_table(budgets: &[&Budget]) -> String {
    let rows: Vec<BudgetRow> = budgets.iter().map(|b| BudgetRow::from_budget(b)).collect();

    i_rs_core::render_table(&rows)
}

pub fn format_expense_table(expenses: &[&Expense]) -> String {
    let rows: Vec<ExpenseRow> = expenses
        .iter()
        .map(|e| ExpenseRow::from_expense(e))
        .collect();

    i_rs_core::render_table(&rows)
}

pub fn format_budget_stats_table(
    budgets: &[&Budget],
    spent_map: &std::collections::HashMap<String, f64>,
) -> String {
    let rows: Vec<BudgetStatsRow> = budgets
        .iter()
        .map(|b| {
            let spent = spent_map.get(&b.category).copied().unwrap_or(0.0);
            BudgetStatsRow::from_budget(b, spent)
        })
        .collect();

    i_rs_core::render_table(&rows)
}

pub fn print_budget_count(count: usize) {
    println!(
        "\n{} {} budgets",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}

pub fn print_expense_count(count: usize) {
    println!(
        "\n{} {} expenses",
        "Total:".dimmed(),
        count.to_string().cyan()
    );
}

pub fn print_total_spent(total: f64) {
    println!(
        "{} {:.2}",
        "Total Spent:".dimmed(),
        total.to_string().cyan()
    );
}
