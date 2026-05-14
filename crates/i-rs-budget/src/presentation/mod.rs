use crate::models::{Budget, BudgetStatsRow, BudgetRow, Expense, ExpenseRow};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item};

pub fn format_budget_table(budgets: &[&Budget]) -> String {
    let rows: Vec<BudgetRow> = budgets
        .iter()
        .map(|b| BudgetRow::from_budget(b))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn format_expense_table(expenses: &[&Expense]) -> String {
    let rows: Vec<ExpenseRow> = expenses
        .iter()
        .map(|e| ExpenseRow::from_expense(e))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn format_budget_stats_table(budgets: &[&Budget], spent_map: &std::collections::HashMap<String, f64>) -> String {
    let rows: Vec<BudgetStatsRow> = budgets
        .iter()
        .map(|b| {
            let spent = spent_map.get(&b.category).copied().unwrap_or(0.0);
            BudgetStatsRow::from_budget(b, spent)
        })
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_budget_count(count: usize) {
    println!("\n{} {} budgets", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_expense_count(count: usize) {
    println!("\n{} {} expenses", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_total_spent(total: f64) {
    println!("{} {:.2}", "Total Spent:".dimmed(), total.to_string().cyan());
}
