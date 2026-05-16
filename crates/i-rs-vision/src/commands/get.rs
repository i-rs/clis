use crate::presentation::{OutputFormat, output_item};
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;

pub fn handle_get(date: String, format: OutputFormat) -> Result<()> {
    let date = parse_date(&date)?;

    let store = storage::load_store()?;

    if let Some(record) = store.get_entry(&date) {
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
    } else {
        if format.is_json() {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": {
                        "code": "NOT_FOUND",
                        "message": format!("No record found for {}", date)
                    }
                })
            );
        }
        anyhow::bail!("No record found for {date}");
    }

    Ok(())
}
