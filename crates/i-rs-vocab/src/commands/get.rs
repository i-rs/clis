use crate::presentation::{OutputFormat, output_error, output_item};
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Style as OwoStyle;

pub fn handle_get(word_key: String, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;

    let word = if let Some(w) = store.get_entry(&word_key) {
        w
    } else {
        let msg = format!("Word '{word_key}' not found");
        if format.is_json() {
            println!("{}", output_error(&msg, "NOT_FOUND", format));
        }
        anyhow::bail!("{msg}");
    };

    if format.is_json() {
        #[derive(serde::Serialize)]
        struct GetOutput {
            word: String,
            definition: String,
            example: Vec<String>,
            status: String,
            status_label: String,
            review_count: u32,
            tags: Vec<String>,
            remark: Vec<String>,
            created_at: String,
            updated_at: String,
        }

        let output = GetOutput {
            word: word.word.clone(),
            definition: word.definition.clone(),
            example: word.example.clone(),
            status: word.status.to_string(),
            status_label: word.status.label().to_string(),
            review_count: word.review_count,
            tags: word.tags.clone(),
            remark: word.remark.clone(),
            created_at: word.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: word.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!("{}", output_item(&output, format));
        return Ok(());
    }

    use crate::presentation::print_header;
    print_header(&format!("Word: {}", word.word.green()));
    println!();

    let style = OwoStyle::new().bold();

    println!(
        "{:16} {}",
        "Definition:".style(style),
        word.definition.cyan()
    );

    if !word.example.is_empty() {
        println!("\n{}:", "Examples".bold());
        for ex in &word.example {
            println!("  • {ex}");
        }
    }

    println!(
        "\n{:16} {} {}",
        "Status:".style(style),
        word.status.emoji(),
        word.status.label().cyan()
    );
    println!(
        "{:16} {}",
        "Reviews:".style(style),
        word.review_count.to_string().magenta()
    );

    if !word.tags.is_empty() {
        println!(
            "\n{:16} {}",
            "Tags:".style(style),
            word.tags
                .iter()
                .map(|t| t.magenta().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !word.remark.is_empty() {
        println!("\n{}:", "Remarks".bold());
        for r in &word.remark {
            println!("  {r}");
        }
    }

    println!(
        "\n{:16} {}",
        "Created:".style(style),
        word.created_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );
    println!(
        "{:16} {}",
        "Updated:".style(style),
        word.updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .dimmed()
    );

    Ok(())
}
