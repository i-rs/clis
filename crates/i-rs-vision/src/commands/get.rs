use crate::presentation::{print_error, output_item, OutputFormat};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;

pub fn handle_get(date: String, format: OutputFormat) -> Result<()> {
    let date = parse_date(&date)?;

    let store = storage::load_store()?;

    match store.get_record(&date) {
        Some(record) => {
            #[derive(serde::Serialize)]
            struct VisionItem {
                date: String,
                left_sphere: Option<f64>,
                right_sphere: Option<f64>,
                left_cylinder: Option<f64>,
                right_cylinder: Option<f64>,
                left_axis: Option<i32>,
                right_axis: Option<i32>,
                tags: Vec<String>,
                remark: Vec<String>,
                created_at: String,
            }

            let item = VisionItem {
                date: record.date.format("%Y-%m-%d").to_string(),
                left_sphere: record.left_sphere,
                right_sphere: record.right_sphere,
                left_cylinder: record.left_cylinder,
                right_cylinder: record.right_cylinder,
                left_axis: record.left_axis,
                right_axis: record.right_axis,
                tags: record.tags.clone(),
                remark: record.remark.clone(),
                created_at: record.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };

            println!("{}", output_item(&item, format));
        }
        None => {
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::json!({
                    "success": false,
                    "error": {
                        "code": "NOT_FOUND",
                        "message": format!("No record found for {}", date)
                    }
                }));
            } else {
                print_error(&format!("No vision record found for {}", date));
            }
            anyhow::bail!("No record found for {}", date);
        }
    }

    Ok(())
}

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
