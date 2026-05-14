use crate::presentation::{print_warning, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;

pub fn handle_stats(format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    if store.records.is_empty() {
        print_warning("No exercise records found.");
        return Ok(());
    }

    let total_duration = store.total_duration();
    let total_calories = store.total_calories();
    let records_count = store.records_count();

    let mut type_duration: HashMap<String, u64> = HashMap::new();
    let mut type_count: HashMap<String, usize> = HashMap::new();

    for record in store.get_all_records() {
        *type_duration.entry(record.exercise_type.clone()).or_insert(0) += u64::from(record.duration_minutes);
        *type_count.entry(record.exercise_type.clone()).or_insert(0) += 1;
    }

    let most_common_type = type_count.iter().max_by_key(|&(_, count)| *count).map(|(t, _)| t.clone());
    let longest_type = type_duration.iter().max_by_key(|&(_, dur)| *dur).map(|(t, _)| t.clone());

    if matches!(format, OutputFormat::Json) {
        let stats_json = serde_json::json!({
            "success": true,
            "data": {
                "total_records": records_count,
                "total_duration_minutes": total_duration,
                "total_calories": total_calories,
                "exercise_types": type_count.len(),
                "most_common_type": most_common_type,
                "longest_duration_type": longest_type,
                "type_breakdown": type_duration.iter().map(|(k, v)| {
                    serde_json::json!({
                        "type": k,
                        "total_minutes": v,
                        "count": type_count.get(k).unwrap_or(&0)
                    })
                }).collect::<Vec<_>>()
            }
        });
        println!("{stats_json}");
        return Ok(());
    }

    println!();
    println!("{}", "Exercise Statistics".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    println!();
    println!("  {:20} {}", "Total Records:".dimmed(), records_count.cyan());
    println!("  {:20} {} min", "Total Duration:".dimmed(), total_duration.to_string().cyan());
    println!("  {:20} {} kcal", "Total Calories:".dimmed(), total_calories.to_string().cyan());
    println!("  {:20} {}", "Exercise Types:".dimmed(), type_count.len().to_string().cyan());
    if let Some(ref t) = most_common_type {
        println!("  {:20} {}", "Most Common:".dimmed(), t.cyan());
    }
    if let Some(ref t) = longest_type {
        println!("  {:20} {}", "Longest Type:".dimmed(), t.cyan());
    }
    println!();

    if !type_duration.is_empty() {
        println!("{}", "Breakdown by Type:".bold().green());
        println!();
        let mut sorted_types: Vec<_> = type_duration.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));

        for (exercise_type, &duration) in sorted_types {
            let count = type_count.get(exercise_type).unwrap_or(&0);
            let avg = if *count > 0 { duration as f64 / *count as f64 } else { 0.0 };
            println!("  {:15} {} records  {:>6} min  (avg {:.1} min)", 
                     exercise_type.dimmed(), 
                     count.to_string().yellow(), 
                     duration.to_string().cyan(),
                     format!("{avg:.1}").dimmed());
        }
    }

    Ok(())
}
