use crate::models::{PlanRow, RunRecord, RunRow};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::output_list;

pub fn format_run_table(records: &[&RunRecord]) -> String {
    let rows: Vec<RunRow> = records
        .iter()
        .map(|r| RunRow::from_record(r))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn format_plan_table(plans: &[&crate::models::RunPlan]) -> String {
    let rows: Vec<PlanRow> = plans
        .iter()
        .map(|p| PlanRow::from_plan(p))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_run_count(count: usize) {
    println!("\n{} {} runs", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_plan_count(count: usize) {
    println!("\n{} {} plans", "Total:".dimmed(), count.to_string().cyan());
}

pub fn print_stats(store: &crate::models::RunStore) {
    println!("\n{}", "Statistics:".bold().cyan());
    println!("  {:12} {} km", "Total:".dimmed(), format!("{:.2}", store.total_distance()));
    println!("  {:12} {}", "Duration:".dimmed(), crate::models::format_duration(store.total_duration()));
    if let Some(pace) = store.avg_pace() {
        println!("  {:12} {}/km", "Avg Pace:".dimmed(), pace);
    }
    println!("  {:12} {}", "Records:".dimmed(), store.records_count());
    println!("  {:12} {}", "Plans:".dimmed(), store.plans_count());
}
