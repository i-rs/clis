use crate::presentation::{output_error, output_item, print_header, OutputFormat};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(name: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let birthday = if let Some(b) = storage::get_entry(&store, &name) { b } else {
        let msg = format!("Birthday '{name}' not found");
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        } 
        anyhow::bail!("{msg}");
    };

    if matches!(format, OutputFormat::Json) {
        #[derive(serde::Serialize)]
        struct GetOutput {
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

        let output = GetOutput {
            name: birthday.name.clone(),
            birth_date: birthday.birth_date.clone(),
            year: birthday.year,
            age: birthday.age(),
            days_until_birthday: birthday.days_until_birthday(),
            is_today: birthday.is_today(),
            relationship: birthday.relationship.clone(),
            tags: birthday.tags.clone(),
            remark: birthday.remark.clone(),
            created_at: birthday.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: birthday.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    print_header(&format!("Birthday: {}", birthday.name.green()));
    println!();

    let style = OwoStyle::new().bold();
    let days = birthday.days_until_birthday();

    println!("{:16} {}", "Birthday:".style(style), birthday.birth_date.cyan());

    if let Some(year) = birthday.year {
        println!("{:16} {}", "Year:".style(style), year.to_string().cyan());
    }

    if let Some(age) = birthday.age() {
        println!("{:16} {}", "Age:".style(style), age.to_string().cyan());
    }

    let days_status = if birthday.is_today() {
        format!("{}", "TODAY!".red().bold())
    } else if days == 1 {
        format!("{} (tomorrow)", "1 day".yellow().bold())
    } else if days <= 7 {
        format!("{days} days left").yellow().to_string()
    } else {
        format!("{days} days left")
    };
    println!("{:16} {}", "Next Birthday:".style(style), days_status);

    println!("{:16} {}", "Relationship:".style(style), birthday.relationship.cyan());

    if !birthday.tags.is_empty() {
        println!("{:16} {}", "Tags:".style(style), birthday.tags.iter().map(|t| t.magenta().to_string()).collect::<Vec<_>>().join(", "));
    }

    if !birthday.remark.is_empty() {
        println!("\n{}:", "Remark".bold());
        for line in &birthday.remark {
            println!("  {line}");
        }
    }

    println!("\n{:16} {}", "Created:".style(style), birthday.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
    println!("{:16} {}", "Updated:".style(style), birthday.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());

    Ok(())
}
