use crate::models::ListItem;
use crate::presentation::{OutputFormat, format_table, output_list, print_count, print_warning};
use crate::storage;
use anyhow::Result;

pub fn handle_list(
    tag: Option<String>,
    exercise_type: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = storage::load_store()?;

    let records: Vec<&crate::models::ExerciseRecord> = if let Some(ref t) = tag {
        store.filter_by_tag(t)
    } else if let Some(ref et) = exercise_type {
        store.filter_by_type(et)
    } else {
        store.get_all_records()
    };

    if records.is_empty() {
        if format.is_json() {
            let filter = tag
                .as_ref()
                .or(exercise_type.as_ref())
                .map(std::string::String::as_str);
            println!(
                "{}",
                output_list::<serde_json::Value>(&[], 0, filter, format)
            );
        } else if let Some(ref t) = tag {
            print_warning(&format!("No exercises found with tag '{t}'"));
        } else if let Some(ref et) = exercise_type {
            print_warning(&format!("No exercises found with type '{et}'"));
        } else {
            print_warning("No exercise records found.");
        }
        return Ok(());
    }

    let records_ref: Vec<&crate::models::ExerciseRecord> = records.clone();

    if format.is_json() {
        let items: Vec<ListItem> = records.iter().map(|r| ListItem::from(*r)).collect();
        let filter = tag
            .as_ref()
            .or(exercise_type.as_ref())
            .map(std::string::String::as_str);
        println!("{}", output_list(&items, items.len(), filter, format));
        return Ok(());
    }

    let table = format_table(&records_ref);
    println!("\n{table}");

    print_count(records_ref.len());

    Ok(())
}
