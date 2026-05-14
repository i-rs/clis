use crate::presentation::{format_table, print_birthday_count, print_warning, output_list, OutputFormat};
use crate::storage;
use anyhow::Result;

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let birthdays: Vec<&crate::models::Birthday> = storage::filter_by_tag(&store, tag.as_deref());

    if birthdays.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(&[], 0, tag.as_deref(), format));
        } else {
            print_warning("No birthdays found.");
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize, Clone)]
        struct ListItem {
            name: String,
            birth_date: String,
            year: Option<i32>,
            age: Option<i32>,
            days_until_birthday: i64,
            is_today: bool,
            relationship: String,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let items: Vec<ListItem> = birthdays.iter().map(|b| ListItem {
            name: b.name.clone(),
            birth_date: b.birth_date.clone(),
            year: b.year,
            age: b.age(),
            days_until_birthday: b.days_until_birthday(),
            is_today: b.is_today(),
            relationship: b.relationship.clone(),
            tags: b.tags.clone(),
            remark: b.remark.clone(),
            created_at: b.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: b.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }).collect();

        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    let table = format_table(&birthdays);
    println!("\n{table}");

    print_birthday_count(birthdays.len());

    Ok(())
}
